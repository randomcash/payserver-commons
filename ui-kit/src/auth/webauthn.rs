//! WebAuthn/Passkey JavaScript bindings.
//!
//! Provides functions to interact with the Web Credential API for
//! passkey (WebAuthn) authentication.
//!
//! The flow is:
//! 1. Server sends challenge options as JSON
//! 2. Client converts JSON to Web Credential API format
//! 3. Client calls navigator.credentials.create/get
//! 4. Client converts response back to JSON for server

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

/// Error type for WebAuthn operations.
#[derive(Debug, Clone)]
pub struct WebAuthnError {
    pub message: String,
    pub name: Option<String>,
}

impl std::fmt::Display for WebAuthnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "WebAuthn error ({}): {}", name, self.message)
        } else {
            write!(f, "WebAuthn error: {}", self.message)
        }
    }
}

impl From<JsValue> for WebAuthnError {
    fn from(value: JsValue) -> Self {
        let message = if let Some(s) = value.as_string() {
            s
        } else if let Ok(obj) = js_sys::Reflect::get(&value, &"message".into()) {
            obj.as_string().unwrap_or_else(|| "Unknown error".to_string())
        } else {
            format!("{:?}", value)
        };

        let name = js_sys::Reflect::get(&value, &"name".into())
            .ok()
            .and_then(|n| n.as_string());

        Self { message, name }
    }
}

/// Check if WebAuthn is available in the browser.
pub fn is_webauthn_available() -> bool {
    if let Some(window) = window() {
        if let Ok(navigator) = js_sys::Reflect::get(&window, &"navigator".into()) {
            if let Ok(credentials) = js_sys::Reflect::get(&navigator, &"credentials".into()) {
                return !credentials.is_undefined() && !credentials.is_null();
            }
        }
    }
    false
}

/// Check if the platform supports passkeys (platform authenticator).
pub async fn is_platform_authenticator_available() -> bool {
    if !is_webauthn_available() {
        return false;
    }

    // Check PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable()
    let window = match window() {
        Some(w) => w,
        None => return false,
    };

    let pkc = match js_sys::Reflect::get(&window, &"PublicKeyCredential".into()) {
        Ok(p) if !p.is_undefined() => p,
        _ => return false,
    };

    let check_fn =
        match js_sys::Reflect::get(&pkc, &"isUserVerifyingPlatformAuthenticatorAvailable".into()) {
            Ok(f) if f.is_function() => f.dyn_into::<js_sys::Function>().ok(),
            _ => return false,
        };

    if let Some(func) = check_fn {
        if let Ok(result) = func.call0(&pkc) {
            if let Ok(promise) = result.dyn_into::<js_sys::Promise>() {
                if let Ok(value) = JsFuture::from(promise).await {
                    return value.as_bool().unwrap_or(false);
                }
            }
        }
    }

    false
}

/// Create a new credential for registration.
///
/// Takes the server's creation options (from webauthn-rs CreationChallengeResponse)
/// and returns the credential response as JSON.
///
/// The options should be the JSON-serialized CreationChallengeResponse from the server.
pub async fn create_credential(options_json: &serde_json::Value) -> Result<serde_json::Value, WebAuthnError> {
    let window = window().ok_or_else(|| WebAuthnError {
        message: "No window object".to_string(),
        name: None,
    })?;

    let navigator = js_sys::Reflect::get(&window, &"navigator".into())
        .map_err(|e| WebAuthnError::from(e))?;

    let credentials = js_sys::Reflect::get(&navigator, &"credentials".into())
        .map_err(|e| WebAuthnError::from(e))?;

    // Convert the server options to the format expected by the Web Credential API
    let public_key = convert_creation_options_to_js(options_json)?;

    // Build the CredentialCreationOptions
    let create_options = js_sys::Object::new();
    js_sys::Reflect::set(&create_options, &"publicKey".into(), &public_key)
        .map_err(|e| WebAuthnError::from(e))?;

    // Call navigator.credentials.create()
    let create_fn = js_sys::Reflect::get(&credentials, &"create".into())
        .map_err(|e| WebAuthnError::from(e))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| WebAuthnError {
            message: "credentials.create is not a function".to_string(),
            name: None,
        })?;

    let result = create_fn
        .call1(&credentials, &create_options)
        .map_err(|e| WebAuthnError::from(e))?;

    let promise = result.dyn_into::<js_sys::Promise>().map_err(|_| WebAuthnError {
        message: "credentials.create did not return a Promise".to_string(),
        name: None,
    })?;

    let credential = JsFuture::from(promise).await.map_err(WebAuthnError::from)?;

    // Convert the credential response to JSON
    convert_registration_response_to_json(&credential)
}

/// Get a credential for authentication (login).
///
/// Takes the server's request options (from webauthn-rs RequestChallengeResponse)
/// and returns the credential response as JSON.
pub async fn get_credential(options_json: &serde_json::Value) -> Result<serde_json::Value, WebAuthnError> {
    let window = window().ok_or_else(|| WebAuthnError {
        message: "No window object".to_string(),
        name: None,
    })?;

    let navigator = js_sys::Reflect::get(&window, &"navigator".into())
        .map_err(|e| WebAuthnError::from(e))?;

    let credentials = js_sys::Reflect::get(&navigator, &"credentials".into())
        .map_err(|e| WebAuthnError::from(e))?;

    // Convert the server options to the format expected by the Web Credential API
    let public_key = convert_request_options_to_js(options_json)?;

    // Build the CredentialRequestOptions
    let get_options = js_sys::Object::new();
    js_sys::Reflect::set(&get_options, &"publicKey".into(), &public_key)
        .map_err(|e| WebAuthnError::from(e))?;

    // Call navigator.credentials.get()
    let get_fn = js_sys::Reflect::get(&credentials, &"get".into())
        .map_err(|e| WebAuthnError::from(e))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| WebAuthnError {
            message: "credentials.get is not a function".to_string(),
            name: None,
        })?;

    let result = get_fn
        .call1(&credentials, &get_options)
        .map_err(|e| WebAuthnError::from(e))?;

    let promise = result.dyn_into::<js_sys::Promise>().map_err(|_| WebAuthnError {
        message: "credentials.get did not return a Promise".to_string(),
        name: None,
    })?;

    let credential = JsFuture::from(promise).await.map_err(WebAuthnError::from)?;

    // Convert the credential response to JSON
    convert_authentication_response_to_json(&credential)
}

// ============================================================================
// Helper functions for converting between JSON and Web Credential API types
// ============================================================================

/// Convert base64url string to Uint8Array.
fn base64url_to_uint8array(base64url: &str) -> Result<js_sys::Uint8Array, WebAuthnError> {
    // Convert base64url to standard base64
    let base64 = base64url.replace('-', "+").replace('_', "/");

    // Add padding if needed
    let padded = match base64.len() % 4 {
        2 => format!("{}==", base64),
        3 => format!("{}=", base64),
        _ => base64,
    };

    // Decode base64
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &padded)
        .map_err(|e| WebAuthnError {
            message: format!("Failed to decode base64: {}", e),
            name: None,
        })?;

    Ok(js_sys::Uint8Array::from(bytes.as_slice()))
}

/// Convert Uint8Array to base64url string.
fn uint8array_to_base64url(array: &js_sys::Uint8Array) -> String {
    let bytes = array.to_vec();
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    // Convert to base64url
    base64
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

/// Convert ArrayBuffer to base64url string.
fn arraybuffer_to_base64url(buffer: &JsValue) -> Result<String, WebAuthnError> {
    let array = js_sys::Uint8Array::new(buffer);
    Ok(uint8array_to_base64url(&array))
}

/// Convert server creation options to Web Credential API format.
fn convert_creation_options_to_js(options: &serde_json::Value) -> Result<JsValue, WebAuthnError> {
    let public_key = options.get("publicKey").unwrap_or(options);

    let js_options = js_sys::Object::new();

    // challenge (base64url -> Uint8Array)
    if let Some(challenge) = public_key.get("challenge").and_then(|c| c.as_str()) {
        let challenge_bytes = base64url_to_uint8array(challenge)?;
        js_sys::Reflect::set(&js_options, &"challenge".into(), &challenge_bytes)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // rp (relying party)
    if let Some(rp) = public_key.get("rp") {
        let js_rp = js_sys::Object::new();
        if let Some(name) = rp.get("name").and_then(|n| n.as_str()) {
            js_sys::Reflect::set(&js_rp, &"name".into(), &name.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        if let Some(id) = rp.get("id").and_then(|i| i.as_str()) {
            js_sys::Reflect::set(&js_rp, &"id".into(), &id.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        js_sys::Reflect::set(&js_options, &"rp".into(), &js_rp)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // user
    if let Some(user) = public_key.get("user") {
        let js_user = js_sys::Object::new();
        if let Some(id) = user.get("id").and_then(|i| i.as_str()) {
            let id_bytes = base64url_to_uint8array(id)?;
            js_sys::Reflect::set(&js_user, &"id".into(), &id_bytes)
                .map_err(|e| WebAuthnError::from(e))?;
        }
        if let Some(name) = user.get("name").and_then(|n| n.as_str()) {
            js_sys::Reflect::set(&js_user, &"name".into(), &name.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        if let Some(display_name) = user.get("displayName").and_then(|n| n.as_str()) {
            js_sys::Reflect::set(&js_user, &"displayName".into(), &display_name.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        js_sys::Reflect::set(&js_options, &"user".into(), &js_user)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // pubKeyCredParams
    if let Some(params) = public_key.get("pubKeyCredParams").and_then(|p| p.as_array()) {
        let js_params = js_sys::Array::new();
        for param in params {
            let js_param = js_sys::Object::new();
            if let Some(alg) = param.get("alg").and_then(|a| a.as_i64()) {
                js_sys::Reflect::set(&js_param, &"alg".into(), &JsValue::from(alg as f64))
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            if let Some(type_) = param.get("type").and_then(|t| t.as_str()) {
                js_sys::Reflect::set(&js_param, &"type".into(), &type_.into())
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            js_params.push(&js_param);
        }
        js_sys::Reflect::set(&js_options, &"pubKeyCredParams".into(), &js_params)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // timeout
    if let Some(timeout) = public_key.get("timeout").and_then(|t| t.as_u64()) {
        js_sys::Reflect::set(&js_options, &"timeout".into(), &JsValue::from(timeout as f64))
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // authenticatorSelection
    if let Some(auth_sel) = public_key.get("authenticatorSelection") {
        let js_auth_sel = js_sys::Object::new();
        if let Some(attachment) = auth_sel.get("authenticatorAttachment").and_then(|a| a.as_str()) {
            js_sys::Reflect::set(&js_auth_sel, &"authenticatorAttachment".into(), &attachment.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        if let Some(resident) = auth_sel.get("residentKey").and_then(|r| r.as_str()) {
            js_sys::Reflect::set(&js_auth_sel, &"residentKey".into(), &resident.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        if let Some(uv) = auth_sel.get("userVerification").and_then(|u| u.as_str()) {
            js_sys::Reflect::set(&js_auth_sel, &"userVerification".into(), &uv.into())
                .map_err(|e| WebAuthnError::from(e))?;
        }
        js_sys::Reflect::set(&js_options, &"authenticatorSelection".into(), &js_auth_sel)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // attestation
    if let Some(attestation) = public_key.get("attestation").and_then(|a| a.as_str()) {
        js_sys::Reflect::set(&js_options, &"attestation".into(), &attestation.into())
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // excludeCredentials - important for preventing duplicate registrations
    if let Some(exclude) = public_key.get("excludeCredentials").and_then(|e| e.as_array()) {
        let js_exclude = js_sys::Array::new();
        for cred in exclude {
            let js_cred = js_sys::Object::new();
            if let Some(id) = cred.get("id").and_then(|i| i.as_str()) {
                let id_bytes = base64url_to_uint8array(id)?;
                js_sys::Reflect::set(&js_cred, &"id".into(), &id_bytes)
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            if let Some(type_) = cred.get("type").and_then(|t| t.as_str()) {
                js_sys::Reflect::set(&js_cred, &"type".into(), &type_.into())
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            if let Some(transports) = cred.get("transports").and_then(|t| t.as_array()) {
                let js_transports = js_sys::Array::new();
                for transport in transports {
                    if let Some(t) = transport.as_str() {
                        js_transports.push(&t.into());
                    }
                }
                js_sys::Reflect::set(&js_cred, &"transports".into(), &js_transports)
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            js_exclude.push(&js_cred);
        }
        js_sys::Reflect::set(&js_options, &"excludeCredentials".into(), &js_exclude)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    Ok(js_options.into())
}

/// Convert server request options to Web Credential API format.
fn convert_request_options_to_js(options: &serde_json::Value) -> Result<JsValue, WebAuthnError> {
    let public_key = options.get("publicKey").unwrap_or(options);

    let js_options = js_sys::Object::new();

    // challenge (base64url -> Uint8Array)
    if let Some(challenge) = public_key.get("challenge").and_then(|c| c.as_str()) {
        let challenge_bytes = base64url_to_uint8array(challenge)?;
        js_sys::Reflect::set(&js_options, &"challenge".into(), &challenge_bytes)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // timeout
    if let Some(timeout) = public_key.get("timeout").and_then(|t| t.as_u64()) {
        js_sys::Reflect::set(&js_options, &"timeout".into(), &JsValue::from(timeout as f64))
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // rpId
    if let Some(rp_id) = public_key.get("rpId").and_then(|r| r.as_str()) {
        js_sys::Reflect::set(&js_options, &"rpId".into(), &rp_id.into())
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // allowCredentials
    if let Some(allow) = public_key.get("allowCredentials").and_then(|a| a.as_array()) {
        let js_allow = js_sys::Array::new();
        for cred in allow {
            let js_cred = js_sys::Object::new();
            if let Some(id) = cred.get("id").and_then(|i| i.as_str()) {
                let id_bytes = base64url_to_uint8array(id)?;
                js_sys::Reflect::set(&js_cred, &"id".into(), &id_bytes)
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            if let Some(type_) = cred.get("type").and_then(|t| t.as_str()) {
                js_sys::Reflect::set(&js_cred, &"type".into(), &type_.into())
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            if let Some(transports) = cred.get("transports").and_then(|t| t.as_array()) {
                let js_transports = js_sys::Array::new();
                for transport in transports {
                    if let Some(t) = transport.as_str() {
                        js_transports.push(&t.into());
                    }
                }
                js_sys::Reflect::set(&js_cred, &"transports".into(), &js_transports)
                    .map_err(|e| WebAuthnError::from(e))?;
            }
            js_allow.push(&js_cred);
        }
        js_sys::Reflect::set(&js_options, &"allowCredentials".into(), &js_allow)
            .map_err(|e| WebAuthnError::from(e))?;
    }

    // userVerification
    if let Some(uv) = public_key.get("userVerification").and_then(|u| u.as_str()) {
        js_sys::Reflect::set(&js_options, &"userVerification".into(), &uv.into())
            .map_err(|e| WebAuthnError::from(e))?;
    }

    Ok(js_options.into())
}

/// Convert registration credential response to JSON for server.
fn convert_registration_response_to_json(credential: &JsValue) -> Result<serde_json::Value, WebAuthnError> {
    let id = js_sys::Reflect::get(credential, &"id".into())
        .map_err(|e| WebAuthnError::from(e))?
        .as_string()
        .ok_or_else(|| WebAuthnError {
            message: "Missing credential id".to_string(),
            name: None,
        })?;

    let raw_id = js_sys::Reflect::get(credential, &"rawId".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let raw_id_b64 = arraybuffer_to_base64url(&raw_id)?;

    let type_ = js_sys::Reflect::get(credential, &"type".into())
        .map_err(|e| WebAuthnError::from(e))?
        .as_string()
        .unwrap_or_else(|| "public-key".to_string());

    let response = js_sys::Reflect::get(credential, &"response".into())
        .map_err(|e| WebAuthnError::from(e))?;

    let attestation_object = js_sys::Reflect::get(&response, &"attestationObject".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let attestation_object_b64 = arraybuffer_to_base64url(&attestation_object)?;

    let client_data_json = js_sys::Reflect::get(&response, &"clientDataJSON".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let client_data_json_b64 = arraybuffer_to_base64url(&client_data_json)?;

    // Optional: get transports if available
    let transports = if let Ok(get_transports) = js_sys::Reflect::get(&response, &"getTransports".into()) {
        if get_transports.is_function() {
            let func = get_transports.dyn_into::<js_sys::Function>().ok();
            if let Some(f) = func {
                if let Ok(result) = f.call0(&response) {
                    let arr = js_sys::Array::from(&result);
                    let mut transports_vec = Vec::new();
                    for i in 0..arr.length() {
                        if let Some(t) = arr.get(i).as_string() {
                            transports_vec.push(serde_json::Value::String(t));
                        }
                    }
                    Some(serde_json::Value::Array(transports_vec))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut response_obj = serde_json::json!({
        "attestationObject": attestation_object_b64,
        "clientDataJSON": client_data_json_b64,
    });

    if let Some(t) = transports {
        response_obj.as_object_mut().unwrap().insert("transports".to_string(), t);
    }

    Ok(serde_json::json!({
        "id": id,
        "rawId": raw_id_b64,
        "type": type_,
        "response": response_obj,
    }))
}

/// Convert authentication credential response to JSON for server.
fn convert_authentication_response_to_json(credential: &JsValue) -> Result<serde_json::Value, WebAuthnError> {
    let id = js_sys::Reflect::get(credential, &"id".into())
        .map_err(|e| WebAuthnError::from(e))?
        .as_string()
        .ok_or_else(|| WebAuthnError {
            message: "Missing credential id".to_string(),
            name: None,
        })?;

    let raw_id = js_sys::Reflect::get(credential, &"rawId".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let raw_id_b64 = arraybuffer_to_base64url(&raw_id)?;

    let type_ = js_sys::Reflect::get(credential, &"type".into())
        .map_err(|e| WebAuthnError::from(e))?
        .as_string()
        .unwrap_or_else(|| "public-key".to_string());

    let response = js_sys::Reflect::get(credential, &"response".into())
        .map_err(|e| WebAuthnError::from(e))?;

    let authenticator_data = js_sys::Reflect::get(&response, &"authenticatorData".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let authenticator_data_b64 = arraybuffer_to_base64url(&authenticator_data)?;

    let client_data_json = js_sys::Reflect::get(&response, &"clientDataJSON".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let client_data_json_b64 = arraybuffer_to_base64url(&client_data_json)?;

    let signature = js_sys::Reflect::get(&response, &"signature".into())
        .map_err(|e| WebAuthnError::from(e))?;
    let signature_b64 = arraybuffer_to_base64url(&signature)?;

    // userHandle is optional
    let user_handle = js_sys::Reflect::get(&response, &"userHandle".into()).ok();
    let user_handle_b64 = if let Some(uh) = user_handle {
        if !uh.is_null() && !uh.is_undefined() {
            Some(arraybuffer_to_base64url(&uh)?)
        } else {
            None
        }
    } else {
        None
    };

    let mut response_obj = serde_json::json!({
        "authenticatorData": authenticator_data_b64,
        "clientDataJSON": client_data_json_b64,
        "signature": signature_b64,
    });

    if let Some(uh) = user_handle_b64 {
        response_obj.as_object_mut().unwrap().insert("userHandle".to_string(), serde_json::Value::String(uh));
    }

    Ok(serde_json::json!({
        "id": id,
        "rawId": raw_id_b64,
        "type": type_,
        "response": response_obj,
    }))
}
