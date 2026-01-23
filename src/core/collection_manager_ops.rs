use crate::core::collection_manager::CollectionResult;
use crate::core::errors::CollectionError;
use crate::core::utils::merge_headers;
use crate::{Collection, CollectionManager};

impl CollectionManager {
    /// Add a new collection
    ///
    /// If a collection with the same name exists, it will be updated.
    pub fn add_collection(
        mut self,
        name: &str,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> CollectionResult<()> {
        match self.get_collection(name) {
            Ok(c) => {
                // Update existing collection
                c.url = url.to_string();
                c.headers = merge_headers(c.headers.clone(), &headers);
            }
            Err(_) => {
                // Create new collection
                let new_collection = Collection {
                    name: name.to_string(),
                    url: url.to_string(),
                    headers: headers.clone(),
                    requests: None,
                };
                if let Some(ref mut cols) = self.loaded_collections {
                    cols.push(new_collection);
                }
            }
        };
        self.save_collections()?;
        Ok(())
    }

    /// Delete a collection
    pub fn delete_collection(mut self, name: &str) -> CollectionResult<()> {
        let collections = self
            .loaded_collections
            .as_mut()
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))?;
        let original_len = collections.len();
        collections.retain(|c| c.name != name);
        if collections.len() == original_len {
            return Err(CollectionError::CollectionNotFound(name.to_string()));
        }

        self.loaded_collections = Some(collections.to_vec());
        self.save_collections()?;
        Ok(())
    }

    /// Update a collection
    pub fn update_collection(
        mut self,
        name: &str,
        url: Option<&str>,
        headers: Option<Vec<(String, String)>>,
    ) -> CollectionResult<()> {
        let collections = self
            .loaded_collections
            .as_mut()
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))?;

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

        self.save_collections()?;
        Ok(())
    }

    /// Copy a collection to a new name
    pub fn copy_collection(mut self, name: &str, new_name: &str) -> CollectionResult<()> {
        let collections = self
            .loaded_collections
            .as_mut()
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))?;

        let col = collections
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| CollectionError::CollectionNotFound(name.to_string()))?;

        let mut new_col = col.clone();
        new_col.name = new_name.to_string();
        collections.push(new_col);

        self.save_collections()?;
        Ok(())
    }
}
