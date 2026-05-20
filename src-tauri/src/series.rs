use tauri::Manager;

use crate::logs::{log_debug, log_info};
use crate::profiles::{read_credentials, ProfileCredentials};
use crate::cache::{
    api_get, flush_to_disk, is_fresh, needs_fetch, now_ts, save_cache_to_disk,
    AppCacheState, VodCategory, CATEGORY_TTL, STREAM_TTL,
};
use crate::playback::launch_mpv;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SeriesItem {
    pub series_id: u64,
    pub name: String,
    #[serde(default)]
    pub cover: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SeriesInfo {
    pub info: serde_json::Value,
    pub episodes: serde_json::Value,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_series_categories(app: tauri::AppHandle, name: String) -> Result<Vec<VodCategory>, String> {
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if is_fresh(pc.categories_at, CATEGORY_TTL) && !pc.series_categories.is_empty() {
                log_debug(&app, "series", format!("Cache hit: series categories for '{name}'"));
                return Ok(pc.series_categories.clone());
            }
        }
    }

    log_debug(&app, "series", format!("Fetching series categories for '{name}'"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "series", &msg);
        msg
    })?.clone();
    let url = format!("{}/player_api.php?username={}&password={}&action=get_series_categories", p.url, p.username, p.password);
    let data: Vec<VodCategory> = api_get(&url).await.map_err(|e| {
        log_info(&app, "series", format!("Failed to fetch series categories for '{name}': {e}"));
        e
    })?;

    log_debug(&app, "series", format!("Fetched {} series categories for '{name}'", data.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.series_categories = data.clone();
        pc.categories_at = now_ts();
        save_cache_to_disk(&app, &cache);
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_series_items(app: tauri::AppHandle, name: String, category_id: String) -> Result<Vec<SeriesItem>, String> {
    let skey = format!("series:{}", category_id);
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if pc.streams_at.get(&skey).copied().map_or(false, |ts| is_fresh(ts, STREAM_TTL)) {
                if let Some(items) = pc.series_items.get(&category_id) {
                    log_debug(&app, "series", format!("Cache hit: series items for '{name}' cat {category_id}"));
                    return Ok(items.clone());
                }
            }
        }
    }

    log_debug(&app, "series", format!("Fetching series items for '{name}' cat {category_id}"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "series", &msg);
        msg
    })?.clone();
    let url = format!("{}/player_api.php?username={}&password={}&action=get_series&category_id={}", p.url, p.username, p.password, category_id);
    let data: Vec<SeriesItem> = api_get(&url).await.map_err(|e| {
        log_info(&app, "series", format!("Failed to fetch series items for '{name}' cat {category_id}: {e}"));
        e
    })?;

    log_debug(&app, "series", format!("Fetched {} series items for '{name}' cat {category_id}", data.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.series_items.insert(category_id.clone(), data.clone());
        pc.streams_at.insert(skey, now_ts());
        save_cache_to_disk(&app, &cache);
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_series_info(app: tauri::AppHandle, name: String, series_id: u64) -> Result<SeriesInfo, String> {
    log_debug(&app, "series", format!("Fetching series info for {series_id} (profile '{name}')"));
    let creds = read_credentials(&app)?;
    let profile = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "series", &msg);
        msg
    })?;
    let url = format!("{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}", profile.url, profile.username, profile.password, series_id);
    api_get(&url).await.map_err(|e| {
        log_info(&app, "series", format!("Failed to fetch series info for {series_id}: {e}"));
        e
    })
}

#[tauri::command]
pub async fn play_episode(
    app: tauri::AppHandle,
    name: String,
    episode_id: String,
    container_extension: String,
    start_over: bool,
) -> Result<(), String> {
    log_info(&app, "series", format!("Playing episode {episode_id} (profile '{name}')"));
    let creds = read_credentials(&app)?;
    let profile = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "series", &msg);
        msg
    })?.clone();
    let ext = if container_extension.is_empty() { "mkv".to_string() } else { container_extension };
    let url = format!("{}/series/{}/{}/{}.{}", profile.url, profile.username, profile.password, episode_id, ext);
    launch_mpv(&app, url, format!("episode_{}", episode_id), start_over, &name).await
}

// ─── Cache priming helper ─────────────────────────────────────────────────────

pub async fn fetch_uncached_streams(app: &tauri::AppHandle, name: &str, p: &ProfileCredentials, cats: &[VodCategory]) {
    let mut dirty = false;
    for cat in cats {
        let skey = format!("series:{}", cat.category_id);
        if !needs_fetch(app, name, &skey) { continue; }
        log_debug(app, "series", format!("Priming series items for '{name}' cat {}", cat.category_id));
        let url = format!("{}/player_api.php?username={}&password={}&action=get_series&category_id={}", p.url, p.username, p.password, cat.category_id);
        let Ok(items) = api_get::<Vec<SeriesItem>>(&url).await else {
            log_info(app, "series", format!("Failed to prime series items for '{name}' cat {}", cat.category_id));
            continue;
        };
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name.to_string()).or_default();
        pc.streams_at.insert(skey, now_ts());
        pc.series_items.insert(cat.category_id.clone(), items);
        dirty = true;
    }
    if dirty { flush_to_disk(app); }
}
