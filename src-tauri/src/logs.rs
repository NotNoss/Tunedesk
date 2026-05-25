use std::io::Write;
use std::sync::Mutex;
use tauri::Manager;

use crate::settings;

const LOG_MAX_AGE_SECS: u64 = 7 * 24 * 3600;
const LOG_MAX_SIZE_BYTES: u64 = 1024 * 1024; // 1 MB

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Debug,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }
}

pub struct AppLogState {
    pub level: Mutex<LogLevel>,
}

impl Default for AppLogState {
    fn default() -> Self {
        AppLogState {
            level: Mutex::new(LogLevel::Info),
        }
    }
}

// ─── Paths ────────────────────────────────────────────────────────────────────

fn log_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("tunedesk.log"))
}

// ─── File rotation ────────────────────────────────────────────────────────────

pub fn rotate_log_if_needed(app: &tauri::AppHandle) {
    let Some(path) = log_path(app) else { return; };

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    if !path.exists() {
        return;
    }

    let Ok(meta) = std::fs::metadata(&path) else { return; };

    let too_old = meta
        .modified()
        .ok()
        .and_then(|m| std::time::SystemTime::now().duration_since(m).ok())
        .map_or(false, |age| age.as_secs() > LOG_MAX_AGE_SECS);

    if too_old || meta.len() > LOG_MAX_SIZE_BYTES {
        let _ = std::fs::remove_file(&path);
    }
}

// ─── Logging functions ────────────────────────────────────────────────────────

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn format_ts(ts: i64) -> String {
    let s = ts as u64;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let mut days = s / 86400;

    let mut year = 1970u32;
    loop {
        let dy = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366u64 } else { 365 };
        if days < dy { break; }
        days -= dy;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days: [u64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u32;
    for &dim in &month_days {
        if days < dim { break; }
        days -= dim;
        month += 1;
    }

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, days + 1, hour, min, sec)
}

pub fn log(app: &tauri::AppHandle, level: LogLevel, module: &str, message: String) {
    {
        let state = app.state::<AppLogState>();
        let current = state.level.lock().unwrap();
        if level == LogLevel::Debug && *current != LogLevel::Debug {
            return;
        }
    }

    let Some(path) = log_path(app) else { return; };
    let line = format!(
        "{} [{:<5}] [{}] {}\n",
        format_ts(now_ts()),
        level.as_str().to_uppercase(),
        module,
        message
    );

    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
    }
}

pub fn log_info(app: &tauri::AppHandle, module: &str, message: impl Into<String>) {
    log(app, LogLevel::Info, module, message.into());
}

pub fn log_debug(app: &tauri::AppHandle, module: &str, message: impl Into<String>) {
    log(app, LogLevel::Debug, module, message.into());
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn log_event(app: tauri::AppHandle, level: String, module: String, message: String) {
    let log_level = LogLevel::from_str(&level);
    log(&app, log_level, &module, message);
}

#[tauri::command]
pub fn get_log_level(app: tauri::AppHandle) -> String {
    app.state::<AppLogState>().level.lock().unwrap().as_str().to_string()
}

#[tauri::command]
pub fn set_log_level(app: tauri::AppHandle, level: String) -> Result<(), String> {
    let new_level = LogLevel::from_str(&level);
    *app.state::<AppLogState>().level.lock().unwrap() = new_level.clone();
    settings::update_log_level(&app, new_level.as_str());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_from_str_info() {
        assert_eq!(LogLevel::from_str("info"), LogLevel::Info);
    }

    #[test]
    fn log_level_from_str_debug() {
        assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
    }

    #[test]
    fn log_level_from_str_unknown_defaults_to_info() {
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Info);
        assert_eq!(LogLevel::from_str(""), LogLevel::Info);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
    }

    #[test]
    fn log_level_as_str() {
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
    }
}
