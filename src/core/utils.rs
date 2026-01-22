use std::collections::HashMap;

use reqwest::header::HeaderMap;

/// Merge headers, replacing existing ones and removing those with empty values
pub fn merge_headers(
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

/// Build a HeaderMap from a vector of key-value pairs
pub fn build_header_map(headers: &[(String, String)]) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        if let Ok(header_name) = key.parse::<reqwest::header::HeaderName>() {
            if let Ok(header_value) = value.parse() {
                header_map.insert(header_name, header_value);
            }
        }
    }
    header_map
}
