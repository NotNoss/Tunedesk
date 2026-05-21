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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(cache::AppCacheState::default())
        .manage(logs::AppLogState::default())
        .setup(|app| {
            let handle = app.handle().clone();

            let level = logs::load_log_level(&handle);
            *handle.state::<logs::AppLogState>().level.lock().unwrap() = level;

            let cache_data = cache::load_cache_from_disk(&handle);
            *handle.state::<cache::AppCacheState>().0.lock().unwrap() = cache_data;

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
            cache::prime_cache,
            cache::clear_cache,
            search::search_all_profiles,
            logs::get_logs,
            logs::clear_logs,
            logs::get_log_level,
            logs::set_log_level,
            update::restart_to_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
