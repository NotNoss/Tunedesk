use std::sync::Mutex;
use tauri::Manager;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
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

#[derive(serde::Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub module: String,
    pub message: String,
}

pub struct AppLogState {
    pub level: Mutex<LogLevel>,
    pub entries: Mutex<Vec<LogEntry>>,
}

impl Default for AppLogState {
    fn default() -> Self {
        AppLogState {
            level: Mutex::new(LogLevel::Info),
            entries: Mutex::new(Vec::new()),
        }
    }
}

// ─── Persistence ──────────────────────────────────────────────────────────────

fn level_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("log_level.txt"))
}

pub fn load_log_level(app: &tauri::AppHandle) -> LogLevel {
    let Some(path) = level_path(app) else { return LogLevel::Info; };
    if !path.exists() { return LogLevel::Info; }
    std::fs::read_to_string(path)
        .map(|s| LogLevel::from_str(s.trim()))
        .unwrap_or(LogLevel::Info)
}

fn save_log_level(app: &tauri::AppHandle, level: &LogLevel) {
    if let Some(path) = level_path(app) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, level.as_str());
    }
}

// ─── Logging functions ────────────────────────────────────────────────────────

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn log(app: &tauri::AppHandle, level: LogLevel, module: &str, message: String) {
    let state = app.state::<AppLogState>();
    {
        let current = state.level.lock().unwrap();
        if level == LogLevel::Debug && *current != LogLevel::Debug {
            return;
        }
    }
    let entry = LogEntry {
        timestamp: now_ts(),
        level: level.as_str().to_string(),
        module: module.to_string(),
        message,
    };
    let mut entries = state.entries.lock().unwrap();
    entries.push(entry);
    // Cap at 2000 entries; trim oldest 500 when exceeded
    if entries.len() > 2000 {
        entries.drain(0..500);
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
pub fn get_logs(app: tauri::AppHandle) -> Vec<LogEntry> {
    app.state::<AppLogState>().entries.lock().unwrap().clone()
}

#[tauri::command]
pub fn clear_logs(app: tauri::AppHandle) {
    app.state::<AppLogState>().entries.lock().unwrap().clear();
}

#[tauri::command]
pub fn get_log_level(app: tauri::AppHandle) -> String {
    app.state::<AppLogState>().level.lock().unwrap().as_str().to_string()
}

#[tauri::command]
pub fn set_log_level(app: tauri::AppHandle, level: String) -> Result<(), String> {
    let new_level = LogLevel::from_str(&level);
    *app.state::<AppLogState>().level.lock().unwrap() = new_level.clone();
    save_log_level(&app, &new_level);
    Ok(())
}
