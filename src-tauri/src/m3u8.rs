use std::collections::HashMap;
use tauri::{Emitter, Manager};

use crate::logs::{log_debug, log_info};
use crate::cache::{
    is_fresh, now_ts, save_cache_to_disk, AppCacheState, VodCategory, EPG_TTL, M3U8_TTL,
};
use crate::live::LiveStream;
use crate::profiles::read_m3u8_credentials;

// ─── Playlist-only load (fast path for navigation) ───────────────────────────

async fn ensure_playlist(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let fresh = {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        cache
            .get(name)
            .map_or(false, |pc| is_fresh(pc.m3u8_at, M3U8_TTL) && !pc.live_categories.is_empty())
    };
    if fresh {
        log_debug(app, "m3u8", format!("Playlist cache hit for '{name}'"));
        return Ok(());
    }
    log_debug(app, "m3u8", format!("Playlist cache miss for '{name}', fetching"));
    let creds = read_m3u8_credentials(app)?;
    let p = creds.get(name).ok_or_else(|| {
        let msg = format!("M3U8 profile '{name}' not found");
        log_info(app, "m3u8", &msg);
        msg
    })?.clone();
    fetch_m3u8(app, name, &p.m3u_url).await
}

// ─── Full priming: playlist + EPG (called by prime_cache on startup) ──────────

pub async fn prime_m3u8_cache(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    log_info(app, "m3u8", format!("Priming M3U8 cache for '{name}'"));
    let creds = read_m3u8_credentials(app)?;
    let p = creds.get(name).ok_or_else(|| {
        let msg = format!("M3U8 profile '{name}' not found");
        log_info(app, "m3u8", &msg);
        msg
    })?.clone();

    let m3u_fresh = {
        let state = app.state::<AppCacheState>();
        let cache = state.0.lock().unwrap();
        cache
            .get(name)
            .map_or(false, |pc| is_fresh(pc.m3u8_at, M3U8_TTL) && !pc.live_categories.is_empty())
    };
    if !m3u_fresh {
        fetch_m3u8(app, name, &p.m3u_url).await?;
    } else {
        log_debug(app, "m3u8", format!("M3U8 playlist still fresh for '{name}', skipping fetch"));
    }

    if !p.epg_url.is_empty() {
        let epg_fresh = {
            let state = app.state::<AppCacheState>();
            let cache = state.0.lock().unwrap();
            cache
                .get(name)
                .map_or(false, |pc| is_fresh(pc.m3u8_epg_at, EPG_TTL))
        };
        if !epg_fresh {
            fetch_epg(app, name, &p.epg_url).await?;
        } else {
            log_debug(app, "m3u8", format!("EPG still fresh for '{name}', skipping fetch"));
        }
    }

    Ok(())
}

// ─── Helpers called from live.rs ──────────────────────────────────────────────

pub async fn get_live_categories_m3u8(
    app: &tauri::AppHandle,
    name: &str,
) -> Result<Vec<VodCategory>, String> {
    ensure_playlist(app, name).await?;
    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    Ok(cache
        .get(name)
        .map(|pc| pc.live_categories.clone())
        .unwrap_or_default())
}

pub async fn get_live_streams_m3u8(
    app: &tauri::AppHandle,
    name: &str,
    category_id: &str,
) -> Result<Vec<LiveStream>, String> {
    ensure_playlist(app, name).await?;
    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    Ok(cache
        .get(name)
        .and_then(|pc| pc.live_streams.get(category_id))
        .cloned()
        .unwrap_or_default())
}

// EPG is populated by prime_cache (background); this just reads what's ready.
pub fn get_channel_epg_m3u8(
    app: &tauri::AppHandle,
    name: &str,
    stream_id: u64,
) -> Result<Vec<serde_json::Value>, String> {
    let key = stream_id.to_string();
    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    let listings = cache
        .get(name)
        .and_then(|pc| {
            pc.m3u8_tvg_ids
                .get(&key)
                .and_then(|tvg_id| pc.m3u8_epg.get(tvg_id))
        })
        .cloned()
        .unwrap_or_default();
    Ok(listings)
}

pub fn get_stream_url_m3u8(app: &tauri::AppHandle, name: &str, stream_id: u64) -> Option<String> {
    let state = app.state::<AppCacheState>();
    let cache = state.0.lock().unwrap();
    cache
        .get(name)
        .and_then(|pc| pc.m3u8_stream_urls.get(&stream_id.to_string()))
        .cloned()
}

// ─── M3U fetch + parse ────────────────────────────────────────────────────────

async fn fetch_m3u8(app: &tauri::AppHandle, name: &str, url: &str) -> Result<(), String> {
    log_info(app, "m3u8", format!("Fetching M3U8 playlist for '{name}'"));
    let event_id = format!("m3u8:{name}");
    let _ = app.emit("fetch:start", serde_json::json!({ "id": event_id, "message": format!("Pulling playlist from {name}") }));
    let resp = reqwest::get(url).await.map_err(|e| {
        let msg = format!("Failed to fetch M3U8 playlist for '{name}': {e}");
        log_info(app, "m3u8", &msg);
        let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
        msg
    })?;
    if !resp.status().is_success() {
        let msg = format!("M3U8 URL returned HTTP {} for '{name}'", resp.status());
        log_info(app, "m3u8", &msg);
        let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
        return Err(msg);
    }
    let text = resp.text().await.map_err(|e| {
        let msg = format!("Failed to read M3U8 response body for '{name}': {e}");
        log_info(app, "m3u8", &msg);
        let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
        msg
    })?;
    log_debug(app, "m3u8", format!("Parsing M3U8 playlist for '{name}' ({} bytes)", text.len()));
    parse_and_store_m3u8(app, name, &text);
    log_info(app, "m3u8", format!("M3U8 playlist loaded for '{name}'"));
    let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
    Ok(())
}

fn parse_and_store_m3u8(app: &tauri::AppHandle, name: &str, text: &str) {
    // Preserve insertion order for categories using a Vec + a set for dedup.
    let mut seen_groups: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut category_order: Vec<VodCategory> = Vec::new();
    let mut live_streams: HashMap<String, Vec<LiveStream>> = HashMap::new();
    let mut stream_urls: HashMap<String, String> = HashMap::new();
    let mut tvg_ids: HashMap<String, String> = HashMap::new();
    let mut stream_id: u64 = 1;

    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if !line.starts_with("#EXTINF:") {
            continue;
        }
        let group = attr_value(line, "group-title").unwrap_or("Uncategorized");
        let tvg_id = attr_value(line, "tvg-id").unwrap_or("");
        let tvg_logo = attr_value(line, "tvg-logo").unwrap_or("");
        let channel_name = line.rsplit(',').next().unwrap_or("").trim();

        let url_line = loop {
            match lines.next() {
                None => break "",
                Some(l) => {
                    let l = l.trim();
                    if !l.is_empty() && !l.starts_with('#') {
                        break l;
                    }
                }
            }
        };
        if url_line.is_empty() {
            continue;
        }

        if seen_groups.insert(group.to_string()) {
            category_order.push(VodCategory {
                category_id: group.to_string(),
                category_name: group.to_string(),
            });
        }

        let sid_str = stream_id.to_string();
        live_streams
            .entry(group.to_string())
            .or_default()
            .push(LiveStream {
                num: stream_id as u32,
                name: channel_name.to_string(),
                stream_id,
                stream_icon: tvg_logo.to_string(),
                epg_channel_id: tvg_id.to_string(),
            });
        stream_urls.insert(sid_str.clone(), url_line.to_string());
        if !tvg_id.is_empty() {
            tvg_ids.insert(sid_str, tvg_id.to_string());
        }
        stream_id += 1;
    }

    log_debug(app, "m3u8", format!(
        "Parsed M3U8 for '{name}': {} categories, {} streams",
        category_order.len(), stream_id - 1
    ));

    let state = app.state::<AppCacheState>();
    let mut cache = state.0.lock().unwrap();
    let pc = cache.entry(name.to_string()).or_default();
    pc.live_categories = category_order;
    pc.live_streams = live_streams;
    pc.m3u8_stream_urls = stream_urls;
    pc.m3u8_tvg_ids = tvg_ids;
    pc.m3u8_at = now_ts();
    pc.categories_at = now_ts();
    save_cache_to_disk(app, &cache);
}

fn attr_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", key);
    let start = line.find(needle.as_str())? + needle.len();
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

// ─── EPG fetch + parse ────────────────────────────────────────────────────────

async fn fetch_epg(app: &tauri::AppHandle, name: &str, url: &str) -> Result<(), String> {
    log_info(app, "m3u8", format!("Fetching EPG for '{name}'"));
    let event_id = format!("epg:{name}");
    let _ = app.emit("fetch:start", serde_json::json!({ "id": event_id, "message": format!("Pulling EPG from {name}") }));
    let bytes = reqwest::get(url)
        .await
        .map_err(|e| {
            let msg = format!("Failed to fetch EPG for '{name}': {e}");
            log_info(app, "m3u8", &msg);
            let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
            msg
        })?
        .bytes()
        .await
        .map_err(|e| {
            let msg = format!("Failed to read EPG response body for '{name}': {e}");
            log_info(app, "m3u8", &msg);
            let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
            msg
        })?;
    log_debug(app, "m3u8", format!("Parsing EPG for '{name}' ({} bytes)", bytes.len()));
    parse_and_store_epg(app, name, &bytes);
    log_info(app, "m3u8", format!("EPG loaded for '{name}'"));
    let _ = app.emit("fetch:end", serde_json::json!({ "id": event_id }));
    Ok(())
}

// Parse XMLTV bytes into a map of channel_id -> EPG listings.
// Shared by both M3U8 profiles (from epg_url) and Xtream profiles (from xmltv.php).
pub fn parse_epg_xml(data: &[u8]) -> HashMap<String, Vec<serde_json::Value>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text_start = true;
    reader.config_mut().trim_text_end = true;

    let mut epg: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut in_programme = false;
    let mut current_channel = String::new();
    let mut current_start: i64 = 0;
    let mut current_stop: i64 = 0;
    let mut current_title = String::new();
    let mut current_desc = String::new();
    let mut capture_title = false;
    let mut capture_desc = false;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"programme" => {
                    in_programme = true;
                    current_title.clear();
                    current_desc.clear();
                    current_channel.clear();
                    current_start = 0;
                    current_stop = 0;
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"channel" => {
                                current_channel =
                                    String::from_utf8_lossy(&attr.value).into_owned()
                            }
                            b"start" => {
                                current_start =
                                    parse_xmltv_time(&String::from_utf8_lossy(&attr.value))
                            }
                            b"stop" => {
                                current_stop =
                                    parse_xmltv_time(&String::from_utf8_lossy(&attr.value))
                            }
                            _ => {}
                        }
                    }
                }
                b"title" if in_programme => capture_title = true,
                b"desc" if in_programme => capture_desc = true,
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if let Ok(text) = e.unescape() {
                    if capture_title {
                        current_title = text.into_owned();
                    } else if capture_desc {
                        current_desc = text.into_owned();
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"programme" if in_programme => {
                    if !current_channel.is_empty() && current_start > 0 {
                        epg.entry(current_channel.clone())
                            .or_default()
                            .push(serde_json::json!({
                                "title": b64_encode(&current_title),
                                "description": b64_encode(&current_desc),
                                "start_timestamp": current_start,
                                "stop_timestamp": current_stop,
                            }));
                    }
                    in_programme = false;
                    capture_title = false;
                    capture_desc = false;
                }
                b"title" => capture_title = false,
                b"desc" => capture_desc = false,
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    epg
}

fn parse_and_store_epg(app: &tauri::AppHandle, name: &str, data: &[u8]) {
    let epg = parse_epg_xml(data);
    log_debug(app, "m3u8", format!("Parsed EPG for '{name}': {} channels", epg.len()));
    let state = app.state::<AppCacheState>();
    let mut cache = state.0.lock().unwrap();
    let pc = cache.entry(name.to_string()).or_default();
    pc.m3u8_epg = epg;
    pc.m3u8_epg_at = now_ts();
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn b64_encode(s: &str) -> String {
    const CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(CHARS[((b0 >> 2) & 0x3f) as usize] as char);
        out.push(CHARS[(((b0 << 4) | (b1 >> 4)) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARS[(((b1 << 2) | (b2 >> 6)) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn parse_xmltv_time(s: &str) -> i64 {
    // Format: "YYYYMMDDHHMMSS +HHMM" (timezone optional)
    let s = s.trim();
    let (dt, tz) = s.split_once(' ').unwrap_or((s, "+0000"));
    if dt.len() < 14 {
        return 0;
    }
    let y: i64 = dt[0..4].parse().unwrap_or(0);
    let mo: i64 = dt[4..6].parse().unwrap_or(0);
    let d: i64 = dt[6..8].parse().unwrap_or(0);
    let h: i64 = dt[8..10].parse().unwrap_or(0);
    let mi: i64 = dt[10..12].parse().unwrap_or(0);
    let sc: i64 = dt[12..14].parse().unwrap_or(0);

    let sign: i64 = if tz.starts_with('-') { -1 } else { 1 };
    let tz_digits: &str = tz.trim_start_matches(['+', '-']);
    let offset = if tz_digits.len() >= 4 {
        let th: i64 = tz_digits[0..2].parse().unwrap_or(0);
        let tm: i64 = tz_digits[2..4].parse().unwrap_or(0);
        sign * (th * 3600 + tm * 60)
    } else {
        0
    };

    date_to_unix(y, mo, d) + h * 3600 + mi * 60 + sc - offset
}

fn date_to_unix(year: i64, month: i64, day: i64) -> i64 {
    // Gregorian calendar → Julian Day Number → Unix epoch seconds
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    (jdn - 2_440_588) * 86400
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── attr_value ───────────────────────────────────────────────────────────

    #[test]
    fn attr_value_extracts_known_keys() {
        let line = r#"#EXTINF:-1 tvg-id="BBC1" tvg-logo="http://logo.png" group-title="News",BBC One"#;
        assert_eq!(attr_value(line, "tvg-id"), Some("BBC1"));
        assert_eq!(attr_value(line, "tvg-logo"), Some("http://logo.png"));
        assert_eq!(attr_value(line, "group-title"), Some("News"));
    }

    #[test]
    fn attr_value_missing_key_returns_none() {
        let line = r#"#EXTINF:-1 tvg-id="BBC1",BBC One"#;
        assert_eq!(attr_value(line, "missing-key"), None);
    }

    #[test]
    fn attr_value_empty_value() {
        let line = r#"#EXTINF:-1 tvg-id="" group-title="Sports",Channel"#;
        assert_eq!(attr_value(line, "tvg-id"), Some(""));
    }

    // ─── b64_encode ───────────────────────────────────────────────────────────

    #[test]
    fn b64_encode_empty_string() {
        assert_eq!(b64_encode(""), "");
    }

    #[test]
    fn b64_encode_single_byte() {
        assert_eq!(b64_encode("M"), "TQ==");
    }

    #[test]
    fn b64_encode_two_bytes() {
        assert_eq!(b64_encode("Ma"), "TWE=");
    }

    #[test]
    fn b64_encode_three_bytes_no_padding() {
        assert_eq!(b64_encode("Man"), "TWFu");
    }

    #[test]
    fn b64_encode_longer_string() {
        assert_eq!(b64_encode("Hello"), "SGVsbG8=");
        assert_eq!(b64_encode("Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    // ─── date_to_unix ─────────────────────────────────────────────────────────

    #[test]
    fn date_to_unix_epoch() {
        assert_eq!(date_to_unix(1970, 1, 1), 0);
    }

    #[test]
    fn date_to_unix_known_date() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        assert_eq!(date_to_unix(2024, 1, 1), 1_704_067_200);
    }

    #[test]
    fn date_to_unix_leap_day() {
        // 2024-02-29 (leap day) = 2024-01-01 + 59 days
        assert_eq!(date_to_unix(2024, 2, 29), 1_704_067_200 + 59 * 86400);
    }

    // ─── parse_xmltv_time ─────────────────────────────────────────────────────

    #[test]
    fn parse_xmltv_time_utc() {
        assert_eq!(parse_xmltv_time("20240101000000 +0000"), 1_704_067_200);
    }

    #[test]
    fn parse_xmltv_time_positive_offset() {
        // 01:00 local with +01:00 = 00:00 UTC
        assert_eq!(parse_xmltv_time("20240101010000 +0100"), 1_704_067_200);
    }

    #[test]
    fn parse_xmltv_time_negative_offset() {
        // Dec 31 2023 23:00 with -01:00 = 2024-01-01 00:00 UTC
        assert_eq!(parse_xmltv_time("20231231230000 -0100"), 1_704_067_200);
    }

    #[test]
    fn parse_xmltv_time_no_timezone_assumes_utc() {
        assert_eq!(parse_xmltv_time("20240101000000"), 1_704_067_200);
    }

    #[test]
    fn parse_xmltv_time_too_short_returns_zero() {
        assert_eq!(parse_xmltv_time("2024"), 0);
        assert_eq!(parse_xmltv_time(""), 0);
    }
}
