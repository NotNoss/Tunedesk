use std::collections::HashMap;
use tauri::Manager;
use url::Url;

use crate::logs::{log_debug, log_info};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProfileCredentials {
    pub url: String,
    pub username: String,
    pub password: String,
}

pub type CredentialsMap = HashMap<String, ProfileCredentials>;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct M3u8Credentials {
    pub m3u_url: String,
    pub epg_url: String,
}

pub type M3u8CredentialsMap = HashMap<String, M3u8Credentials>;

// ─── Storage helpers ──────────────────────────────────────────────────────────

fn write_json_secret(path: &std::path::Path, json: &str) -> Result<(), String> {
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn credentials_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("credentials.json"))
}

pub fn read_credentials(app: &tauri::AppHandle) -> Result<CredentialsMap, String> {
    let path = credentials_path(app)?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    log_debug(app, "profiles", format!("Reading credentials from {}", path.display()));
    std::fs::read_to_string(&path)
        .map_err(|e| {
            log_info(app, "profiles", format!("Failed to read credentials: {e}"));
            e.to_string()
        })
        .and_then(|json| serde_json::from_str(&json).map_err(|e| {
            log_info(app, "profiles", format!("Failed to parse credentials: {e}"));
            e.to_string()
        }))
}

fn write_credentials(app: &tauri::AppHandle, creds: &CredentialsMap) -> Result<(), String> {
    let path = credentials_path(app)?;
    log_debug(app, "profiles", format!("Writing credentials to {}", path.display()));
    write_json_secret(&path, &serde_json::to_string_pretty(creds).unwrap())
        .map_err(|e| {
            log_info(app, "profiles", format!("Failed to write credentials: {e}"));
            e
        })
}

fn m3u8_credentials_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("m3u8_credentials.json"))
}

pub fn read_m3u8_credentials(app: &tauri::AppHandle) -> Result<M3u8CredentialsMap, String> {
    let path = m3u8_credentials_path(app)?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    log_debug(app, "profiles", format!("Reading M3U8 credentials from {}", path.display()));
    std::fs::read_to_string(&path)
        .map_err(|e| {
            log_info(app, "profiles", format!("Failed to read M3U8 credentials: {e}"));
            e.to_string()
        })
        .and_then(|json| serde_json::from_str(&json).map_err(|e| {
            log_info(app, "profiles", format!("Failed to parse M3U8 credentials: {e}"));
            e.to_string()
        }))
}

fn write_m3u8_credentials(app: &tauri::AppHandle, creds: &M3u8CredentialsMap) -> Result<(), String> {
    let path = m3u8_credentials_path(app)?;
    log_debug(app, "profiles", format!("Writing M3U8 credentials to {}", path.display()));
    write_json_secret(&path, &serde_json::to_string_pretty(creds).unwrap())
        .map_err(|e| {
            log_info(app, "profiles", format!("Failed to write M3U8 credentials: {e}"));
            e
        })
}

pub fn get_profile_list(app: &tauri::AppHandle) -> Vec<String> {
    let Ok(data_dir) = app.path().app_data_dir() else { return vec![]; };
    match std::fs::read_to_string(data_dir.join("profiles.json")) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn save_profile_list(app: &tauri::AppHandle, names: &[String]) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    std::fs::write(
        data_dir.join("profiles.json"),
        serde_json::to_string(names).unwrap(),
    )
    .map_err(|e| {
        log_info(app, "profiles", format!("Failed to save profile list: {e}"));
        e.to_string()
    })
}

fn extract_base_url(raw: &str) -> Result<String, String> {
    let parsed = Url::parse(raw).map_err(|e| format!("Invalid URL: {}", e))?;
    let scheme = parsed.scheme();
    let host = parsed.host_str().ok_or("URL has no host")?;
    match parsed.port() {
        Some(port) => Ok(format!("{}://{}:{}", scheme, host, port)),
        None => Ok(format!("{}://{}", scheme, host)),
    }
}

pub fn is_m3u8_profile(app: &tauri::AppHandle, name: &str) -> bool {
    read_m3u8_credentials(app)
        .map(|m| m.contains_key(name))
        .unwrap_or(false)
}

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_xtream_profiles(app: tauri::AppHandle) -> Vec<String> {
    get_profile_list(&app)
}

#[tauri::command]
pub fn get_m3u8_profiles(app: tauri::AppHandle) -> Vec<String> {
    read_m3u8_credentials(&app)
        .map(|m| m.into_keys().collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_xtream_profile(
    app: tauri::AppHandle,
    name: String,
    url: String,
    username: String,
    password: String,
) -> Result<(), String> {
    log_info(&app, "profiles", format!("Saving Xtream profile '{name}'"));
    let base_url = extract_base_url(&url).map_err(|e| {
        log_info(&app, "profiles", format!("Invalid URL for profile '{name}': {e}"));
        e
    })?;

    let mut creds = read_credentials(&app)?;
    creds.insert(name.clone(), ProfileCredentials { url: base_url, username, password });
    write_credentials(&app, &creds)?;

    let mut profiles = get_profile_list(&app);
    if !profiles.contains(&name) {
        profiles.push(name.clone());
        save_profile_list(&app, &profiles)?;
    }

    log_info(&app, "profiles", format!("Xtream profile '{name}' saved"));
    Ok(())
}

#[tauri::command]
pub fn save_m3u8_profile(
    app: tauri::AppHandle,
    name: String,
    m3u_url: String,
    epg_url: String,
) -> Result<(), String> {
    log_info(&app, "profiles", format!("Saving M3U8 profile '{name}'"));
    let mut creds = read_m3u8_credentials(&app)?;
    creds.insert(name.clone(), M3u8Credentials { m3u_url, epg_url });
    write_m3u8_credentials(&app, &creds)?;

    let mut profiles = get_profile_list(&app);
    if !profiles.contains(&name) {
        profiles.push(name.clone());
        save_profile_list(&app, &profiles)?;
    }

    log_info(&app, "profiles", format!("M3U8 profile '{name}' saved"));
    Ok(())
}

#[tauri::command]
pub fn get_xtream_profile(app: tauri::AppHandle, name: String) -> Result<ProfileCredentials, String> {
    let creds = read_credentials(&app)?;
    creds.get(&name).cloned().ok_or_else(|| {
        let msg = format!("Profile '{name}' not found");
        log_info(&app, "profiles", &msg);
        msg
    })
}

#[tauri::command]
pub fn get_m3u8_profile(app: tauri::AppHandle, name: String) -> Result<M3u8Credentials, String> {
    let creds = read_m3u8_credentials(&app)?;
    creds.get(&name).cloned().ok_or_else(|| {
        let msg = format!("M3U8 profile '{name}' not found");
        log_info(&app, "profiles", &msg);
        msg
    })
}

#[tauri::command]
pub fn delete_profile(app: tauri::AppHandle, name: String) -> Result<(), String> {
    log_info(&app, "profiles", format!("Deleting profile '{name}'"));

    let mut creds = read_credentials(&app)?;
    creds.remove(&name);
    write_credentials(&app, &creds)?;

    let mut m3u8_creds = read_m3u8_credentials(&app)?;
    m3u8_creds.remove(&name);
    write_m3u8_credentials(&app, &m3u8_creds)?;

    let mut profiles = get_profile_list(&app);
    profiles.retain(|p| p != &name);
    save_profile_list(&app, &profiles)?;

    let mut icons = read_profile_icons(&app);
    icons.remove(&name);
    let _ = write_profile_icons(&app, &icons);

    log_info(&app, "profiles", format!("Profile '{name}' deleted"));
    Ok(())
}

// ─── Profile icons ────────────────────────────────────────────────────────────

fn profile_icons_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("profile_icons.json"))
}

fn read_profile_icons(app: &tauri::AppHandle) -> HashMap<String, String> {
    let Ok(path) = profile_icons_path(app) else { return HashMap::new(); };
    if !path.exists() { return HashMap::new(); }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_profile_icons(app: &tauri::AppHandle, icons: &HashMap<String, String>) -> Result<(), String> {
    let path = profile_icons_path(app)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(path, serde_json::to_string(icons).unwrap()).map_err(|e| {
        log_info(app, "profiles", format!("Failed to write profile icons: {e}"));
        e.to_string()
    })
}

#[tauri::command]
pub fn get_profile_icons(app: tauri::AppHandle) -> HashMap<String, String> {
    read_profile_icons(&app)
}

#[tauri::command]
pub fn set_profile_icon(app: tauri::AppHandle, name: String, icon: String) -> Result<(), String> {
    log_debug(&app, "profiles", format!("Setting icon for profile '{name}'"));
    let mut icons = read_profile_icons(&app);
    icons.insert(name, icon);
    write_profile_icons(&app, &icons)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_base_url_with_port() {
        assert_eq!(
            extract_base_url("http://example.com:8080/player_api.php?username=u&password=p"),
            Ok("http://example.com:8080".to_string())
        );
    }

    #[test]
    fn extract_base_url_without_port() {
        assert_eq!(
            extract_base_url("https://example.com/player_api.php"),
            Ok("https://example.com".to_string())
        );
    }

    #[test]
    fn extract_base_url_strips_path_and_query() {
        assert_eq!(
            extract_base_url("http://tv.example.com:1234/get.php?username=u&password=p&type=m3u_plus"),
            Ok("http://tv.example.com:1234".to_string())
        );
    }

    #[test]
    fn extract_base_url_invalid_returns_err() {
        assert!(extract_base_url("not-a-url").is_err());
        assert!(extract_base_url("").is_err());
    }

    #[test]
    fn extract_base_url_default_port_is_stripped() {
        // The url crate normalises default ports (80 for http) to None,
        // so the result omits the port number.
        assert_eq!(
            extract_base_url("http://tv.example.com:80"),
            Ok("http://tv.example.com".to_string())
        );
    }
}
