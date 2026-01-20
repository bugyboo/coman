//! Collection Manager - Core business logic for managing API collections
//!
//! This module provides a clean API for managing collections and endpoints
//! without any CLI dependencies.

use std::collections::HashMap;

use crate::helper;
use crate::models::collection::{Collection, Method, Request};

/// Result type for collection operations
pub type CollectionResult<T> = Result<T, CollectionError>;

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

/// Manager for API collections
///
/// Provides methods for CRUD operations on collections and endpoints.
#[derive(Clone)]
pub struct CollectionManager {
    file_path: Option<String>,
}

impl Default for CollectionManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CollectionManager {
    /// Create a new CollectionManager
    ///
    /// # Arguments
    ///
    /// * `file_path` - Optional custom file path. If None, uses default location.
    pub fn new(file_path: Option<String>) -> Self {
        if let Some(ref path) = file_path {
            std::env::set_var("COMAN_JSON", path);
        }
        Self { file_path }
    }

    /// Get the file path being used
    pub fn get_file_path(&self) -> String {
        self.file_path
            .clone()
            .unwrap_or_else(|| helper::get_file_path().to_string())
    }

    /// Load all collections from the storage file
    pub fn load_collections(&self) -> CollectionResult<Vec<Collection>> {
        match helper::read_json_from_file() {
            Ok(c) => Ok(c),
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        Ok(Vec::new())
                    } else {
                        Err(CollectionError::Other(e.to_string()))
                    }
                } else {
                    Err(CollectionError::Other(e.to_string()))
                }
            }
        }
    }

    /// Save collections to the storage file
    pub fn save_collections(&self, collections: &[Collection]) -> CollectionResult<()> {
        let vec: Vec<Collection> = collections.to_vec();
        helper::write_json_to_file(&vec).map_err(|e| CollectionError::Other(e.to_string()))
    }

    /// Get a specific collection by name
    pub fn get_collection(&self, name: &str) -> CollectionResult<Collection> {
        let collections = self.load_collections()?;
        collections
            .into_iter()
            .find(|c| c.name == name)
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))
    }

    /// Get a specific endpoint from a collection
    pub fn get_endpoint(&self, collection: &str, endpoint: &str) -> CollectionResult<Request> {
        let col = self.get_collection(collection)?;
        col.requests
            .and_then(|requests| requests.into_iter().find(|r| r.name == endpoint))
            .ok_or_else(|| CollectionError::EndpointNotFound(endpoint.to_string()))
    }

    /// Get the full URL for an endpoint (base URL + endpoint path)
    pub fn get_endpoint_url(&self, collection: &str, endpoint: &str) -> CollectionResult<String> {
        let col = self.get_collection(collection)?;
        let req = self.get_endpoint(collection, endpoint)?;
        Ok(format!("{}{}", col.url, req.endpoint))
    }

    /// Get merged headers for an endpoint (collection headers + endpoint headers)
    pub fn get_endpoint_headers(
        &self,
        collection: &str,
        endpoint: &str,
    ) -> CollectionResult<Vec<(String, String)>> {
        let col = self.get_collection(collection)?;
        let req = self.get_endpoint(collection, endpoint)?;

        let mut merged: HashMap<String, String> = HashMap::new();
        for (k, v) in &col.headers {
            merged.insert(k.clone(), v.clone());
        }
        for (k, v) in &req.headers {
            merged.insert(k.clone(), v.clone());
        }

        Ok(merged.into_iter().collect())
    }

    /// Add a new collection
    ///
    /// If a collection with the same name exists, it will be updated.
    pub fn add_collection(
        &self,
        name: &str,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        if let Some(col) = collections.iter_mut().find(|c| c.name == name) {
            // Update existing collection
            col.url = url.to_string();
            col.headers = headers;
        } else {
            // Add new collection
            collections.push(Collection {
                name: name.to_string(),
                url: url.to_string(),
                headers,
                requests: None,
            });
        }

        self.save_collections(&collections)
    }

    /// Delete a collection
    pub fn delete_collection(&self, name: &str) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;
        let original_len = collections.len();
        collections.retain(|c| c.name != name);

        if collections.len() == original_len {
            return Err(CollectionError::CollectionNotFound(name.to_string()));
        }

        self.save_collections(&collections)
    }

    /// Update a collection
    pub fn update_collection(
        &self,
        name: &str,
        url: Option<&str>,
        headers: Option<Vec<(String, String)>>,
    ) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        let col = collections
            .iter_mut()
            .find(|c| c.name == name)
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))?;

        if let Some(url) = url {
            col.url = url.to_string();
        }

        if let Some(new_headers) = headers {
            col.headers = Self::merge_headers(col.headers.clone(), &new_headers);
        }

        self.save_collections(&collections)
    }

    /// Copy a collection to a new name
    pub fn copy_collection(&self, name: &str, new_name: &str) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        let col = collections
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))?;

        let mut new_col = col.clone();
        new_col.name = new_name.to_string();
        collections.push(new_col);

        self.save_collections(&collections)
    }

    /// Add an endpoint to a collection
    ///
    /// If an endpoint with the same name exists, it will be updated.
    pub fn add_endpoint(
        &self,
        collection: &str,
        name: &str,
        path: &str,
        method: Method,
        headers: Vec<(String, String)>,
        body: Option<String>,
    ) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        let col = collections
            .iter_mut()
            .find(|c| c.name == collection)
            .ok_or_else(|| CollectionError::CollectionNotFound(collection.to_string()))?;

        let request = Request {
            name: name.to_string(),
            endpoint: path.to_string(),
            method,
            headers,
            body,
        };

        let mut requests = col.requests.clone().unwrap_or_default();
        requests.retain(|r| r.name != name);
        requests.push(request);
        col.requests = Some(requests);

        self.save_collections(&collections)
    }

    /// Delete an endpoint from a collection
    pub fn delete_endpoint(&self, collection: &str, endpoint: &str) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        let col = collections
            .iter_mut()
            .find(|c| c.name == collection)
            .ok_or_else(|| CollectionError::CollectionNotFound(collection.to_string()))?;

        if let Some(requests) = col.requests.as_mut() {
            let original_len = requests.len();
            requests.retain(|r| r.name != endpoint);

            if requests.len() == original_len {
                return Err(CollectionError::EndpointNotFound(endpoint.to_string()));
            }
        } else {
            return Err(CollectionError::EndpointNotFound(endpoint.to_string()));
        }

        self.save_collections(&collections)
    }

    /// Update an endpoint in a collection
    pub fn update_endpoint(
        &self,
        collection: &str,
        endpoint: &str,
        path: Option<&str>,
        headers: Option<Vec<(String, String)>>,
        body: Option<String>,
    ) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        let col = collections
            .iter_mut()
            .find(|c| c.name == collection)
            .ok_or_else(|| CollectionError::CollectionNotFound(collection.to_string()))?;

        if let Some(requests) = col.requests.as_mut() {
            let req = requests
                .iter_mut()
                .find(|r| r.name == endpoint)
                .ok_or_else(|| CollectionError::EndpointNotFound(endpoint.to_string()))?;

            if let Some(path) = path {
                req.endpoint = path.to_string();
            }

            if let Some(new_headers) = headers {
                req.headers = Self::merge_headers(req.headers.clone(), &new_headers);
            }

            if let Some(new_body) = body {
                req.body = if new_body.is_empty() {
                    None
                } else {
                    Some(new_body)
                };
            }
        } else {
            return Err(CollectionError::EndpointNotFound(endpoint.to_string()));
        }

        self.save_collections(&collections)
    }

    /// Copy an endpoint within the same collection or to another collection
    pub fn copy_endpoint(
        &self,
        collection: &str,
        endpoint: &str,
        new_name: &str,
        to_collection: Option<&str>,
    ) -> CollectionResult<()> {
        let mut collections = self.load_collections()?;

        // Find the source endpoint
        let source_col = collections
            .iter()
            .find(|c| c.name == collection)
            .ok_or_else(|| CollectionError::CollectionNotFound(collection.to_string()))?;

        let source_req = source_col
            .requests
            .as_ref()
            .and_then(|r| r.iter().find(|r| r.name == endpoint))
            .ok_or_else(|| CollectionError::EndpointNotFound(endpoint.to_string()))?;

        let mut new_req = source_req.clone();

        if let Some(target_col_name) = to_collection {
            // Copy to another collection (keep original name)
            let target_col = collections
                .iter_mut()
                .find(|c| c.name == target_col_name)
                .ok_or_else(|| CollectionError::CollectionNotFound(target_col_name.to_string()))?;

            let mut requests = target_col.requests.clone().unwrap_or_default();
            requests.push(new_req);
            target_col.requests = Some(requests);
        } else {
            // Copy within the same collection with a new name
            new_req.name = new_name.to_string();

            let col = collections
                .iter_mut()
                .find(|c| c.name == collection)
                .ok_or_else(|| CollectionError::CollectionNotFound(collection.to_string()))?;

            let mut requests = col.requests.clone().unwrap_or_default();
            requests.push(new_req);
            col.requests = Some(requests);
        }

        self.save_collections(&collections)
    }

    /// List all collections
    pub fn list_collections(&self) -> CollectionResult<Vec<Collection>> {
        self.load_collections()
    }

    /// Merge headers, replacing existing ones and removing those with empty values
    fn merge_headers(
        existing: Vec<(String, String)>,
        new_headers: &[(String, String)],
    ) -> Vec<(String, String)> {
        let mut merged: HashMap<String, String> = existing.into_iter().collect();
        for (key, value) in new_headers.iter() {
            if merged.contains_key(key) {
                if value.is_empty() {
                    merged.remove(key);
                } else {
                    merged.entry(key.clone()).and_modify(|v| *v = value.clone());
                }
            } else {
                merged.insert(key.clone(), value.clone());
            }
        }
        merged.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serial_test::serial;

    fn setup_test_manager() -> CollectionManager {
        std::env::set_var("COMAN_JSON", "test.json");
        CollectionManager::new(Some("test.json".to_string()))
    }

    #[test]
    #[serial]
    fn test_load_collections() {
        let manager = setup_test_manager();
        let result = manager.load_collections();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_get_collection() {
        let manager = setup_test_manager();
        // This test assumes there's a collection in test.json
        let result = manager.load_collections();
        assert!(result.is_ok());

        let collections = result.unwrap();

        if let Some(col) = collections.first() {
            let result = manager.get_collection(&col.name);
            assert!(result.is_ok());
            let fetched_col = result.unwrap();
            assert_eq!(fetched_col.name, col.name);
        }
    }

    #[test]
    #[serial]
    fn test_save_collections() {
        let manager = setup_test_manager();
        // This test assumes there's a collection in test.json
        let result = manager.load_collections();
        assert!(result.is_ok());

        let result = manager.save_collections(result.unwrap().as_slice());
        assert!(result.is_ok());        
    }
}
