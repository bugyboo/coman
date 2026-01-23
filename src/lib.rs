//! # Coman - API Collection Manager
//!
//! Coman is a library for managing API collections and making HTTP requests.
//! It can be used as a standalone library or through its CLI interface.

pub mod core;
pub mod helper;
pub mod models;

// Re-export main types for convenience
pub use core::collection_manager::CollectionManager;
pub use core::http_client::{HttpClient, HttpMethod, HttpResult};
pub use core::http_request::HttpRequest;
pub use core::http_response::HttpResponse;
pub use models::collection::{Collection, Method, Request};

// CLI module (only available with the cli feature)
#[cfg(feature = "cli")]
pub mod cli;
