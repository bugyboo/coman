//! # Core Module
//!
//! The core module provides the main business logic for the Coman API Collection Manager.
//! It handles managing collections of API endpoints and executing HTTP requests,
//! independent of any CLI interface.
//!
//! ## Main Components
//!
//! - [`CollectionManager`]: Manages API collections and their endpoints
//! - [`HttpClient`]: Executes HTTP requests with a clean, library-friendly API
//! - [`HttpRequest`] and [`HttpResponse`]: Represent HTTP requests and responses
//! - Error types: [`CollectionError`] and [`HttpError`] for handling failures
//!
//! ## Basic Usage
//!
//! ### Managing Collections
//!
//! ```rust,no_run
//! use coman::core::CollectionManager;
//! use coman::models::collection::{Collection, Request, Method};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a collection manager
//! let manager = CollectionManager::default();
//!
//! // Create a new collection
//! let collection = Collection {
//!     name: "api.example.com".to_string(),
//!     url: "https://api.example.com".to_string(),
//!     headers: vec![
//!         ("Authorization".to_string(), "Bearer token".to_string()),
//!         ("Content-Type".to_string(), "application/json".to_string()),
//!     ],
//!     requests: Some(vec![
//!         Request {
//!             name: "get_users".to_string(),
//!             endpoint: "/users".to_string(),
//!             method: Method::Get,
//!             headers: vec![],
//!             body: None,
//!         },
//!         Request {
//!             name: "create_user".to_string(),
//!             endpoint: "/users".to_string(),
//!             method: Method::Post,
//!             headers: vec![],
//!             body: Some(r#"{"name": "John Doe"}"#.to_string()),
//!         },
//!     ]),
//! };
//!
//! // Add the collection
//! manager.update_add_collection(collection).await?;
//!
//! // Get all collections
//! let collections = manager.get_collections().await;
//! println!("Loaded {} collections", collections.len());
//!
//! // Get a specific collection
//! if let Some(col) = manager.get_collection("api.example.com").await? {
//!     println!("Found collection: {}", col.name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Making HTTP Requests
//!
//! ```rust,no_run
//! use coman::core::{HttpClient, HttpMethod};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an HTTP client with default settings
//! let client = HttpClient::new()
//!     .with_timeout(Duration::from_secs(30))
//!     .with_follow_redirects(true)
//!     .with_default_headers(vec![
//!         ("User-Agent".to_string(), "Coman/1.0".to_string()),
//!     ]);
//!
//! // Make a simple GET request
//! let response = client.get("https://httpbin.org/get").send().await?;
//! println!("Status: {}", response.status);
//! println!("Response: {}", response.body);
//!
//! // Make a POST request with JSON body
//! let response = client
//!     .post("https://httpbin.org/post")
//!     .header("Content-Type", "application/json")
//!     .body(r#"{"key": "value"}"#)
//!     .send()
//!     .await?;
//!
//! if response.is_success() {
//!     println!("Request successful!");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Executing Collection Endpoints
//!
//! ```rust,no_run
//! use coman::core::{CollectionManager, HttpClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = CollectionManager::default();
//! let client = HttpClient::new();
//!
//! // Execute a saved endpoint from a collection
//! let response = client
//!     .execute_endpoint(manager, "api.example.com", "get_users")
//!     .await?;
//!
//! println!("Response status: {}", response.status);
//! println!("Response body: {}", response.body);
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! The core module uses `Result` types for all operations that can fail:
//!
//! - `CollectionResult<T>` for collection operations
//! - `HttpResult<T>` for HTTP operations
//!
//! ```rust,no_run
//! use coman::core::{CollectionManager, HttpClient};
//!
//! # async fn example() {
//! let manager = CollectionManager::default();
//! let client = HttpClient::new();
//!
//! match manager.get_collection("nonexistent").await {
//!     Ok(Some(collection)) => println!("Found: {}", collection.name),
//!     Ok(None) => println!("Collection not found"),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//!
//! match client.get("https://invalid.url").send().await {
//!     Ok(response) => println!("Success: {}", response.status),
//!     Err(e) => eprintln!("HTTP Error: {}", e),
//! }
//! # }
//! ```

pub mod collection_manager;
pub mod collection_manager_ops;
pub mod endpoint_ops;
pub mod errors;
pub mod http_client;
pub mod http_request;
pub mod http_response;
pub mod utils;
