use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::Manager;

use crate::logs::{log_debug, log_info};
use crate::profiles::{read_credentials, ProfileCredentials};

// ─── TTL constants ────────────────────────────────────────────────────────────

pub const CATEGORY_TTL: i64 = 6 * 3600;
pub const STREAM_TTL: i64 = 6 * 3600;
pub const EPG_TTL: i64 = 2 * 3600;
pub const M3U8_TTL: i64 = 6 * 3600;

pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn is_fresh(ts: i64, ttl: i64) -> bool {
    ts > 0 && now_ts() - ts < ttl
}

// ─── Shared type ──────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VodCategory {
    pub category_id: String,
    pub category_name: String,
}

// ─── Cache state ──────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct ProfileCache {
    pub live_categories: Vec<VodCategory>,
    pub vod_categories: Vec<VodCategory>,
    pub series_categories: Vec<VodCategory>,
    pub categories_at: i64,
    // key: category_id
    pub live_streams: HashMap<String, Vec<crate::live::LiveStream>>,
    pub vod_streams: HashMap<String, Vec<crate::vods::VodStream>>,
    pub series_items: HashMap<String, Vec<crate::series::SeriesItem>>,
    // key: "live:{cat}", "vod:{cat}", "series:{cat}"
    pub streams_at: HashMap<String, i64>,
    // EPG: in-memory only (short TTL, not persisted)
    #[serde(skip)]
    pub epg: HashMap<String, Vec<serde_json::Value>>,
    #[serde(skip)]
    pub epg_at: HashMap<String, i64>,

    // M3U8-specific (persisted): stream_id (as string) → URL / tvg-id
    #[serde(default)]
    pub m3u8_at: i64,
    #[serde(default)]
    pub m3u8_stream_urls: HashMap<String, String>,
    #[serde(default)]
    pub m3u8_tvg_ids: HashMap<String, String>,

    // M3U8 EPG: in-memory only
    #[serde(skip)]
    pub m3u8_epg: HashMap<String, Vec<serde_json::Value>>,
    #[serde(skip)]
    pub m3u8_epg_at: i64,

    // Xtream bulk XMLTV EPG: in-memory only, keyed by epg_channel_id
    #[serde(skip)]
    pub xtream_epg: HashMap<String, Vec<serde_json::Value>>,
    #[serde(skip)]
    pub xtream_epg_at: i64,
}

pub struct AppCacheState(pub Mutex<HashMap<String, ProfileCache>>);

impl Default for AppCacheState {
    fn default() -> Self {
        AppCacheState(Mutex::new(HashMap::new()))
    }
}

// ─── Disk I/O ─────────────────────────────────────────────────────────────────

fn cache_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("cache.json"))
}

pub fn load_cache_from_disk(app: &tauri::AppHandle) -> HashMap<String, ProfileCache> {
    let Some(path) = cache_path(app) else { return HashMap::new(); };
    if !path.exists() {
        log_debug(app, "cache", "No cache file found, starting fresh");
        return HashMap::new();
    }
    log_debug(app, "cache", format!("Loading cache from {}", path.display()));
    match std::fs::read_to_string(&path) {
        Err(e) => {
            log_info(app, "cache", format!("Failed to read cache file: {e}"));
            HashMap::new()
        }
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => {
                log_debug(app, "cache", "Cache loaded from disk");
                v
            }
            Err(e) => {
                log_info(app, "cache", format!("Failed to parse cache file: {e}"));
                HashMap::new()
            }
        },
    }
}

pub fn save_cache_to_disk(app: &tauri::AppHandle, cache: &HashMap<String, ProfileCache>) {
    if let Some(path) = cache_path(app) {
        match serde_json::to_string(cache) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log_info(app, "cache", format!("Failed to write cache to disk: {e}"));
                } else {
                    log_debug(app, "cache", "Cache saved to disk");
                }
            }
            Err(e) => {
                log_info(app, "cache", format!("Failed to serialize cache: {e}"));
            }
        }
    }
}

// ─── HTTP helper ──────────────────────────────────────────────────────────────

pub async fn api_get<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

// ─── Cache helpers ────────────────────────────────────────────────────────────

pub fn needs_fetch(app: &tauri::AppHandle, name: &str, skey: &str) -> bool {
    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    !cache.get(name).map_or(false, |pc|
        pc.streams_at.get(skey).copied().map_or(false, |ts| is_fresh(ts, STREAM_TTL)))
}

pub fn flush_to_disk(app: &tauri::AppHandle) {
    let snapshot = app.state::<AppCacheState>().0.lock().unwrap().clone();
    save_cache_to_disk(app, &snapshot);
}

// ─── Cache management ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn clear_cache(app: tauri::AppHandle) -> Result<(), String> {
    log_info(&app, "cache", "Clearing cache (user request)");

    *app.state::<AppCacheState>().0.lock().unwrap() = HashMap::new();

    if let Some(path) = cache_path(&app) {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                let msg = format!("Failed to delete cache file: {e}");
                log_info(&app, "cache", &msg);
                msg
            })?;
        }
    }

    log_info(&app, "cache", "Cache cleared");
    Ok(())
}

// ─── Cache priming ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn prime_cache(app: tauri::AppHandle, name: String) -> Result<(), String> {
    log_info(&app, "cache", format!("Starting cache prime for profile '{name}'"));
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if crate::profiles::is_m3u8_profile(&handle, &name) {
            if let Err(e) = crate::m3u8::prime_m3u8_cache(&handle, &name).await {
                log_info(&handle, "cache", format!("M3U8 cache prime failed for '{name}': {e}"));
            } else {
                log_info(&handle, "cache", format!("M3U8 cache prime completed for '{name}'"));
            }
        } else if let Err(e) = do_prime_cache(&handle, &name).await {
            log_info(&handle, "cache", format!("Cache prime failed for '{name}': {e}"));
        } else {
            log_info(&handle, "cache", format!("Cache prime completed for '{name}'"));
        }
    });
    Ok(())
}

async fn do_prime_cache(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let creds = read_credentials(app)?;
    let p = creds.get(name).ok_or("Profile not found")?.clone();

    let (live_cats, vod_cats, series_cats) = prime_categories(app, name, &p).await;

    crate::live::fetch_uncached_streams(app, name, &p, &live_cats).await;
    crate::vods::fetch_uncached_streams(app, name, &p, &vod_cats).await;
    crate::series::fetch_uncached_streams(app, name, &p, &series_cats).await;

    Ok(())
}

async fn prime_categories(
    app: &tauri::AppHandle,
    name: &str,
    p: &ProfileCredentials,
) -> (Vec<VodCategory>, Vec<VodCategory>, Vec<VodCategory>) {
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(name) {
            if is_fresh(pc.categories_at, CATEGORY_TTL)
                && !pc.live_categories.is_empty()
                && !pc.vod_categories.is_empty()
                && !pc.series_categories.is_empty()
            {
                log_debug(app, "cache", format!("Categories cache hit for '{name}'"));
                return (pc.live_categories.clone(), pc.vod_categories.clone(), pc.series_categories.clone());
            }
        }
    }

    log_debug(app, "cache", format!("Fetching categories for '{name}'"));

    let lc_url = format!("{}/player_api.php?username={}&password={}&action=get_live_categories", p.url, p.username, p.password);
    let vc_url = format!("{}/player_api.php?username={}&password={}&action=get_vod_categories", p.url, p.username, p.password);
    let sc_url = format!("{}/player_api.php?username={}&password={}&action=get_series_categories", p.url, p.username, p.password);

    let lc = api_get::<Vec<VodCategory>>(&lc_url).await.unwrap_or_else(|e| {
        log_info(app, "cache", format!("Failed to fetch live categories for '{name}': {e}"));
        vec![]
    });
    let vc = api_get::<Vec<VodCategory>>(&vc_url).await.unwrap_or_else(|e| {
        log_info(app, "cache", format!("Failed to fetch VOD categories for '{name}': {e}"));
        vec![]
    });
    let sc = api_get::<Vec<VodCategory>>(&sc_url).await.unwrap_or_else(|e| {
        log_info(app, "cache", format!("Failed to fetch series categories for '{name}': {e}"));
        vec![]
    });

    log_debug(app, "cache", format!(
        "Fetched categories for '{name}': {} live, {} vod, {} series",
        lc.len(), vc.len(), sc.len()
    ));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name.to_string()).or_default();
        if !lc.is_empty() { pc.live_categories = lc.clone(); }
        if !vc.is_empty() { pc.vod_categories = vc.clone(); }
        if !sc.is_empty() { pc.series_categories = sc.clone(); }
        pc.categories_at = now_ts();
        save_cache_to_disk(app, &cache);
    }

    (lc, vc, sc)
}

// ─── Progress tracking ────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProgressEntry {
    pub position: f64,
    pub duration: f64,
}

pub fn progress_path(app: &tauri::AppHandle, profile: &str) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let safe = profile.replace(['/', '\\', '\0'], "_");
    Ok(data_dir.join(format!("progress_{}.json", safe)))
}

pub fn read_progress_map(app: &tauri::AppHandle, profile: &str) -> HashMap<String, ProgressEntry> {
    let Ok(path) = progress_path(app, profile) else { return HashMap::new(); };
    if !path.exists() { return HashMap::new(); }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn write_progress_entry(app: &tauri::AppHandle, profile: &str, key: &str, position: f64, duration: f64) {
    let mut map = read_progress_map(app, profile);
    if duration > 0.0 && position / duration > 0.95 {
        map.remove(key);
        add_watched_key(app, profile, key);
    } else if position > 5.0 {
        map.insert(key.to_string(), ProgressEntry { position, duration });
    }
    if let Ok(path) = progress_path(app, profile) {
        if let Err(e) = std::fs::write(path, serde_json::to_string_pretty(&map).unwrap()) {
            log_info(app, "cache", format!("Failed to write progress for '{profile}': {e}"));
        }
    }
}

#[tauri::command]
pub fn get_progress(app: tauri::AppHandle, profile: String, keys: Vec<String>) -> HashMap<String, ProgressEntry> {
    let map = read_progress_map(&app, &profile);
    keys.into_iter()
        .filter_map(|k| map.get(&k).cloned().map(|v| (k, v)))
        .collect()
}

// ─── Watched tracking ─────────────────────────────────────────────────────────

fn watched_path(app: &tauri::AppHandle, profile: &str) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let safe = profile.replace(['/', '\\', '\0'], "_");
    Ok(data_dir.join(format!("watched_{}.json", safe)))
}

fn read_watched_set(app: &tauri::AppHandle, profile: &str) -> HashSet<String> {
    let Ok(path) = watched_path(app, profile) else { return HashSet::new(); };
    if !path.exists() { return HashSet::new(); }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .map(HashSet::from_iter)
        .unwrap_or_default()
}

fn add_watched_key(app: &tauri::AppHandle, profile: &str, key: &str) {
    let mut set = read_watched_set(app, profile);
    if set.insert(key.to_string()) {
        if let Ok(path) = watched_path(app, profile) {
            let list: Vec<&String> = set.iter().collect();
            if let Err(e) = std::fs::write(path, serde_json::to_string(&list).unwrap()) {
                log_info(app, "cache", format!("Failed to write watched list for '{profile}': {e}"));
            }
        }
    }
}

#[tauri::command]
pub fn get_watched(app: tauri::AppHandle, profile: String, keys: Vec<String>) -> Vec<String> {
    let set = read_watched_set(&app, &profile);
    keys.into_iter().filter(|k| set.contains(k)).collect()
}

#[tauri::command]
pub fn set_watched(app: tauri::AppHandle, profile: String, keys: Vec<String>) -> Result<(), String> {
    let mut progress_map = read_progress_map(&app, &profile);
    let mut progress_changed = false;
    for key in &keys {
        if progress_map.remove(key).is_some() {
            progress_changed = true;
        }
    }
    if progress_changed {
        if let Ok(path) = progress_path(&app, &profile) {
            if let Err(e) = std::fs::write(path, serde_json::to_string_pretty(&progress_map).unwrap()) {
                log_info(&app, "cache", format!("Failed to write progress for '{profile}': {e}"));
            }
        }
    }

    let mut set = read_watched_set(&app, &profile);
    let mut watched_changed = false;
    for key in &keys {
        if set.insert(key.clone()) {
            watched_changed = true;
        }
    }
    if watched_changed {
        if let Ok(path) = watched_path(&app, &profile) {
            let list: Vec<&String> = set.iter().collect();
            if let Err(e) = std::fs::write(path, serde_json::to_string(&list).unwrap()) {
                log_info(&app, "cache", format!("Failed to write watched list for '{profile}': {e}"));
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_unwatched(app: tauri::AppHandle, profile: String, keys: Vec<String>) -> Result<(), String> {
    let mut progress_map = read_progress_map(&app, &profile);
    let mut progress_changed = false;
    for key in &keys {
        if progress_map.remove(key).is_some() {
            progress_changed = true;
        }
    }
    if progress_changed {
        if let Ok(path) = progress_path(&app, &profile) {
            if let Err(e) = std::fs::write(path, serde_json::to_string_pretty(&progress_map).unwrap()) {
                log_info(&app, "cache", format!("Failed to write progress for '{profile}': {e}"));
            }
        }
    }

    let mut set = read_watched_set(&app, &profile);
    let mut watched_changed = false;
    for key in &keys {
        if set.remove(key) {
            watched_changed = true;
        }
    }
    if watched_changed {
        if let Ok(path) = watched_path(&app, &profile) {
            let list: Vec<&String> = set.iter().collect();
            if let Err(e) = std::fs::write(path, serde_json::to_string(&list).unwrap()) {
                log_info(&app, "cache", format!("Failed to write watched list for '{profile}': {e}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_fresh_zero_timestamp_is_stale() {
        assert!(!is_fresh(0, CATEGORY_TTL));
    }

    #[test]
    fn is_fresh_recent_timestamp() {
        assert!(is_fresh(now_ts(), CATEGORY_TTL));
    }

    #[test]
    fn is_fresh_expired_timestamp() {
        assert!(!is_fresh(now_ts() - CATEGORY_TTL - 1, CATEGORY_TTL));
    }

    #[test]
    fn is_fresh_exactly_at_boundary_is_stale() {
        // age == TTL is not fresh (requires strictly less than TTL)
        assert!(!is_fresh(now_ts() - CATEGORY_TTL, CATEGORY_TTL));
    }

    #[test]
    fn is_fresh_one_second_before_expiry() {
        assert!(is_fresh(now_ts() - CATEGORY_TTL + 1, CATEGORY_TTL));
    }

    #[test]
    fn is_fresh_negative_timestamp_is_stale() {
        assert!(!is_fresh(-1, CATEGORY_TTL));
    }
}
