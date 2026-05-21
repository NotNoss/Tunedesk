use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

use crate::logs::{log_debug, log_info};

pub async fn check_for_updates(app: tauri::AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            log_info(&app, "updater", format!("Updater unavailable: {e}"));
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            log_info(&app, "updater", format!("Update available: {version}"));

            let mut downloaded = 0u64;
            let result = update
                .download_and_install(
                    |chunk, total| {
                        downloaded += chunk as u64;
                        log_debug(&app, "updater", format!("Downloaded {downloaded}/{total:?}"));
                    },
                    || log_info(&app, "updater", "Download finished, staging update..."),
                )
                .await;

            match result {
                Ok(_) => {
                    log_info(&app, "updater", "Update staged, waiting for user restart");
                    app.emit("update-ready", &version).ok();
                }
                Err(e) => log_info(&app, "updater", format!("Update failed: {e}")),
            }
        }
        Ok(None) => log_info(&app, "updater", "App is up to date"),
        Err(e) => log_info(&app, "updater", format!("Update check failed: {e}")),
    }
}

#[tauri::command]
pub fn restart_to_update(app: tauri::AppHandle) {
    app.restart();
}
