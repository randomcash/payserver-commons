//! Local storage utilities.

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

/// Get a value from local storage.
pub fn get_local<T: DeserializeOwned>(key: &str) -> Option<T> {
    LocalStorage::get(key).ok()
}

/// Set a value in local storage.
pub fn set_local<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    LocalStorage::set(key, value).map_err(|e| e.to_string())
}

/// Remove a value from local storage.
pub fn remove_local(key: &str) {
    LocalStorage::delete(key);
}

/// Clear all local storage.
pub fn clear_local() {
    LocalStorage::clear();
}

/// Get a value from session storage.
pub fn get_session<T: DeserializeOwned>(key: &str) -> Option<T> {
    SessionStorage::get(key).ok()
}

/// Set a value in session storage.
pub fn set_session<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    SessionStorage::set(key, value).map_err(|e| e.to_string())
}

/// Remove a value from session storage.
pub fn remove_session(key: &str) {
    SessionStorage::delete(key);
}

/// Clear all session storage.
pub fn clear_session() {
    SessionStorage::clear();
}
