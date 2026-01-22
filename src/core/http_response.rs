use std::collections::HashMap;

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
