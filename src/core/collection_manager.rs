//! Collection Manager - Core business logic for managing API collections
//!
//! This module provides a clean API for managing collections and endpoints
//! without any CLI dependencies.

use crate::core::errors::{CollectionError, CollectionResult};
use crate::helper;
use crate::models::collection::Collection;

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
