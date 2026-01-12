use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use fs2::FileExt;
use once_cell::sync::Lazy;
use tempfile::NamedTempFile;

pub static COMAN_FILE: &str = "coman.json";

pub static HOME_DIR: Lazy<String> = Lazy::new(|| {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
});

pub static COMAN_JSON: Lazy<String> =
    Lazy::new(|| env::var("COMAN_JSON").unwrap_or_else(|_| COMAN_FILE.to_string()));

fn get_file_path() -> String {
    if COMAN_FILE != *COMAN_JSON {
        COMAN_JSON.to_string()
    } else {
        format!("{}/{}", *HOME_DIR, *COMAN_JSON)
    }
}

/// Atomically writes JSON data to file with file locking.
/// 
/// This function:
/// 1. Acquires an exclusive lock on a lock file to prevent concurrent writes
/// 2. Writes data to a temporary file in the same directory
/// 3. Atomically renames the temp file to the target file
/// 
/// This ensures file integrity even if the process is interrupted.
pub fn write_json_to_file<T: serde::Serialize>(data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_file_path();
    let path = Path::new(&file_path);
    
    // Get parent directory for temp file (must be on same filesystem for atomic rename)
    let parent_dir = path.parent().unwrap_or(Path::new("."));
    
    // Create lock file path
    let lock_path = format!("{}.lock", file_path);
    
    // Open/create lock file and acquire exclusive lock
    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)?;
    
    // Acquire exclusive lock (blocks until available)
    lock_file.lock_exclusive()?;
    
    // Use a closure to ensure lock is released even on error
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
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
    })();
    
    // Release lock (automatically dropped, but explicit for clarity)
    lock_file.unlock()?;
    
    result
}

/// Reads JSON data from file with shared file locking.
/// 
/// This function:
/// 1. Acquires a shared lock allowing concurrent reads but blocking writes
/// 2. Reads and deserializes the JSON data
/// 
/// This prevents reading partially written data during concurrent access.
pub fn read_json_from_file<T: serde::de::DeserializeOwned>() -> Result<T, Box<dyn std::error::Error>>
{
    let file_path = get_file_path();
    let path = Path::new(&file_path);
    
    // Create lock file path (same as write uses)
    let lock_path = format!("{}.lock", file_path);
    
    // Try to open lock file - if it doesn't exist, neither does the data file
    let lock_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(_) => {
            // If we can't create lock file, check if data file exists
            if !path.exists() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {}", file_path),
                )));
            }
            // Try to read without lock (fallback)
            let json = std::fs::read_to_string(&file_path)?;
            let data = serde_json::from_str(&json)?;
            return Ok(data);
        }
    };
    
    // Acquire shared lock (allows concurrent reads, blocks writes)
    lock_file.lock_shared()?;
    
    let result = (|| -> Result<T, Box<dyn std::error::Error>> {
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
    })();
    
    // Release shared lock
    lock_file.unlock()?;
    
    result
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
    fn test_01_get_file_path() {
        //set env variable
        std::env::set_var("COMAN_JSON", "test.json");

        let path = "test.json".to_string();

        assert_eq!(super::get_file_path(), path);
    }

    #[test]
    #[serial]
    fn test_02_write_json_to_file() {
        std::env::set_var("COMAN_JSON", "test.json");
        
        let collection = crate::models::collection::Collection {
            name: "coman".to_owned(),
            url: "http://localhost:8080".to_owned(),
            headers: vec![],
            requests: None,
        };

        let data = vec![collection];
        let result = super::write_json_to_file(&data);

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_03_read_json_from_file() {
        std::env::set_var("COMAN_JSON", "test.json");
        
        let result: Result<Vec<crate::models::collection::Collection>, Box<dyn std::error::Error>> =
            super::read_json_from_file();

        if let Err(e) = &result {
            println!("Error: {}", e);
        }

        assert!(result.is_ok());
    }
}
