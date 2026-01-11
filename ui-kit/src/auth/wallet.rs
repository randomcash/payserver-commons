//! Ethereum wallet (MetaMask) JavaScript bindings.
//!
//! Provides functions to interact with the Ethereum provider (window.ethereum)
//! for wallet authentication via EIP-191 personal_sign.

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

/// Error type for wallet operations.
#[derive(Debug, Clone)]
pub struct WalletError {
    pub message: String,
    pub code: Option<i32>,
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = self.code {
            write!(f, "Wallet error ({}): {}", code, self.message)
        } else {
            write!(f, "Wallet error: {}", self.message)
        }
    }
}

impl From<JsValue> for WalletError {
    fn from(value: JsValue) -> Self {
        // Try to extract error message and code from JsValue
        let message = if let Some(s) = value.as_string() {
            s
        } else if let Ok(obj) = js_sys::Reflect::get(&value, &"message".into()) {
            obj.as_string().unwrap_or_else(|| "Unknown error".to_string())
        } else {
            format!("{:?}", value)
        };

        let code = js_sys::Reflect::get(&value, &"code".into())
            .ok()
            .and_then(|c| c.as_f64())
            .map(|c| c as i32);

        Self { message, code }
    }
}

/// Check if an Ethereum wallet (e.g., MetaMask) is available.
pub fn is_wallet_available() -> bool {
    if let Some(window) = window() {
        js_sys::Reflect::get(&window, &"ethereum".into())
            .map(|v| !v.is_undefined() && !v.is_null())
            .unwrap_or(false)
    } else {
        false
    }
}

/// Get the Ethereum provider from window.ethereum.
fn get_ethereum() -> Result<JsValue, WalletError> {
    let window = window().ok_or_else(|| WalletError {
        message: "No window object".to_string(),
        code: None,
    })?;

    let ethereum = js_sys::Reflect::get(&window, &"ethereum".into()).map_err(|_| WalletError {
        message: "Failed to access window.ethereum".to_string(),
        code: None,
    })?;

    if ethereum.is_undefined() || ethereum.is_null() {
        return Err(WalletError {
            message: "No Ethereum wallet found. Please install MetaMask or another Web3 wallet."
                .to_string(),
            code: None,
        });
    }

    Ok(ethereum)
}

/// Request wallet connection and return the connected address.
///
/// Prompts the user to connect their wallet if not already connected.
/// Returns the checksummed Ethereum address.
pub async fn connect_wallet() -> Result<String, WalletError> {
    let ethereum = get_ethereum()?;

    // Call eth_requestAccounts to prompt connection
    let request = js_sys::Object::new();
    js_sys::Reflect::set(&request, &"method".into(), &"eth_requestAccounts".into())
        .map_err(|e| WalletError::from(e))?;

    let promise = js_sys::Reflect::get(&ethereum, &"request".into())
        .map_err(|e| WalletError::from(e))?;

    let request_fn = promise
        .dyn_ref::<js_sys::Function>()
        .ok_or_else(|| WalletError {
            message: "ethereum.request is not a function".to_string(),
            code: None,
        })?;

    let result = request_fn
        .call1(&ethereum, &request)
        .map_err(|e| WalletError::from(e))?;

    let promise = js_sys::Promise::from(result);
    let accounts = JsFuture::from(promise).await.map_err(WalletError::from)?;

    // Get the first account from the array
    let accounts_array = js_sys::Array::from(&accounts);
    if accounts_array.length() == 0 {
        return Err(WalletError {
            message: "No accounts returned from wallet".to_string(),
            code: None,
        });
    }

    let address = accounts_array
        .get(0)
        .as_string()
        .ok_or_else(|| WalletError {
            message: "Invalid address format".to_string(),
            code: None,
        })?;

    Ok(address)
}

/// Get the currently connected accounts without prompting.
///
/// Returns an empty vec if no accounts are connected.
pub async fn get_accounts() -> Result<Vec<String>, WalletError> {
    let ethereum = get_ethereum()?;

    let request = js_sys::Object::new();
    js_sys::Reflect::set(&request, &"method".into(), &"eth_accounts".into())
        .map_err(|e| WalletError::from(e))?;

    let request_fn = js_sys::Reflect::get(&ethereum, &"request".into())
        .map_err(|e| WalletError::from(e))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| WalletError {
            message: "ethereum.request is not a function".to_string(),
            code: None,
        })?;

    let result = request_fn
        .call1(&ethereum, &request)
        .map_err(|e| WalletError::from(e))?;

    let promise = js_sys::Promise::from(result);
    let accounts = JsFuture::from(promise).await.map_err(WalletError::from)?;

    let accounts_array = js_sys::Array::from(&accounts);
    let mut result = Vec::new();
    for i in 0..accounts_array.length() {
        if let Some(addr) = accounts_array.get(i).as_string() {
            result.push(addr);
        }
    }

    Ok(result)
}

/// Sign a message using EIP-191 personal_sign.
///
/// Prompts the user to sign the message with their wallet.
/// Returns the signature as a hex string (0x-prefixed, 65 bytes: r + s + v).
pub async fn sign_message(address: &str, message: &str) -> Result<String, WalletError> {
    let ethereum = get_ethereum()?;

    // Build the request for personal_sign
    // personal_sign params: [message, address]
    let params = js_sys::Array::new();
    params.push(&message.into());
    params.push(&address.into());

    let request = js_sys::Object::new();
    js_sys::Reflect::set(&request, &"method".into(), &"personal_sign".into())
        .map_err(|e| WalletError::from(e))?;
    js_sys::Reflect::set(&request, &"params".into(), &params)
        .map_err(|e| WalletError::from(e))?;

    let request_fn = js_sys::Reflect::get(&ethereum, &"request".into())
        .map_err(|e| WalletError::from(e))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| WalletError {
            message: "ethereum.request is not a function".to_string(),
            code: None,
        })?;

    let result = request_fn
        .call1(&ethereum, &request)
        .map_err(|e| WalletError::from(e))?;

    let promise = js_sys::Promise::from(result);
    let signature = JsFuture::from(promise).await.map_err(WalletError::from)?;

    signature.as_string().ok_or_else(|| WalletError {
        message: "Invalid signature format".to_string(),
        code: None,
    })
}

/// Format an address for display (truncated).
/// e.g., "0x1234...5678"
pub fn format_address(address: &str) -> String {
    if address.len() > 10 {
        format!("{}...{}", &address[..6], &address[address.len() - 4..])
    } else {
        address.to_string()
    }
}
