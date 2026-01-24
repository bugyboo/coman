use super::collection_manager::CollectionManager;
use crate::core::collection_manager::CollectionResult;
use crate::core::errors::CollectionError;
use crate::core::utils::merge_headers;
use crate::models::collection::Collection;

impl CollectionManager {
    /// Add a new collection
    ///
    /// If a collection with the same name exists, it will be updated.
    pub async fn add_collection(
        &self,
        name: &str,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> CollectionResult<()> {
        match self.get_collection(name).await {
            Ok(c) => {
                // Update existing collection
                let mut c = c.unwrap();
                c.url = url.to_string();
                c.headers = merge_headers(c.headers.clone(), &headers);
                self.update_add_collection(c).await?;
            }
            Err(_) => {
                // Create new collection
                let new_collection = Collection {
                    name: name.to_string(),
                    url: url.to_string(),
                    headers: headers.clone(),
                    requests: None,
                };
                self.update_add_collection(new_collection).await?;
            }
        };
        Ok(())
    }

    /// Copy a collection to a new name
    pub async fn copy_collection(&self, name: &str, new_name: &str) -> CollectionResult<()> {
        match self.get_collection(name).await {
            Ok(Some(c)) => {
                let mut new_col = c.clone();
                new_col.name = new_name.to_string();
                self.update_add_collection(new_col).await?;
                Ok(())
            }
            _ => Err(CollectionError::CollectionNotFound(name.to_string())),
        }
    }
}
