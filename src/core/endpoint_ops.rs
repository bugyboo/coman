use crate::core::collection_manager::CollectionResult;
use crate::core::errors::CollectionError;
use crate::core::utils::merge_headers;
use crate::{CollectionManager, Method, Request};

impl CollectionManager {
    /// Get the full URL for an endpoint (base URL + endpoint path)
    pub async fn get_endpoint_url(
        &self,
        col_name: &str,
        ep_name: &str,
    ) -> CollectionResult<String> {
        if let Some(col) = self.get_collection(col_name).await? {
            if let Some(ep) = self.get_endpoint(col_name, ep_name).await? {
                return Ok(format!("{}{}", col.url, ep.endpoint));
            }
        }
        Err(CollectionError::EndpointNotFound(format!(
            "{} in {}",
            ep_name, col_name
        )))
    }

    /// Get merged headers for an endpoint (collection headers + endpoint headers)
    pub async fn get_endpoint_headers(
        &self,
        collection: &str,
        endpoint: &str,
    ) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        if let Some(col) = self.get_collection(collection).await.unwrap() {
            headers = merge_headers(headers, &col.headers);
            if let Some(ep) = self.get_endpoint(collection, endpoint).await.unwrap() {
                headers = merge_headers(headers, &ep.headers);
            }
        }
        headers
    }

    /// Add an endpoint to a collection
    ///
    /// If an endpoint with the same name exists, it will be updated.
    pub async fn add_endpoint(
        &self,
        col_name: &str,
        ep_name: &str,
        path: &str,
        method: Method,
        headers: Vec<(String, String)>,
        body: Option<String>,
    ) -> CollectionResult<()> {
        let request = Request {
            name: ep_name.to_string(),
            endpoint: path.to_string(),
            method,
            headers,
            body,
        };

        self.update_add_request(col_name, ep_name, request).await?;
        Ok(())
    }

    /// Update an endpoint in a collection
    pub async fn update_endpoint(
        &self,
        col_name: &str,
        ep_name: &str,
        path: Option<&str>,
        headers: Option<Vec<(String, String)>>,
        body: Option<String>,
    ) -> CollectionResult<()> {
        if let Some(mut req) = self.get_endpoint(col_name, ep_name).await? {
            if let Some(p) = path {
                req.endpoint = p.to_string();
            }
            if let Some(h) = headers {
                req.headers = merge_headers(req.headers.clone(), &h);
            }
            if let Some(b) = body {
                req.body = if b.is_empty() { None } else { Some(b) };
            }
            self.update_add_request(col_name, ep_name, req).await?;
            Ok(())
        } else {
            Err(CollectionError::EndpointNotFound(format!(
                "{} in {}",
                ep_name, col_name
            )))
        }
    }

    /// Copy an endpoint within the same collection or to another collection
    pub async fn copy_endpoint(
        &self,
        col_name: &str,
        ep_name: &str,
        new_name: &str,
        to_col: Option<&str>,
    ) -> CollectionResult<()> {
        let request = self.get_endpoint(col_name, ep_name).await?;
        if let Some(req) = request {
            let mut new_req = req.clone();
            if let Some(target_col_name) = to_col {
                // Copy to another collection (keep original name)
                self.update_add_request(target_col_name, &new_req.name, new_req.clone())
                    .await?;
            } else {
                // Copy within the same collection with a new name
                new_req.name = new_name.to_string();
                self.update_add_request(col_name, &new_req.name, new_req.clone())
                    .await?;
            }
            Ok(())
        } else {
            Err(CollectionError::EndpointNotFound(format!(
                "{} in {}",
                ep_name, col_name
            )))
        }
    }

    /// Delete an endpoint from a collection
    pub async fn delete_endpoint(&self, collection: &str, endpoint: &str) -> CollectionResult<()> {
        if let Some(mut col) = self.get_collection(collection).await? {
            if let Some(ref mut requests) = col.requests {
                let original_len = requests.len();
                requests.retain(|r| r.name != endpoint);

                if requests.len() == original_len {
                    return Err(CollectionError::EndpointNotFound(endpoint.to_string()));
                }

                self.update_add_collection(col).await?;
                return Ok(());
            }
            return Err(CollectionError::EndpointNotFound(endpoint.to_string()));
        }
        Err(CollectionError::CollectionNotFound(collection.to_string()))
    }
}
