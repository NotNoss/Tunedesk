use std::collections::HashMap;
use tauri::{Emitter, Manager};

use crate::logs::{log_debug, log_info};
use crate::profiles::{read_credentials, ProfileCredentials};
use crate::cache::{
    api_get, is_fresh, needs_fetch, now_ts, save_cache_to_disk,
    AppCacheState, VodCategory, CATEGORY_TTL, EPG_TTL, STREAM_TTL,
};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LiveStream {
    #[serde(default)]
    pub num: u32,
    #[serde(default)]
    pub name: String,
    pub stream_id: u64,
    #[serde(default)]
    pub stream_icon: String,
    #[serde(default)]
    pub epg_channel_id: String,
}

// Xtream providers vary: stream_id can be an integer, a float, or a quoted string.
// Parse a single raw JSON object into a LiveStream, returning None on invalid data.
fn parse_live_stream(v: &serde_json::Value) -> Option<LiveStream> {
    let raw_id = v.get("stream_id")?;
    let stream_id = raw_id
        .as_u64()
        .or_else(|| raw_id.as_f64().map(|f| f as u64))
        .or_else(|| raw_id.as_str().and_then(|s| s.trim().parse().ok()))
        .filter(|&id| id > 0)?;

    let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
    let num = v.get("num")
        .and_then(|n| n.as_u64().or_else(|| n.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0) as u32;
    let stream_icon = v.get("stream_icon").and_then(|n| n.as_str()).unwrap_or("").to_string();
    let epg_channel_id = v.get("epg_channel_id").and_then(|n| n.as_str()).unwrap_or("").to_string();

    Some(LiveStream { num, name, stream_id, stream_icon, epg_channel_id })
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

    let creds = read_credentials(&app)?;
    let p = creds.get(&name).ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "live", &msg);
        msg
    })?.clone();

    fetch_all_live_and_cache(&app, &name, &p).await?;

    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    Ok(cache
        .get(&name)
        .and_then(|pc| pc.live_streams.get(&category_id))
        .cloned()
        .unwrap_or_default())
}

// Fetch XMLTV EPG for an Xtream profile (one HTTP request) and cache by epg_channel_id.
async fn fetch_xtream_xmltv(app: &tauri::AppHandle, name: &str, p: &ProfileCredentials) -> Result<(), String> {
    let event_id = format!("epg:{name}");
    let _ = app.emit("fetch:start", serde_json::json!({ "id": event_id, "message": format!("Pulling EPG from {name}") }));
    let url = format!("{}/xmltv.php?username={}&password={}", p.url, p.username, p.password);
    log_info(app, "live", format!("Fetching XMLTV EPG for '{name}'"));
    let result = reqwest::get(&url).await
        .map_err(|e| e.to_string())
        .and_then(|r| {
            if r.status().is_success() { Ok(r) }
            else { Err(format!("XMLTV returned HTTP {}", r.status())) }
        });
    let bytes = match result {
        Err(e) => {
            log_info(app, "live", format!("XMLTV fetch failed for '{name}': {e}"));
            let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
            return Err(e);
        }
        Ok(r) => r.bytes().await.map_err(|e| {
            let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
            e.to_string()
        })?,
    };

    let epg = crate::m3u8::parse_epg_xml(&bytes);
    log_info(app, "live", format!("Parsed XMLTV EPG for '{name}': {} channels", epg.len()));

    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name.to_string()).or_default();
        pc.xtream_epg = epg;
        pc.xtream_epg_at = now_ts();
    }
    let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
    Ok(())
}

// Fetch all EPG data once and return it keyed by epg_channel_id.
// The frontend maps each channel's epg_channel_id to its listings.
#[tauri::command]
pub async fn get_all_epg(
    app: tauri::AppHandle,
    name: String,
) -> Result<HashMap<String, Vec<serde_json::Value>>, String> {
    if crate::profiles::is_m3u8_profile(&app, &name) {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        return Ok(cache.get(&name).map(|pc| pc.m3u8_epg.clone()).unwrap_or_default());
    }

    let epg_fresh = {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        cache.get(&name).map_or(false, |pc| is_fresh(pc.xtream_epg_at, EPG_TTL))
    };
    if !epg_fresh {
        let creds = read_credentials(&app)?;
        let p = creds.get(&name).ok_or_else(|| format!("Profile '{name}' not found"))?.clone();
        let _ = fetch_xtream_xmltv(&app, &name, &p).await;
    }

    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    Ok(cache.get(&name).map(|pc| pc.xtream_epg.clone()).unwrap_or_default())
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
    let mut child = crate::playback::mpv_command()
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

// Fetch every live stream from the provider in a single request, group by category_id,
// and cache all categories at once. This avoids per-category server-side filtering bugs
// where some providers omit channels from category-specific responses.
async fn fetch_all_live_and_cache(app: &tauri::AppHandle, name: &str, p: &ProfileCredentials) -> Result<(), String> {
    log_info(app, "live", format!("Fetching all live streams for '{name}'"));
    let event_id = format!("live:{name}");
    let _ = app.emit("fetch:start", serde_json::json!({ "id": event_id, "message": format!("Pulling live channels from {name}") }));
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_live_streams",
        p.url, p.username, p.password
    );
    let raw: Vec<serde_json::Value> = api_get(&url).await.map_err(|e| {
        log_info(app, "live", format!("Failed to fetch live streams for '{name}': {e}"));
        let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
        e
    })?;

    let mut grouped: HashMap<String, Vec<LiveStream>> = HashMap::new();
    let mut skipped = 0usize;
    for v in &raw {
        let cat_id = v.get("category_id")
            .and_then(|c| {
                c.as_str().map(|s| s.to_string())
                    .or_else(|| c.as_u64().map(|n| n.to_string()))
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "0".to_string());
        match parse_live_stream(v) {
            Some(stream) => grouped.entry(cat_id).or_default().push(stream),
            None => skipped += 1,
        }
    }

    let total: usize = grouped.values().map(|v| v.len()).sum();
    log_info(app, "live", format!(
        "Parsed {total} live streams for '{name}' ({} raw, {skipped} skipped, {} categories)",
        raw.len(), grouped.len()
    ));

    let ts = now_ts();
    {
        let state = app.state::<AppCacheState>();
        let mut cache = state.0.lock().unwrap();
        let pc = cache.entry(name.to_string()).or_default();
        for (cat_id, streams) in grouped {
            pc.streams_at.insert(format!("live:{cat_id}"), ts);
            pc.live_streams.insert(cat_id, streams);
        }
        save_cache_to_disk(app, &cache);
    }
    let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
    Ok(())
}

pub async fn fetch_uncached_streams(app: &tauri::AppHandle, name: &str, p: &ProfileCredentials, cats: &[VodCategory]) {
    let any_stale = cats.iter().any(|cat| {
        let skey = format!("live:{}", cat.category_id);
        needs_fetch(app, name, &skey)
    });
    if !any_stale {
        log_debug(app, "live", format!("All live stream categories fresh for '{name}', skipping prime"));
        return;
    }
    if let Err(e) = fetch_all_live_and_cache(app, name, p).await {
        log_info(app, "live", format!("Failed to prime live streams for '{name}': {e}"));
    }
}
