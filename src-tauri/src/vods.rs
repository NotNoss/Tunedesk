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
pub struct VodStream {
    pub stream_id: u64,
    pub name: String,
    #[serde(default)]
    pub stream_icon: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VodInfo {
    pub info: serde_json::Value,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_vod_categories(app: tauri::AppHandle, name: String) -> Result<Vec<VodCategory>, String> {
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if is_fresh(pc.categories_at, CATEGORY_TTL) && !pc.vod_categories.is_empty() {
                log_debug(&app, "vods", format!("Cache hit: VOD categories for '{name}'"));
                return Ok(pc.vod_categories.clone());
            }
        }
    }

    log_debug(&app, "vods", format!("Fetching VOD categories for '{name}'"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "vods", &msg);
        msg
    })?.clone();
    let url = format!("{}/player_api.php?username={}&password={}&action=get_vod_categories", p.url, p.username, p.password);
    let data: Vec<VodCategory> = api_get(&url).await.map_err(|e| {
        log_info(&app, "vods", format!("Failed to fetch VOD categories for '{name}': {e}"));
        e
    })?;

    log_debug(&app, "vods", format!("Fetched {} VOD categories for '{name}'", data.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.vod_categories = data.clone();
        pc.categories_at = now_ts();
        save_cache_to_disk(&app, &cache);
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_vod_streams(app: tauri::AppHandle, name: String, category_id: String) -> Result<Vec<VodStream>, String> {
    let skey = format!("vod:{}", category_id);
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if pc.streams_at.get(&skey).copied().map_or(false, |ts| is_fresh(ts, STREAM_TTL)) {
                if let Some(streams) = pc.vod_streams.get(&category_id) {
                    log_debug(&app, "vods", format!("Cache hit: VOD streams for '{name}' cat {category_id}"));
                    return Ok(streams.clone());
                }
            }
        }
    }

    log_debug(&app, "vods", format!("Fetching VOD streams for '{name}' cat {category_id}"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "vods", &msg);
        msg
    })?.clone();
    let url = format!("{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}", p.url, p.username, p.password, category_id);
    let data: Vec<VodStream> = api_get(&url).await.map_err(|e| {
        log_info(&app, "vods", format!("Failed to fetch VOD streams for '{name}' cat {category_id}: {e}"));
        e
    })?;

    log_debug(&app, "vods", format!("Fetched {} VOD streams for '{name}' cat {category_id}", data.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.vod_streams.insert(category_id.clone(), data.clone());
        pc.streams_at.insert(skey, now_ts());
        save_cache_to_disk(&app, &cache);
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_vod_info(app: tauri::AppHandle, name: String, vod_id: u64) -> Result<VodInfo, String> {
    log_debug(&app, "vods", format!("Fetching VOD info for stream {vod_id} (profile '{name}')"));
    let creds = read_credentials(&app)?;
    let profile = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "vods", &msg);
        msg
    })?;
    let url = format!("{}/player_api.php?username={}&password={}&action=get_vod_info&vod_id={}", profile.url, profile.username, profile.password, vod_id);
    api_get(&url).await.map_err(|e| {
        log_info(&app, "vods", format!("Failed to fetch VOD info for stream {vod_id}: {e}"));
        e
    })
}

#[tauri::command]
pub async fn play_vod(
    app: tauri::AppHandle,
    name: String,
    stream_id: u64,
    container_extension: String,
    start_over: bool,
) -> Result<(), String> {
    log_info(&app, "vods", format!("Playing VOD {stream_id} (profile '{name}')"));
    let creds = read_credentials(&app)?;
    let profile = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "vods", &msg);
        msg
    })?.clone();
    let ext = if container_extension.is_empty() { "mkv".to_string() } else { container_extension };
    let url = format!("{}/movie/{}/{}/{}.{}", profile.url, profile.username, profile.password, stream_id, ext);
    launch_mpv(&app, url, format!("movie_{}", stream_id), start_over, &name).await
}

// ─── Cache priming helper ─────────────────────────────────────────────────────

pub async fn fetch_uncached_streams(app: &tauri::AppHandle, name: &str, p: &ProfileCredentials, cats: &[VodCategory]) {
    let mut dirty = false;
    for cat in cats {
        let skey = format!("vod:{}", cat.category_id);
        if !needs_fetch(app, name, &skey) { continue; }
        log_debug(app, "vods", format!("Priming VOD streams for '{name}' cat {}", cat.category_id));
        let url = format!("{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}", p.url, p.username, p.password, cat.category_id);
        let Ok(items) = api_get::<Vec<VodStream>>(&url).await else {
            log_info(app, "vods", format!("Failed to prime VOD streams for '{name}' cat {}", cat.category_id));
            continue;
        };
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name.to_string()).or_default();
        pc.streams_at.insert(skey, now_ts());
        pc.vod_streams.insert(cat.category_id.clone(), items);
        dirty = true;
    }
    if dirty { flush_to_disk(app); }
}
