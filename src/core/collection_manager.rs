//! Collection Manager - Core business logic for managing API collections
//!
//! This module provides a clean API for managing collections and endpoints
//! without any CLI dependencies.

use std::sync::Arc;

use tokio::sync::Mutex;

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
    loaded_collections: Arc<Mutex<Vec<Collection>>>,
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
                Arc::new(Mutex::new(Vec::new()))
            } else {
                Self::load_collections_from_file()
                    .unwrap_or_else(|_| Arc::new(Mutex::new(Vec::new())))
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
    fn load_collections_from_file() -> CollectionResult<Arc<Mutex<Vec<Collection>>>> {
        match helper::read_json_from_file::<Vec<Collection>>() {
            Ok(c) => Ok(Arc::new(Mutex::new(c))),
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        Ok(Arc::new(Mutex::new(Vec::new())))
                    } else {
                        Err(CollectionError::Other(e.to_string()))
                    }
                } else {
                    Err(CollectionError::Other(e.to_string()))
                }
            }
        }
    }

    /// Get loaded collections
    pub async fn get_collections(&self) -> Vec<Collection> {
        let collections = self.loaded_collections.lock().await;
        collections.clone()
    }

    /// Get a specific Collection by name
    pub async fn get_collection(&self, name: &str) -> CollectionResult<Option<Collection>> {
        let collections = self.loaded_collections.lock().await;
        for c in collections.iter() {
            if c.name == name {
                return Ok(Some(c.clone()));
            }
        }
        Err(CollectionError::CollectionNotFound(name.to_string()))
    }

    /// Get a specific Endpoint by collection name and request name
    pub async fn get_endpoint(
        &self,
        col_name: &str,
        ep_name: &str,
    ) -> CollectionResult<Option<Request>> {
        let collections = self.loaded_collections.lock().await;
        for c in collections.iter() {
            if c.name == col_name {
                if let Some(ref requests) = c.requests {
                    for r in requests.iter() {
                        if r.name == ep_name {
                            return Ok(Some(r.clone()));
                        }
                    }
                }
            }
        }
        Err(CollectionError::EndpointNotFound(format!(
            "{} in {}",
            ep_name, col_name
        )))
    }

    /// Update an existing collection or add a new one
    pub async fn update_add_collection(&self, updated: Collection) -> CollectionResult<()> {
        let mut collections = self.loaded_collections.lock().await;
        if let Some(pos) = collections.iter().position(|c| c.name == updated.name) {
            collections[pos] = updated;
            if !self.in_memory {
                helper::write_json_to_file(&*collections)?;
            }
            Ok(())
        } else {
            collections.push(updated);
            if !self.in_memory {
                helper::write_json_to_file(&*collections)?;
            }
            Ok(())
        }
    }

    /// Update an existing request within a collection or add a new one
    pub async fn update_add_request(
        &self,
        col_name: &str,
        ep_name: &str,
        updated: Request,
    ) -> CollectionResult<()> {
        let mut collections = self.loaded_collections.lock().await;
        if let Some(col_pos) = collections.iter().position(|c| c.name == col_name) {
            if let Some(ref mut requests) = collections[col_pos].requests {
                if let Some(req_pos) = requests.iter().position(|r| r.name == ep_name) {
                    requests[req_pos] = updated;
                    if !self.in_memory {
                        helper::write_json_to_file(&*collections)?;
                    }
                    return Ok(());
                } else {
                    requests.push(updated);
                    if !self.in_memory {
                        helper::write_json_to_file(&*collections)?;
                    }
                    return Ok(());
                }
            } else {
                collections[col_pos].requests = Some(vec![updated]);
                if !self.in_memory {
                    helper::write_json_to_file(&*collections)?;
                }
                return Ok(());
            }
        }
        Err(CollectionError::CollectionNotFound(col_name.to_string()))
    }

    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> CollectionResult<()> {
        match self.get_collection(name).await {
            Ok(_) => {
                let mut collections = self.loaded_collections.lock().await;
                collections.retain(|c| c.name != name);
                if !self.in_memory {
                    helper::write_json_to_file(&*collections)?;
                }
                Ok(())
            }
            Err(_) => Err(CollectionError::CollectionNotFound(name.to_string())),
        }
    }

    /// Save collections to the storage file
    pub async fn save_loaded_collections(self) -> CollectionResult<()> {
        if !self.in_memory {
            let collections = self.loaded_collections.lock().await;
            helper::write_json_to_file(&*collections)?;
        }
        Ok(())
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

    #[tokio::test]
    #[serial]
    async fn test_load_collections() {
        let manager = setup_test_manager();
        let result = manager.get_collections().await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_collection() {
        let manager = setup_test_manager();

        let result = manager.get_collection("coman").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_save_collections() {
        let manager = setup_test_manager();

        let result = manager.save_loaded_collections().await;
        assert!(result.is_ok());
    }
}
