//! # Coman - API Collection Manager
//!
//! Coman is a library for managing API collections and making HTTP requests.
//! It can be used as a standalone library or through its CLI interface.
//!
//! ## Usage as a Library
//!
//! ```rust,no_run
//! use coman::{CollectionManager, HttpClient, Method};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a collection manager with a custom file path
//!     let manager = CollectionManager::new(Some("my-apis.json".to_string()));
//!
//!     // Add a new collection
//!     manager.add_collection("my-api", "https://api.example.com", vec![])?;
//!
//!     // Add an endpoint to the collection
//!     manager.add_endpoint(
//!         "my-api",
//!         "get-users",
//!         "/users",
//!         Method::Get,
//!         vec![],
//!         None,
//!     )?;
//!
//!     // Make an HTTP request using the HttpClient
//!     let client = HttpClient::new();
//!     let response = client
//!         .get("https://api.example.com/users")
//!         .headers(vec![("Authorization".to_string(), "Bearer token".to_string())])
//!         .send()
//!         .await?;
//!
//!     println!("Status: {}", response.status);
//!     println!("Body: {}", response.body);
//!
//!     Ok(())
//! }
//! ```

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
