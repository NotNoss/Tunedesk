use std::sync::Mutex;
use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct UserSettings {
    pub volume: Option<f64>,
    // "off" means the user explicitly disabled subtitles; a language code means preferred lang
    pub subtitle_lang: Option<String>,
    // language code for preferred audio track
    pub audio_lang: Option<String>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_maximized: Option<bool>,
    pub theme: Option<String>,
    pub auto_play_next: Option<bool>,
    pub log_level: Option<String>,
}

pub struct AppSettingsState(pub Mutex<UserSettings>);

impl Default for AppSettingsState {
    fn default() -> Self {
        AppSettingsState(Mutex::new(UserSettings::default()))
    }
}

fn settings_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("settings.json"))
}

pub fn load_settings(app: &tauri::AppHandle) -> UserSettings {
    let Some(path) = settings_path(app) else { return UserSettings::default(); };
    if !path.exists() { return UserSettings::default(); }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_to_disk(app: &tauri::AppHandle, settings: &UserSettings) {
    if let Some(path) = settings_path(app) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = std::fs::write(path, json);
        }
    }
}

// Called after MPV exits to persist playback preferences captured by the Lua script
pub fn update_from_mpv(
    app: &tauri::AppHandle,
    volume: Option<f64>,
    sub_lang: Option<String>,
    audio_lang: Option<String>,
) {
    let state = app.state::<AppSettingsState>();
    let mut settings = state.0.lock().unwrap();
    if let Some(v) = volume {
        settings.volume = Some(v);
    }
    if let Some(s) = sub_lang {
        settings.subtitle_lang = Some(s);
    }
    if let Some(a) = audio_lang {
        if !a.is_empty() {
            settings.audio_lang = Some(a);
        }
    }
    save_to_disk(app, &settings);
}

#[tauri::command]
pub fn get_user_settings(app: tauri::AppHandle) -> UserSettings {
    app.state::<AppSettingsState>().0.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_window_size(app: tauri::AppHandle, width: u32, height: u32, maximized: bool) -> Result<(), String> {
    let state = app.state::<AppSettingsState>();
    let mut settings = state.0.lock().unwrap();
    settings.window_maximized = Some(maximized);
    // Only persist the windowed dimensions when not maximized, so we keep the
    // user's preferred non-maximized size even if they close while maximized.
    if !maximized {
        settings.window_width = Some(width);
        settings.window_height = Some(height);
    }
    save_to_disk(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn save_theme(app: tauri::AppHandle, theme: String) -> Result<(), String> {
    let state = app.state::<AppSettingsState>();
    let mut settings = state.0.lock().unwrap();
    settings.theme = Some(theme);
    save_to_disk(&app, &settings);
    Ok(())
}

#[tauri::command]
pub fn save_auto_play_next(app: tauri::AppHandle, value: bool) -> Result<(), String> {
    let state = app.state::<AppSettingsState>();
    let mut settings = state.0.lock().unwrap();
    settings.auto_play_next = Some(value);
    save_to_disk(&app, &settings);
    Ok(())
}

pub fn update_log_level(app: &tauri::AppHandle, level_str: &str) {
    let state = app.state::<AppSettingsState>();
    let mut settings = state.0.lock().unwrap();
    settings.log_level = Some(level_str.to_string());
    save_to_disk(app, &settings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_settings_default_all_none() {
        let s = UserSettings::default();
        assert!(s.volume.is_none());
        assert!(s.subtitle_lang.is_none());
        assert!(s.audio_lang.is_none());
        assert!(s.window_width.is_none());
        assert!(s.window_height.is_none());
        assert!(s.window_maximized.is_none());
        assert!(s.theme.is_none());
        assert!(s.auto_play_next.is_none());
        assert!(s.log_level.is_none());
    }

    #[test]
    fn user_settings_round_trips_through_json() {
        let original = UserSettings {
            volume: Some(0.85),
            subtitle_lang: Some("en".to_string()),
            audio_lang: Some("fr".to_string()),
            window_width: Some(1280),
            window_height: Some(720),
            window_maximized: Some(false),
            theme: Some("dark".to_string()),
            auto_play_next: Some(true),
            log_level: Some("debug".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.volume, original.volume);
        assert_eq!(parsed.subtitle_lang, original.subtitle_lang);
        assert_eq!(parsed.audio_lang, original.audio_lang);
        assert_eq!(parsed.window_width, original.window_width);
        assert_eq!(parsed.window_height, original.window_height);
        assert_eq!(parsed.window_maximized, original.window_maximized);
        assert_eq!(parsed.theme, original.theme);
        assert_eq!(parsed.auto_play_next, original.auto_play_next);
        assert_eq!(parsed.log_level, original.log_level);
    }

    #[test]
    fn user_settings_partial_json_fills_missing_fields_with_none() {
        let json = r#"{"volume": 0.5}"#;
        let s: UserSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.volume, Some(0.5));
        assert!(s.theme.is_none());
        assert!(s.auto_play_next.is_none());
        assert!(s.window_width.is_none());
    }
}
