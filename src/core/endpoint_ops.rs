use std::collections::HashMap;

use crate::{
    core::{
        errors::{CollectionError, CollectionResult},
        utils::merge_headers,
    },
    Collection, CollectionManager, Method, Request,
};

impl CollectionManager {
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
                req.headers = merge_headers(req.headers.clone(), &new_headers);
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

    /// List all collections
    pub fn list_collections(&self) -> CollectionResult<Vec<Collection>> {
        self.load_collections()
    }
}
