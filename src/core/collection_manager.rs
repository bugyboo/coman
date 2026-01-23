//! Collection Manager - Core business logic for managing API collections
//!
//! This module provides a clean API for managing collections and endpoints
//! without any CLI dependencies.

use crate::core::errors::CollectionError;
use crate::models::collection::Collection;
use crate::{helper, Request};

/// Result type for collection operations
pub type CollectionResult<T> = Result<T, CollectionError>;

/// Manager for API collections
///
/// Provides methods for CRUD operations on collections and endpoints.
#[derive(Clone)]
pub struct CollectionManager {
    in_memory: bool,
    file_path: Option<String>,
    pub loaded_collections: Option<Vec<Collection>>,
}

impl Default for CollectionManager {
    fn default() -> Self {
        Self::new(None, false)
    }
}

impl CollectionManager {
    /// Create a new CollectionManager
    ///
    /// # Arguments
    ///
    /// * `file_path` - Optional custom file path. If None, uses default location.
    pub fn new(file_path: Option<String>, in_memory: bool) -> Self {
        if let Some(ref path) = file_path {
            std::env::set_var("COMAN_JSON", path);
        }
        Self {
            in_memory,
            file_path,
            loaded_collections: if in_memory {
                Some(Vec::new())
            } else {
                Self::load_collections_from_file().ok()
            },
        }
    }

    /// Get the file path being used
    pub fn get_file_path(&self) -> String {
        self.file_path
            .clone()
            .unwrap_or_else(|| helper::get_file_path().to_string())
    }

    /// Load all collections from the storage file
    fn load_collections_from_file() -> CollectionResult<Vec<Collection>> {
        match helper::read_json_from_file::<Vec<Collection>>() {
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

    /// Get a specific collection by name
    pub fn get_collection(&mut self, name: &str) -> CollectionResult<&mut Collection> {
        match &mut self.loaded_collections {
            Some(cols) => cols
                .iter_mut()
                .find(|c| c.name == name)
                .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string())),
            None => Err(CollectionError::CollectionNotFound(name.to_string())),
        }
    }

    /// Get a specific collection imutable by name
    pub fn get_collection_imutable(&self, name: &str) -> CollectionResult<&Collection> {
        match &self.loaded_collections {
            Some(cols) => cols
                .iter()
                .find(|c| c.name == name)
                .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string())),
            None => Err(CollectionError::CollectionNotFound(name.to_string())),
        }
    }

    /// Get a specific endpoint from a collection
    pub fn get_endpoint(&mut self, col_name: &str, ep_name: &str) -> CollectionResult<Request> {
        let col = self.get_collection(col_name)?;
        col.get_request(ep_name)
            .ok_or_else(|| CollectionError::EndpointNotFound(ep_name.to_string()))
    }

    /// Save collections to the storage file
    pub fn save_collections(self) -> CollectionResult<()> {
        if !self.in_memory {
            if let Some(collections) = self.loaded_collections {
                helper::write_json_to_file(&collections)?;
                Ok(())
            } else {
                Err(CollectionError::Other(
                    "No collections loaded to save".to_string(),
                ))
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serial_test::serial;

    fn setup_test_manager() -> CollectionManager {
        std::env::set_var("COMAN_JSON", "test.json");
        CollectionManager::new(Some("test.json".to_string()), false)
    }

    #[test]
    #[serial]
    fn test_load_collections() {
        let manager = setup_test_manager();
        let result = manager.loaded_collections;
        assert!(result.is_some());
    }

    #[test]
    #[serial]
    fn test_get_collection() {
        let mut manager = setup_test_manager();

        let result = manager.get_collection("coman");

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_save_collections() {
        let manager = setup_test_manager();

        let result = manager.save_collections();
        assert!(result.is_ok());
    }
}
