use std::path::PathBuf;
use tauri::Manager;

use crate::cache::{progress_path, read_progress_map, write_progress_entry};
use crate::logs::{log_debug, log_info};
use crate::settings;

// Locate the mpv binary. When Tauri bundles a sidecar it places the binary
// next to the app executable, so we check there first and fall back to the
// system PATH so development / non-bundled installs still work.
pub fn mpv_executable() -> PathBuf {
    let binary_name = if cfg!(target_os = "windows") { "mpv.exe" } else { "mpv" };

    // On Linux, only use the bundled binary inside an AppImage. The .deb installs
    // the binary to /usr/bin/mpv which conflicts with the system mpv package, so
    // for .deb we rely on system mpv via PATH instead.
    #[cfg(target_os = "linux")]
    let check_bundled = std::env::var_os("APPIMAGE").is_some();
    #[cfg(not(target_os = "linux"))]
    let check_bundled = true;

    if check_bundled {
        if let Ok(exe) = std::env::current_exe() {
            let candidate = exe.parent().map(|d| d.join(binary_name));
            if let Some(path) = candidate.filter(|p| p.exists()) {
                return path;
            }
        }
    }

    PathBuf::from(binary_name)
}

// Returns a Command for the mpv binary with the environment correctly set up.
// Inside an AppImage, LD_LIBRARY_PATH is overridden to point at bundled libs;
// system mpv needs the original path restored so it can find its own libraries.
pub fn mpv_command() -> std::process::Command {
    let mut cmd = std::process::Command::new(mpv_executable());
    #[cfg(target_os = "linux")]
    if std::env::var_os("APPIMAGE").is_some() {
        let orig = std::env::var("LD_LIBRARY_PATH_ORIG").unwrap_or_default();
        if orig.is_empty() {
            cmd.env_remove("LD_LIBRARY_PATH");
        } else {
            cmd.env("LD_LIBRARY_PATH", orig);
        }
    }
    cmd
}

pub async fn launch_mpv(app: &tauri::AppHandle, url: String, key: String, start_over: bool, profile: &str) -> Result<(), String> {
    let pid = std::process::id();
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("tunedesk_{}.lua", pid));
    let pos_path = temp_dir.join(format!("tunedesk_{}.json", pid));

    // Lua's io.open accepts forward slashes on all platforms, so normalise
    // Windows backslashes so the path is valid inside the Lua string literal.
    let pos_path_lua = pos_path.to_string_lossy().replace('\\', "/");

    let script = format!(
        r#"local last_pos = 0
local last_dur = 0
local last_vol = 100
local last_sub_lang = "off"
local last_audio_lang = ""

mp.observe_property("time-pos", "number", function(_, val)
    if val and val > 0 then last_pos = val end
end)
mp.observe_property("duration", "number", function(_, val)
    if val and val > 0 then last_dur = val end
end)
mp.observe_property("volume", "number", function(_, val)
    if val then last_vol = val end
end)

local function find_lang(track_type, id_str)
    if not id_str or id_str == "no" then return nil end
    local tracks = mp.get_property_native("track-list") or {{}}
    for _, t in ipairs(tracks) do
        if t.type == track_type and tostring(t.id) == id_str then
            return t.lang
        end
    end
    return nil
end

mp.observe_property("sid", "string", function(_, val)
    if not val or val == "no" then
        last_sub_lang = "off"
    else
        last_sub_lang = find_lang("sub", val) or "off"
    end
end)

mp.observe_property("aid", "string", function(_, val)
    if not val or val == "no" then
        last_audio_lang = ""
    else
        last_audio_lang = find_lang("audio", val) or ""
    end
end)

mp.register_event("shutdown", function()
    local f = io.open("{pos}", "w")
    if f then
        f:write(string.format(
            '{{"position":%f,"duration":%f,"volume":%f,"sub_lang":"%s","audio_lang":"%s"}}',
            last_pos, last_dur, last_vol, last_sub_lang, last_audio_lang))
        f:close()
    end
end)"#,
        pos = pos_path_lua
    );

    std::fs::write(&script_path, &script).map_err(|e| {
        let msg = format!("Failed to write mpv script: {e}");
        log_info(app, "playback", &msg);
        msg
    })?;

    if start_over {
        let mut map = read_progress_map(app, profile);
        if map.remove(&key).is_some() {
            if let Ok(path) = progress_path(app, profile) {
                let _ = std::fs::write(path, serde_json::to_string_pretty(&map).unwrap());
            }
        }
    }

    let start_pos = if start_over {
        0.0
    } else {
        read_progress_map(app, profile).get(&key).map(|e| e.position).unwrap_or(0.0)
    };

    log_info(app, "playback", format!("Launching mpv for '{key}' (profile: {profile}, start: {start_pos:.1}s)"));
    log_debug(app, "playback", format!("mpv url: {url}"));

    let result = run_mpv(app, &key, url, start_pos, &script_path).await;

    let _ = std::fs::remove_file(&script_path);

    result?;

    if let Ok(json) = std::fs::read_to_string(&pos_path) {
        if let Ok(data) = serde_json::from_str::<MpvExitData>(&json) {
            log_debug(app, "playback", format!("Saving progress for '{key}': {:.1}s / {:.1}s", data.position, data.duration));
            write_progress_entry(app, profile, &key, data.position, data.duration);

            let user_settings = app.state::<settings::AppSettingsState>().0.lock().unwrap().clone();

            // Log when a subtitle preference wasn't satisfied
            let pref_sub = user_settings.subtitle_lang.as_deref().unwrap_or("");
            if !pref_sub.is_empty() && pref_sub != "off" {
                let actual_sub = data.sub_lang.as_deref().unwrap_or("off");
                if actual_sub == "off" {
                    log_info(app, "playback", format!("Preferred subtitle language '{pref_sub}' was not available, defaulted to off"));
                }
            }

            // Log when an audio preference wasn't satisfied
            let pref_audio = user_settings.audio_lang.as_deref().unwrap_or("");
            if !pref_audio.is_empty() {
                let actual_audio = data.audio_lang.as_deref().unwrap_or("");
                if !actual_audio.is_empty() && actual_audio != pref_audio {
                    log_info(app, "playback", format!("Preferred audio language '{pref_audio}' was not available, defaulted to '{actual_audio}'"));
                }
            }

            settings::update_from_mpv(app, data.volume, data.sub_lang, data.audio_lang);
        }
    }
    let _ = std::fs::remove_file(&pos_path);

    Ok(())
}

#[derive(serde::Deserialize)]
struct MpvExitData {
    position: f64,
    duration: f64,
    #[serde(default)]
    volume: Option<f64>,
    #[serde(default)]
    sub_lang: Option<String>,
    #[serde(default)]
    audio_lang: Option<String>,
}

async fn run_mpv(app: &tauri::AppHandle, key: &str, url: String, start_pos: f64, script_path: &std::path::Path) -> Result<(), String> {
    let script_arg = format!("--script={}", script_path.to_string_lossy());
    let mut args = vec![url, script_arg, "--fs".to_string()];
    if start_pos > 0.0 {
        args.push(format!("--start={:.1}", start_pos));
    }

    let user_settings = app.state::<settings::AppSettingsState>().0.lock().unwrap().clone();

    if let Some(vol) = user_settings.volume {
        args.push(format!("--volume={:.0}", vol));
    }
    match user_settings.subtitle_lang.as_deref() {
        Some("off") => {
            args.push("--sid=no".to_string());
        }
        Some(lang) => {
            args.push(format!("--slang={}", lang));
        }
        None => {}
    }
    if let Some(lang) = &user_settings.audio_lang {
        if !lang.is_empty() {
            args.push(format!("--alang={}", lang));
        }
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        let config_dir = resource_dir.join("mpv-config");
        log_debug(app, "playback", format!("mpv config dir: {} (exists: {})", config_dir.display(), config_dir.exists()));
        if config_dir.exists() {
            args.push(format!("--config-dir={}", config_dir.to_string_lossy()));
        }
    }

    let mpv = mpv_executable();
    log_debug(app, "playback", format!("mpv binary: {}", mpv.display()));

    let mut child = mpv_command()
        .args(&args)
        .spawn()
        .map_err(|e| {
            let msg = format!("Failed to launch mpv ({}): {e}", mpv.display());
            log_info(app, "playback", &msg);
            msg
        })?;

    tauri::async_runtime::spawn_blocking(move || child.wait())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| {
            log_info(app, "playback", format!("mpv exited with error: {e}"));
            e.to_string()
        })?;

    log_info(app, "playback", format!("mpv exited for '{key}'"));
    Ok(())
}
