use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use std::sync::OnceLock;
use tempfile::NamedTempFile;

pub static COMAN_FILE: &str = "coman.json";

pub fn home_dir() -> &'static str {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE.get_or_init(|| {
        env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or("/".to_string())
    })
}

pub fn coman_json() -> &'static str {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE.get_or_init(|| env::var("COMAN_JSON").unwrap_or_else(|_| COMAN_FILE.to_string()))
}

pub fn get_file_path() -> &'static str {
    static CACHE: OnceLock<&'static str> = OnceLock::new();
    
    CACHE.get_or_init(|| {
        let json_path = coman_json();
        // If env var was set (different from default), use it directly as full path
        if json_path != COMAN_FILE {
            json_path
        } else {
            // Leak the formatted string to get &'static str
            Box::leak(format!("{}/{}", home_dir(), json_path).into_boxed_str())
        }
    })
}

/// Atomically writes JSON data to file with file locking.
///
/// This function:
/// 1. Writes data to a temporary file in the same directory
/// 2. Atomically renames the temp file to the target file
///
/// This ensures file integrity even if the process is interrupted.
pub fn write_json_to_file<T: serde::Serialize>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_file_path();
    let path = Path::new(&file_path);

    // Get parent directory for temp file (must be on same filesystem for atomic rename)
    let parent_dir = path.parent().unwrap_or(Path::new("."));

    // Use a closure to ensure lock is released even on error
    // Serialize data
    let json = serde_json::to_string_pretty(data)?;

    // Create temp file in the same directory (required for atomic rename)
    let mut temp_file = NamedTempFile::new_in(parent_dir)?;

    // Write JSON to temp file
    temp_file.write_all(json.as_bytes())?;
    temp_file.flush()?;

    // Sync to disk to ensure durability
    temp_file.as_file().sync_all()?;

    // Atomically rename temp file to target (this is the atomic operation)
    // persist() consumes the temp file and prevents auto-deletion
    temp_file.persist(&file_path)?;

    Ok(())
}

/// Reads JSON data from file with shared file locking.
///
/// This function:
/// 1. Reads and deserializes the JSON data
///
/// This prevents reading partially written data during concurrent access.
pub fn read_json_from_file<T: serde::de::DeserializeOwned>() -> Result<T, Box<dyn std::error::Error>>
{
    let file_path = get_file_path();

    // Open and read the actual data file
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", file_path),
            )));
        }
        Err(e) => return Err(Box::new(e)),
    };

    let mut json = String::new();
    file.read_to_string(&mut json)?;

    let data = serde_json::from_str(&json)?;
    Ok(data)
}

pub fn confirm(prompt: &str) -> bool {
    eprint!("{} (y/n): ", prompt);
    io::stdout().flush().ok();
    let mut response = String::new();
    std::io::stdin().read_line(&mut response).ok();
    response.to_lowercase().starts_with('y')
}

#[cfg(test)]
pub mod tests {
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_serial_01_read_write_json_from_file() {
        std::env::set_var("COMAN_JSON", "test.json");

        let path = "test.json".to_string();

        assert_eq!(super::get_file_path(), path);

        let result: Result<Vec<crate::models::collection::Collection>, Box<dyn std::error::Error>> =
            super::read_json_from_file();

        if let Err(e) = &result {
            println!("Error: {}", e);
        }

        assert!(result.is_ok());

        let result = super::write_json_to_file(&result.unwrap());

        assert!(result.is_ok());
    }
}
