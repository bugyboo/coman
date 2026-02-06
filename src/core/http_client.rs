//! HTTP Client - Core HTTP request functionality
//!
//! This module provides a clean, library-friendly HTTP client API
//! without any CLI dependencies (no progress bars, colored output, etc.)

use crate::core::errors::HttpError;
use crate::core::http_request::HttpRequest;
use crate::core::http_response::HttpResponse;
use crate::CollectionManager;
use std::time::Duration;

/// HTTP methods supported by the client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

impl From<crate::models::collection::Method> for HttpMethod {
    fn from(method: crate::models::collection::Method) -> Self {
        match method {
            crate::models::collection::Method::Get => HttpMethod::Get,
            crate::models::collection::Method::Post => HttpMethod::Post,
            crate::models::collection::Method::Put => HttpMethod::Put,
            crate::models::collection::Method::Delete => HttpMethod::Delete,
            crate::models::collection::Method::Patch => HttpMethod::Patch,
        }
    }
}

/// Result type for HTTP operations
pub type HttpResult<T> = Result<T, HttpError>;

/// HTTP Client with convenience methods
#[derive(Debug, Clone, Default)]
pub struct HttpClient {
    default_headers: Vec<(String, String)>,
    timeout: Option<Duration>,
    follow_redirects: bool,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default headers for all requests
    pub fn with_default_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.default_headers = headers;
        self
    }

    /// Set default timeout for all requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enable following redirects by default
    pub fn with_follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Create a GET request
    pub fn get(&self, url: &str) -> HttpRequest {
        self.request(HttpMethod::Get, url)
    }

    /// Create a POST request
    pub fn post(&self, url: &str) -> HttpRequest {
        self.request(HttpMethod::Post, url)
    }

    /// Create a PUT request
    pub fn put(&self, url: &str) -> HttpRequest {
        self.request(HttpMethod::Put, url)
    }

    /// Create a DELETE request
    pub fn delete(&self, url: &str) -> HttpRequest {
        self.request(HttpMethod::Delete, url)
    }

    /// Create a PATCH request
    pub fn patch(&self, url: &str) -> HttpRequest {
        self.request(HttpMethod::Patch, url)
    }

    /// Create a request with a specific method
    pub fn request(&self, method: HttpMethod, url: &str) -> HttpRequest {
        let mut request = HttpRequest::new(method, url)
            .headers(self.default_headers.clone())
            .follow_redirects(self.follow_redirects);

        if let Some(timeout) = self.timeout {
            request = request.timeout(timeout);
        }

        request
    }

    /// Execute a request from a collection endpoint
    pub async fn execute_endpoint(
        &self,
        manager: CollectionManager,
        col_name: &str,
        ep_name: &str,
    ) -> HttpResult<HttpResponse> {
        let col = manager
            .get_collection(col_name)
            .await
            .map_err(|e| HttpError::Other(e.to_string()))?
            .ok_or_else(|| HttpError::Other(format!("Collection '{}' not found", col_name)))?;

        let req = col.get_request(ep_name).ok_or_else(|| {
            HttpError::Other(format!(
                "Endpoint '{}' not found in collection '{}'",
                ep_name, col_name
            ))
        })?;

        let url = format!("{}{}", col.url, req.endpoint);
        let headers = req.headers;

        let method: HttpMethod = req.method.into();

        let mut request = HttpRequest::new(method, &url)
            .headers(headers)
            .follow_redirects(self.follow_redirects);

        if let Some(body) = &req.body {
            request = request.body(body);
        }

        if let Some(timeout) = self.timeout {
            request = request.timeout(timeout);
        }

        request.send().await
    }
}

#[cfg(test)]
mod tests {

    use crate::core::utils::build_header_map;

    use super::*;

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    }

    #[test]
    fn test_http_response_status_checks() {
        let response = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status: 200,
            status_text: "OK".to_string(),
            headers: Vec::new(),
            body: String::new(),
            elapsed_ms: 0,
            url: String::new(),
        };

        assert!(response.is_success());
        assert!(!response.is_redirect());
        assert!(!response.is_client_error());
        assert!(!response.is_server_error());
    }

    #[test]
    fn test_build_header_map() {
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), "Bearer token".to_string()),
        ];

        let header_map = build_header_map(&headers);
        assert_eq!(header_map.len(), 2);
    }
}
