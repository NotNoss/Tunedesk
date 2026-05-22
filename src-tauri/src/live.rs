use tauri::Manager;

use crate::logs::{log_debug, log_info};
use crate::profiles::{read_credentials, ProfileCredentials};
use crate::cache::{
    api_get, flush_to_disk, is_fresh, needs_fetch, now_ts, save_cache_to_disk,
    AppCacheState, VodCategory, CATEGORY_TTL, EPG_TTL, STREAM_TTL,
};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LiveStream {
    #[serde(default)]
    pub num: u32,
    pub name: String,
    pub stream_id: u64,
    #[serde(default)]
    pub stream_icon: String,
    #[serde(default)]
    pub epg_channel_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct ChannelEpg {
    #[serde(default)]
    pub epg_listings: Vec<serde_json::Value>,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_live_categories(app: tauri::AppHandle, name: String) -> Result<Vec<VodCategory>, String> {
    if crate::profiles::is_m3u8_profile(&app, &name) {
        return crate::m3u8::get_live_categories_m3u8(&app, &name).await;
    }
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if is_fresh(pc.categories_at, CATEGORY_TTL) && !pc.live_categories.is_empty() {
                log_debug(&app, "live", format!("Cache hit: live categories for '{name}'"));
                return Ok(pc.live_categories.clone());
            }
        }
    }

    log_debug(&app, "live", format!("Fetching live categories for '{name}'"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "live", &msg);
        msg
    })?.clone();
    let url = format!("{}/player_api.php?username={}&password={}&action=get_live_categories", p.url, p.username, p.password);
    let data: Vec<VodCategory> = api_get(&url).await.map_err(|e| {
        log_info(&app, "live", format!("Failed to fetch live categories for '{name}': {e}"));
        e
    })?;

    log_debug(&app, "live", format!("Fetched {} live categories for '{name}'", data.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.live_categories = data.clone();
        pc.categories_at = now_ts();
        save_cache_to_disk(&app, &cache);
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_live_streams(app: tauri::AppHandle, name: String, category_id: String) -> Result<Vec<LiveStream>, String> {
    if crate::profiles::is_m3u8_profile(&app, &name) {
        return crate::m3u8::get_live_streams_m3u8(&app, &name, &category_id).await;
    }
    let skey = format!("live:{}", category_id);
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if pc.streams_at.get(&skey).copied().map_or(false, |ts| is_fresh(ts, STREAM_TTL)) {
                if let Some(streams) = pc.live_streams.get(&category_id) {
                    log_debug(&app, "live", format!("Cache hit: live streams for '{name}' cat {category_id}"));
                    return Ok(streams.clone());
                }
            }
        }
    }

    log_debug(&app, "live", format!("Fetching live streams for '{name}' cat {category_id}"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "live", &msg);
        msg
    })?.clone();
    let url = format!("{}/player_api.php?username={}&password={}&action=get_live_streams&category_id={}", p.url, p.username, p.password, category_id);
    let data: Vec<LiveStream> = api_get(&url).await.map_err(|e| {
        log_info(&app, "live", format!("Failed to fetch live streams for '{name}' cat {category_id}: {e}"));
        e
    })?;

    log_debug(&app, "live", format!("Fetched {} live streams for '{name}' cat {category_id}", data.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.live_streams.insert(category_id.clone(), data.clone());
        pc.streams_at.insert(skey, now_ts());
        save_cache_to_disk(&app, &cache);
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_channel_epg(app: tauri::AppHandle, name: String, stream_id: u64) -> Result<ChannelEpg, String> {
    if crate::profiles::is_m3u8_profile(&app, &name) {
        let listings = crate::m3u8::get_channel_epg_m3u8(&app, &name, stream_id)?;
        return Ok(ChannelEpg { epg_listings: listings });
    }
    let key = stream_id.to_string();
    {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        if let Some(pc) = cache.get(&name) {
            if pc.epg_at.get(&key).copied().map_or(false, |ts| is_fresh(ts, EPG_TTL)) {
                if let Some(listings) = pc.epg.get(&key) {
                    log_debug(&app, "live", format!("Cache hit: EPG for stream {stream_id}"));
                    return Ok(ChannelEpg { epg_listings: listings.clone() });
                }
            }
        }
    }

    log_debug(&app, "live", format!("Fetching EPG for stream {stream_id} (profile '{name}')"));
    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "live", &msg);
        msg
    })?.clone();
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}&limit=10",
        p.url, p.username, p.password, stream_id
    );
    let result: ChannelEpg = reqwest::get(&url)
        .await
        .map_err(|e| {
            log_info(&app, "live", format!("Failed to fetch EPG for stream {stream_id}: {e}"));
            e.to_string()
        })?
        .json::<ChannelEpg>()
        .await
        .unwrap_or_default();

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name).or_default();
        pc.epg.insert(key.clone(), result.epg_listings.clone());
        pc.epg_at.insert(key, now_ts());
        // EPG is in-memory only; no disk save here
    }
    Ok(result)
}

#[tauri::command]
pub async fn play_live(app: tauri::AppHandle, name: String, stream_id: u64) -> Result<(), String> {
    log_info(&app, "live", format!("Playing live stream {stream_id} (profile '{name}')"));
    let url = if crate::profiles::is_m3u8_profile(&app, &name) {
        crate::m3u8::get_stream_url_m3u8(&app, &name, stream_id)
            .ok_or_else(|| {
                let msg = "Stream URL not in cache — try again after profile loads";
                log_info(&app, "live", msg);
                msg.to_string()
            })?
    } else {
        let creds = read_credentials(&app)?;
        let profile = creds.get(&name).ok_or_else(|| {
            let msg = format!("Profile '{name}' not found");
            log_info(&app, "live", &msg);
            msg
        })?;
        format!("{}/{}/{}/{}.ts", profile.url, profile.username, profile.password, stream_id)
    };

    log_debug(&app, "live", format!("Live stream URL resolved for stream {stream_id}"));

    let window = app.get_webview_window("main");
    if let Some(w) = &window {
        let _ = w.hide();
    }

    let result = run_live_mpv(&app, url).await;

    if let Some(w) = &window {
        let _ = w.show();
    }

    result?;
    log_info(&app, "live", format!("Live stream {stream_id} playback ended"));
    Ok(())
}

async fn run_live_mpv(app: &tauri::AppHandle, url: String) -> Result<(), String> {
    let mut args = vec![url, "--fs".to_string()];
    if let Ok(resource_dir) = app.path().resource_dir() {
        let config_dir = resource_dir.join("mpv-config");
        if config_dir.exists() {
            args.push(format!("--config-dir={}", config_dir.to_string_lossy()));
        }
    }

    let mpv = crate::playback::mpv_executable();
    let mut child = std::process::Command::new(&mpv)
        .args(&args)
        .spawn()
        .map_err(|e| {
            let msg = format!("Failed to launch mpv ({}): {e}", mpv.display());
            log_info(app, "live", &msg);
            msg
        })?;

    tauri::async_runtime::spawn_blocking(move || child.wait())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| {
            log_info(app, "live", format!("mpv exited with error: {e}"));
            e.to_string()
        })?;

    Ok(())
}

// ─── Cache priming helper ─────────────────────────────────────────────────────

pub async fn fetch_uncached_streams(app: &tauri::AppHandle, name: &str, p: &ProfileCredentials, cats: &[VodCategory]) {
    let mut dirty = false;
    for cat in cats {
        let skey = format!("live:{}", cat.category_id);
        if !needs_fetch(app, name, &skey) { continue; }
        log_debug(app, "live", format!("Priming live streams for '{name}' cat {}", cat.category_id));
        let url = format!("{}/player_api.php?username={}&password={}&action=get_live_streams&category_id={}", p.url, p.username, p.password, cat.category_id);
        let Ok(items) = api_get::<Vec<LiveStream>>(&url).await else {
            log_info(app, "live", format!("Failed to prime live streams for '{name}' cat {}", cat.category_id));
            continue;
        };
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name.to_string()).or_default();
        pc.streams_at.insert(skey, now_ts());
        pc.live_streams.insert(cat.category_id.clone(), items);
        dirty = true;
    }
    if dirty { flush_to_disk(app); }
}
