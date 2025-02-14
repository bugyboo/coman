use std::{env, io::{self, Write}};

use once_cell::sync::Lazy;

pub static COMAN_FILE : &str = "coman.json";

pub static HOME_DIR: Lazy<String> = Lazy::new(|| {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
});

pub static COMAN_JSON: Lazy<String> = Lazy::new(|| {
    env::var("COMAN_JSON").unwrap_or_else(|_| COMAN_FILE.to_string() )
});

fn get_file_path() -> String {
    if COMAN_FILE != *COMAN_JSON {
        COMAN_JSON.to_string()
    } else {
        format!("{}/{}", *HOME_DIR, *COMAN_JSON)
    }
}

pub fn write_json_to_file<T: serde::Serialize>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(get_file_path(), json)?;
    Ok(())
}

pub fn read_json_from_file<T: serde::de::DeserializeOwned>() -> Result<T, Box<dyn std::error::Error>> {
    let file_path = get_file_path();
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
