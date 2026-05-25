// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod live;
mod logs;
mod m3u8;
mod playback;
mod profiles;
mod search;
mod series;
mod settings;
mod update;
mod vods;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
async fn cancel_playback(app: tauri::AppHandle) -> Result<(), String> {
    let pid = {
        let state = app.state::<playback::MpvPidState>();
        let guard = state.0.lock().unwrap();
        *guard
    };
    if let Some(pid) = pid {
        #[cfg(not(target_os = "windows"))]
        { std::process::Command::new("kill").arg(pid.to_string()).output().ok(); }
        #[cfg(target_os = "windows")]
        { std::process::Command::new("taskkill").args(["/F", "/PID", &pid.to_string()]).output().ok(); }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(cache::AppCacheState::default())
        .manage(logs::AppLogState::default())
        .manage(settings::AppSettingsState::default())
        .manage(playback::MpvPidState(std::sync::Mutex::new(None)))
        .setup(|app| {
            let handle = app.handle().clone();

            logs::rotate_log_if_needed(&handle);

            let cache_data = cache::load_cache_from_disk(&handle);
            *handle.state::<cache::AppCacheState>().0.lock().unwrap() = cache_data;

            let user_settings = settings::load_settings(&handle);
            let log_level = user_settings.log_level
                .as_deref()
                .map(logs::LogLevel::from_str)
                .unwrap_or(logs::LogLevel::Info);
            *handle.state::<logs::AppLogState>().level.lock().unwrap() = log_level;
            if let Some(window) = handle.get_webview_window("main") {
                if user_settings.window_maximized == Some(true) {
                    let _ = window.maximize();
                } else if let (Some(w), Some(h)) = (user_settings.window_width, user_settings.window_height) {
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: w, height: h }));
                }
            }
            *handle.state::<settings::AppSettingsState>().0.lock().unwrap() = user_settings;

            logs::log_info(&handle, "app", "Application started");

            let update_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                update::check_for_updates(update_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            exit_app,
            profiles::save_xtream_profile,
            profiles::save_m3u8_profile,
            profiles::get_xtream_profiles,
            profiles::get_m3u8_profiles,
            profiles::get_xtream_profile,
            profiles::get_m3u8_profile,
            profiles::delete_profile,
            profiles::get_profile_icons,
            profiles::set_profile_icon,
            live::get_live_categories,
            live::get_live_streams,
            live::get_channel_epg,
            live::get_all_epg,
            live::play_live,
            vods::get_vod_categories,
            vods::get_vod_streams,
            vods::get_vod_info,
            series::get_series_categories,
            series::get_series_items,
            series::get_series_info,
            vods::play_vod,
            series::play_episode,
            cache::get_progress,
            cache::get_watched,
            cache::set_watched,
            cache::set_unwatched,
            cache::prime_cache,
            cache::clear_cache,
            search::search_all_profiles,
            logs::log_event,
            logs::get_log_level,
            logs::set_log_level,
            settings::get_user_settings,
            settings::save_window_size,
            settings::save_theme,
            settings::save_auto_play_next,
            update::restart_to_update,
            cancel_playback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
