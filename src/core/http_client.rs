//! HTTP Client - Core HTTP request functionality
//!
//! This module provides a clean, library-friendly HTTP client API
//! without any CLI dependencies (no progress bars, colored output, etc.)

use futures::stream::StreamExt;
use reqwest::header::HeaderMap;
use reqwest::multipart::{self, Part};
use reqwest::{redirect::Policy, ClientBuilder};
use std::collections::HashMap;
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

/// Errors that can occur during HTTP operations
#[derive(Debug)]
pub enum HttpError {
    /// Request timed out
    Timeout,
    /// Connection error
    ConnectionError(String),
    /// Redirect error
    RedirectError(String),
    /// Request building error
    RequestError(String),
    /// Response error
    ResponseError(String),
    /// Generic error
    Other(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::Timeout => write!(f, "Request timed out"),
            HttpError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            HttpError::RedirectError(msg) => write!(f, "Redirect error: {}", msg),
            HttpError::RequestError(msg) => write!(f, "Request error: {}", msg),
            HttpError::ResponseError(msg) => write!(f, "Response error: {}", msg),
            HttpError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

impl From<reqwest::Error> for HttpError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            HttpError::Timeout
        } else if err.is_connect() {
            HttpError::ConnectionError(err.to_string())
        } else if err.is_redirect() {
            HttpError::RedirectError(err.to_string())
        } else {
            HttpError::Other(err.to_string())
        }
    }
}

/// HTTP Response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP Version
    pub version: String,
    /// Response status code
    pub status: u16,
    /// Response status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as string
    pub body: String,
    /// Response body as bytes (for binary data)
    // pub body_bytes: Vec<u8>,
    /// Request duration in milliseconds
    pub elapsed_ms: u128,
    /// Final URL (after redirects)
    pub url: String,
}

impl HttpResponse {
    /// Check if the response status is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response status is a redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if the response status is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response status is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Try to parse the response body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.body)
    }
}

/// HTTP Request Builder
#[derive(Debug, Clone)]
pub struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: Vec<(String, String)>,
    body: Option<String>,
    body_bytes: Option<Vec<u8>>,
    timeout: Option<Duration>,
    follow_redirects: bool,
}

impl HttpRequest {
    /// Create a new HTTP request
    pub fn new(method: HttpMethod, url: &str) -> Self {
        Self {
            url: url.to_string(),
            method,
            headers: Vec::new(),
            body: None,
            body_bytes: None,
            timeout: None,
            follow_redirects: false,
        }
    }

    /// Set request headers
    pub fn headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = headers;
        self
    }

    /// Add a single header
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    /// Set request body as string
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Set request body as bytes
    pub fn body_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body_bytes = Some(bytes);
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enable following redirects
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Execute the request
    pub async fn send(self) -> HttpResult<HttpResponse> {
        let client_builder = ClientBuilder::new();

        let client_builder = if self.follow_redirects {
            client_builder.redirect(Policy::default())
        } else {
            client_builder.redirect(Policy::none())
        };

        let client_builder = if let Some(timeout) = self.timeout {
            client_builder.timeout(timeout)
        } else {
            client_builder
        };

        let client = client_builder
            .build()
            .map_err(|e| HttpError::RequestError(e.to_string()))?;

        let header_map = build_header_map(&self.headers);

        let method = match self.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
        };

        let start = std::time::Instant::now();

        let request_builder = client.request(method, &self.url).headers(header_map);

        let request_builder = if let Some(bytes) = self.body_bytes {
            request_builder.body(bytes)
        } else if let Some(body) = self.body {
            request_builder.body(body)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;

        let elapsed = start.elapsed().as_millis();
        let status = response.status().as_u16();
        let status_text = response.status().to_string();
        let url = response.url().to_string();
        let version = format!("{:?}", response.version());

        let mut headers = HashMap::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        let body_bytes = response.bytes().await?.to_vec();
        let body = String::from_utf8_lossy(&body_bytes).to_string();

        Ok(HttpResponse {
            version,
            status,
            status_text,
            headers,
            body,
            elapsed_ms: elapsed,
            url,
        })
    }

    /// Execute the request and stream the response
    pub async fn send_streaming<F>(self, mut on_chunk: F) -> HttpResult<HttpResponse>
    where
        F: FnMut(&[u8]) -> Result<(), Box<dyn std::error::Error>> + Send,
    {
        let client_builder = ClientBuilder::new();

        let client_builder = if self.follow_redirects {
            client_builder.redirect(Policy::default())
        } else {
            client_builder.redirect(Policy::none())
        };

        let client_builder = if let Some(timeout) = self.timeout {
            client_builder.timeout(timeout)
        } else {
            client_builder
        };

        let client = client_builder
            .build()
            .map_err(|e| HttpError::RequestError(e.to_string()))?;

        let header_map = build_header_map(&self.headers);

        let method = match self.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
        };

        let start = std::time::Instant::now();

        let request_builder = client.request(method, &self.url).headers(header_map);

        let request_builder = if let Some(bytes) = self.body_bytes {
            request_builder.body(bytes)
        } else if let Some(body) = self.body {
            request_builder.body(body)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;

        let status = response.status().as_u16();
        let status_text = response.status().to_string();
        let url = response.url().to_string();
        let version = format!("{:?}", response.version());

        let mut headers = HashMap::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| HttpError::ResponseError(e.to_string()))?;
            on_chunk(&chunk).map_err(|e| HttpError::Other(e.to_string()))?;
        }

        let elapsed = start.elapsed().as_millis();

        Ok(HttpResponse {
            version,
            status,
            status_text,
            headers,
            body: String::new(),
            elapsed_ms: elapsed,
            url,
        })
    }

    pub async fn send_multipart(self, part: Part) -> HttpResult<HttpResponse> {
        let client_builder = ClientBuilder::new();

        let client_builder = if self.follow_redirects {
            client_builder.redirect(Policy::default())
        } else {
            client_builder.redirect(Policy::none())
        };

        let client_builder = if let Some(timeout) = self.timeout {
            client_builder.timeout(timeout)
        } else {
            client_builder
        };

        let client = client_builder
            .build()
            .map_err(|e| HttpError::RequestError(e.to_string()))?;

        let header_map = build_header_map(&self.headers);

        let method = match self.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
        };

        let form = multipart::Form::new().part("file", part);

        let start = std::time::Instant::now();

        let response = client
            .request(method, &self.url)
            .headers(header_map)
            .multipart(form)
            .send()
            .await?;

        let elapsed = start.elapsed().as_millis();
        let status = response.status().as_u16();
        let status_text = response.status().to_string();
        let url = response.url().to_string();
        let version = format!("{:?}", response.version());

        let mut headers = HashMap::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        let body_bytes = response.bytes().await?.to_vec();
        let body = String::from_utf8_lossy(&body_bytes).to_string();

        Ok(HttpResponse {
            version,
            status,
            status_text,
            headers,
            body,
            elapsed_ms: elapsed,
            url,
        })
    }
}

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
        manager: &crate::core::collection_manager::CollectionManager,
        collection: &str,
        endpoint: &str,
    ) -> HttpResult<HttpResponse> {
        let col = manager
            .get_collection(collection)
            .map_err(|e| HttpError::Other(e.to_string()))?;
        let req = manager
            .get_endpoint(collection, endpoint)
            .map_err(|e| HttpError::Other(e.to_string()))?;

        let url = format!("{}{}", col.url, req.endpoint);
        let headers = manager
            .get_endpoint_headers(collection, endpoint)
            .map_err(|e| HttpError::Other(e.to_string()))?;

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

/// Build a HeaderMap from a vector of key-value pairs
pub fn build_header_map(headers: &[(String, String)]) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        if let Ok(header_name) = key.parse::<reqwest::header::HeaderName>() {
            if let Ok(header_value) = value.parse() {
                header_map.insert(header_name, header_value);
            }
        }
    }
    header_map
}

#[cfg(test)]
mod tests {
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
            headers: HashMap::new(),
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
