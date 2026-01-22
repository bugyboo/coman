use crate::core::errors::CollectionError;
use crate::core::utils::merge_headers;
use crate::{core::errors::CollectionResult, Collection, CollectionManager};

impl CollectionManager {
    /// Get a specific collection by name
    pub fn get_collection(&self, name: &str) -> CollectionResult<Collection> {
        let collections = self.load_collections()?;
        collections
            .into_iter()
            .find(|c| c.name == name)
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))
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
            col.headers = merge_headers(col.headers.clone(), &new_headers);
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
}
