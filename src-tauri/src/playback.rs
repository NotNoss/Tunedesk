use std::path::PathBuf;
use tauri::Manager;

use crate::cache::{progress_path, read_progress_map, write_progress_entry, ProgressEntry};
use crate::logs::{log_debug, log_info};

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
mp.observe_property("time-pos", "number", function(_, val)
    if val and val > 0 then last_pos = val end
end)
mp.observe_property("duration", "number", function(_, val)
    if val and val > 0 then last_dur = val end
end)
mp.register_event("shutdown", function()
    local f = io.open("{pos}", "w")
    if f then
        f:write(string.format('{{"position":%f,"duration":%f}}', last_pos, last_dur))
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

    let window = app.get_webview_window("main");
    if let Some(w) = &window {
        let _ = w.hide();
    }

    let result = run_mpv(app, &key, url, start_pos, &script_path).await;

    if let Some(w) = &window {
        let _ = w.show();
    }

    let _ = std::fs::remove_file(&script_path);
    let _ = std::fs::remove_file(&pos_path);

    result?;

    if let Ok(json) = std::fs::read_to_string(&pos_path) {
        if let Ok(entry) = serde_json::from_str::<ProgressEntry>(&json) {
            log_debug(app, "playback", format!("Saving progress for '{key}': {:.1}s / {:.1}s", entry.position, entry.duration));
            write_progress_entry(app, profile, &key, entry.position, entry.duration);
        }
    }

    Ok(())
}

async fn run_mpv(app: &tauri::AppHandle, key: &str, url: String, start_pos: f64, script_path: &std::path::Path) -> Result<(), String> {
    let script_arg = format!("--script={}", script_path.to_string_lossy());
    let mut args = vec![url, script_arg, "--fs".to_string()];
    if start_pos > 0.0 {
        args.push(format!("--start={:.1}", start_pos));
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
