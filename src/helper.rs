use std::{env, io::{self, Write}};

use once_cell::sync::Lazy;
use reqwest::header::HeaderMap;

pub static HOME_DIR: Lazy<String> = Lazy::new(|| {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
});

pub static COMAN_JSON : &str = "coman.json";

pub fn parse_header(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid header format: '{}'. Use KEY:VALUE", s));
    }
    Ok((
        parts[0].trim().to_string(),
        parts[1].trim().to_string(),
    ))
}

pub fn build_header_map(headers: &[(String, String)]) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        if let Ok(header_name) = key.parse::<reqwest::header::HeaderName>() {
            header_map.insert(header_name, value.parse().unwrap());
        }
    }
    header_map
}

pub fn write_json_to_file<T: serde::Serialize>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(format!("{}/{}", *HOME_DIR, COMAN_JSON), json)?;
    Ok(())
}

pub fn read_json_from_file<T: serde::de::DeserializeOwned>() -> Result<T, Box<dyn std::error::Error>> {
    let file_path = format!("{}/{}", *HOME_DIR, COMAN_JSON);
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", file_path),
        )));
    }
    let json = std::fs::read_to_string(file_path)?;
    let data = serde_json::from_str(&json)?;
    Ok(data)
}

pub fn confirm(prompt: &str) -> bool {
    print!("{} (y/n): ", prompt);
    io::stdout().flush().ok();
    let mut response = String::new();
    std::io::stdin().read_line(&mut response).ok();
    response.to_lowercase().starts_with('y')
}