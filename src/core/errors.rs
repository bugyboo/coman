/// Errors that can occur during collection operations
#[derive(Debug)]
pub enum CollectionError {
    /// Collection was not found
    CollectionNotFound(String),
    /// Endpoint was not found
    EndpointNotFound(String),
    /// IO error occurred
    IoError(std::io::Error),
    /// JSON serialization/deserialization error
    JsonError(serde_json::Error),
    /// Generic error with message
    Other(String),
}

impl std::fmt::Display for CollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionError::CollectionNotFound(name) => {
                write!(f, "Collection not found: {}", name)
            }
            CollectionError::EndpointNotFound(name) => write!(f, "Endpoint not found: {}", name),
            CollectionError::IoError(e) => write!(f, "IO error: {}", e),
            CollectionError::JsonError(e) => write!(f, "JSON error: {}", e),
            CollectionError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CollectionError {}

impl From<std::io::Error> for CollectionError {
    fn from(err: std::io::Error) -> Self {
        CollectionError::IoError(err)
    }
}

impl From<serde_json::Error> for CollectionError {
    fn from(err: serde_json::Error) -> Self {
        CollectionError::JsonError(err)
    }
}

impl From<Box<dyn std::error::Error>> for CollectionError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CollectionError::Other(err.to_string())
    }
}

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
