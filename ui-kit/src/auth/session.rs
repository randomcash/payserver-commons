//! Session persistence using localStorage.

use super::types::{DeviceId, LoginResponse, SessionId, StoredSession};
use crate::hooks::use_storage::{get_local, remove_local, set_local};

/// Key for storing session data in localStorage.
const SESSION_KEY: &str = "ps_session";

/// Key for storing device ID in localStorage.
/// Device ID persists across sessions to identify the same device.
const DEVICE_ID_KEY: &str = "ps_device_id";

/// Save session data from login response to localStorage.
pub fn save_session(response: &LoginResponse) -> Result<(), String> {
    let stored = StoredSession::from_login_response(response);
    set_local(SESSION_KEY, &stored)?;

    // Also persist device ID separately for reuse in future logins
    set_local(DEVICE_ID_KEY, &response.device_id)?;

    Ok(())
}

/// Load session data from localStorage.
/// Returns None if no session exists or if it's expired.
pub fn load_session() -> Option<StoredSession> {
    let session: StoredSession = get_local(SESSION_KEY)?;

    // Check if session is expired
    if session.is_expired() {
        clear_session();
        return None;
    }

    Some(session)
}

/// Get the session ID from localStorage.
pub fn get_session_id() -> Option<SessionId> {
    load_session().map(|s| s.session_id)
}

/// Get the stored device ID from localStorage.
/// This persists across sessions to identify the same device.
pub fn get_device_id() -> Option<DeviceId> {
    get_local(DEVICE_ID_KEY)
}

/// Clear session data from localStorage.
/// Note: Device ID is preserved for future logins.
pub fn clear_session() {
    remove_local(SESSION_KEY);
}

/// Clear all auth data from localStorage, including device ID.
pub fn clear_all_auth_data() {
    remove_local(SESSION_KEY);
    remove_local(DEVICE_ID_KEY);
}

/// Get a human-readable device name from the user agent.
pub fn get_device_name() -> String {
    if let Some(window) = web_sys::window() {
        if let Ok(navigator) = js_sys::Reflect::get(&window, &"navigator".into()) {
            if let Ok(ua) = js_sys::Reflect::get(&navigator, &"userAgent".into()) {
                if let Some(ua_str) = ua.as_string() {
                    // Simple device detection based on user agent
                    if ua_str.contains("Chrome") && !ua_str.contains("Edge") {
                        return "Chrome Browser".to_string();
                    } else if ua_str.contains("Firefox") {
                        return "Firefox Browser".to_string();
                    } else if ua_str.contains("Safari") && !ua_str.contains("Chrome") {
                        return "Safari Browser".to_string();
                    } else if ua_str.contains("Edge") {
                        return "Edge Browser".to_string();
                    }
                }
            }
        }
    }
    "Web Browser".to_string()
}
