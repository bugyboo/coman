//! Test runner for API collections
//!
//! This module provides the test command functionality for the CLI.

use colored::Colorize;
use coman::cli::manager::ManagerCommands;
use coman::cli::request::RequestCommands;
use coman::CollectionManager;

pub async fn run_tests(collection_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let manager = CollectionManager::default();
    let collections = manager.load_collections().unwrap_or_default();
    let collection = collections
        .iter()
        .find(|col| col.name == collection_name)
        .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;

    if let Some(requests) = &collection.requests {
        for request in requests {
            let command = ManagerCommands::get_endpoint_command(collection_name, &request.name)
                .ok_or_else(|| {
                    format!(
                        "Endpoint '{}' not found in collection '{}'",
                        request.name, collection_name
                    )
                })?;

            let stdin_input = Vec::new();
            // Run the request
            match command.execute_request(false, stdin_input, false).await {
                Ok((response, elapsed)) => {
                    // Print the test result in the same format as print_request_method
                    println!(
                        "[{}] {} - {} ({} ms)\n",
                        command.to_string().bold().bright_yellow(),
                        response.url().to_string().bold().bright_white(),
                        RequestCommands::colorize_status(response.status()),
                        elapsed
                    );
                }
                Err(e) => {
                    // Print error message and continue
                    println!("Failed: {}", e);
                }
            }
        }
    } else {
        println!("No requests found in collection '{}'", collection_name);
    }

    Ok("Tests completed".to_string())
}
