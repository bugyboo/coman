use std::time::Duration;

use futures::StreamExt;
use reqwest::multipart::Part;
use reqwest::redirect::Policy;
use reqwest::{multipart, ClientBuilder};

use crate::core::errors::HttpError;
use crate::core::http_client::{HttpMethod, HttpResult};
use crate::core::http_response::HttpResponse;
use crate::core::utils::build_header_map;

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

        let mut headers = Vec::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.push((key.to_string(), v.to_string()));
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

        let mut headers = Vec::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.push((key.to_string(), v.to_string()));
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

        let mut headers = Vec::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.push((key.to_string(), v.to_string()));
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
