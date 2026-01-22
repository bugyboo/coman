//! Core module containing the main business logic
//!
//! This module provides the core functionality for managing collections
//! and making HTTP requests, independent of CLI concerns.

pub mod collection_manager;
pub mod collection_manager_ops;
pub mod endpoint_ops;
pub mod errors;
pub mod http_client;
pub mod http_request;
pub mod http_response;
pub mod utils;
