//! API client hooks.

use gloo_net::http::{Request, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// API errors.
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unauthorized")]
    Unauthorized,
}

/// API client for making authenticated requests.
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    /// Create a new API client.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
        }
    }

    /// Set the authorization token.
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }

    /// Build a request builder with authentication.
    fn build_request(&self, method: &str, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let builder = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            "PATCH" => Request::patch(&url),
            _ => Request::get(&url),
        };

        // Add auth header if token exists
        let builder = if let Some(ref token) = self.token {
            builder.header("Authorization", &format!("Bearer {}", token))
        } else {
            builder
        };

        builder.header("Content-Type", "application/json")
    }

    /// Make a GET request.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let request = self
            .build_request("GET", path)
            .build()
            .map_err(|e| ApiError::Network(e.to_string()))?;

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Make a POST request.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let request = self
            .build_request("POST", path)
            .json(body)
            .map_err(|e| ApiError::Parse(e.to_string()))?;

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Make a PUT request.
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let request = self
            .build_request("PUT", path)
            .json(body)
            .map_err(|e| ApiError::Parse(e.to_string()))?;

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Make a DELETE request.
    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let request = self
            .build_request("DELETE", path)
            .build()
            .map_err(|e| ApiError::Network(e.to_string()))?;

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if response.status() == 401 {
            return Err(ApiError::Unauthorized);
        }

        if !response.ok() {
            let message = response.text().await.unwrap_or_default();
            return Err(ApiError::Http {
                status: response.status(),
                message,
            });
        }

        Ok(())
    }

    /// Handle response and parse JSON.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<T, ApiError> {
        if response.status() == 401 {
            return Err(ApiError::Unauthorized);
        }

        if !response.ok() {
            let message = response.text().await.unwrap_or_default();
            return Err(ApiError::Http {
                status: response.status(),
                message,
            });
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::Parse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let network_err = ApiError::Network("connection refused".to_string());
        assert_eq!(network_err.to_string(), "Network error: connection refused");

        let http_err = ApiError::Http {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert_eq!(http_err.to_string(), "HTTP error 404: Not Found");

        let parse_err = ApiError::Parse("invalid JSON".to_string());
        assert_eq!(parse_err.to_string(), "Parse error: invalid JSON");

        let unauth_err = ApiError::Unauthorized;
        assert_eq!(unauth_err.to_string(), "Unauthorized");
    }

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("https://api.example.com");
        assert_eq!(client.base_url, "https://api.example.com");
        assert_eq!(client.token, None);
    }

    #[test]
    fn test_api_client_with_token() {
        let client = ApiClient::new("https://api.example.com")
            .with_token(Some("my-token".to_string()));

        assert_eq!(client.token, Some("my-token".to_string()));
    }

    #[test]
    fn test_api_client_with_none_token() {
        let client = ApiClient::new("https://api.example.com")
            .with_token(Some("initial".to_string()))
            .with_token(None);

        assert_eq!(client.token, None);
    }

    #[test]
    fn test_api_client_clone() {
        let client = ApiClient::new("https://api.example.com")
            .with_token(Some("token".to_string()));

        let cloned = client.clone();

        assert_eq!(cloned.base_url, client.base_url);
        assert_eq!(cloned.token, client.token);
    }

    #[test]
    fn test_api_error_clone() {
        let err = ApiError::Http {
            status: 500,
            message: "Internal Server Error".to_string(),
        };

        let cloned = err.clone();

        match cloned {
            ApiError::Http { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Internal Server Error");
            }
            _ => panic!("Expected Http error"),
        }
    }
}
