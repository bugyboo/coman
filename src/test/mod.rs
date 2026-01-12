#[cfg(test)]
pub mod tests {

    use crate::{
        commands::{
            manager::ManagerCommands,
            request::{RequestCommands, RequestData},
        },
        Commands,
    };
    use serial_test::serial;

    /// Setup test environment - ensures all tests use the same test file
    fn setup() {
        std::env::set_var("COMAN_JSON", "test.json");
    }

    /// Clean up test data - removes test collections
    fn cleanup() {
        setup();
        // Try to delete test collections, ignore errors
        let _ = ManagerCommands::Delete {
            collection: "test".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        }.run();
        let _ = ManagerCommands::Delete {
            collection: "test2".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        }.run();
        let _ = ManagerCommands::Delete {
            collection: "test3".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        }.run();
    }

    // ==========================================
    // Integration test - runs all steps in order
    // ==========================================

    #[test]
    #[serial]
    fn test_integration_collection_workflow() {
        setup();
        cleanup();

        // Step 1: Create collection
        let command = ManagerCommands::Col {
            name: "test".to_owned(),
            url: "http://localhost:8080".to_owned(),
            headers: vec![],
        };
        assert!(command.run().is_ok(), "Failed to create collection");

        // Step 2: Create endpoint
        let command = ManagerCommands::Endpoint {
            collection: "test".to_owned(),
            name: "ver".to_owned(),
            path: "/ver".to_owned(),
            method: "GET".to_owned(),
            headers: vec![("Content-type".to_owned(), "application/json".to_owned())],
            body: "".to_owned(),
        };
        assert!(command.run().is_ok(), "Failed to create endpoint");

        // Step 3: Verify URL generation
        let url = Commands::run_url("test", "ver");
        assert!(url.is_ok(), "Failed to generate URL");
        assert!(
            url.unwrap().contains("get 'http://localhost:8080/ver' -H \"Content-type: application/json\""),
            "URL format mismatch"
        );

        // Step 4: List collections
        let command = ManagerCommands::List {
            col: "test".to_owned(),
            endpoint: "".to_owned(),
            quiet: false,
            verbose: true,
        };
        assert!(command.run().is_ok(), "Failed to list collections");

        // Step 5: Delete endpoint
        let command = ManagerCommands::Delete {
            collection: "test".to_owned(),
            endpoint: "ver".to_owned(),
            yes: true,
        };
        assert!(command.run().is_ok(), "Failed to delete endpoint");

        // Verify endpoint is gone
        let url = Commands::run_url("test", "ver");
        assert!(url.is_err(), "Endpoint should be deleted");

        // Step 6: Delete collection
        let command = ManagerCommands::Delete {
            collection: "test".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        };
        assert!(command.run().is_ok(), "Failed to delete collection");
    }

    #[test]
    #[serial]
    fn test_integration_collection_with_headers() {
        setup();

        // Create collection with headers
        let command = ManagerCommands::Col {
            name: "test2".to_owned(),
            url: "http://localhost:8080".to_owned(),
            headers: vec![
                ("Authorization".to_owned(), "Bearer token".to_owned()),
                ("Content-type".to_owned(), "application/json".to_owned()),
            ],
        };
        assert!(command.run().is_ok());

        // Create endpoint with custom headers
        let command = ManagerCommands::Endpoint {
            collection: "test2".to_owned(),
            name: "ver".to_owned(),
            path: "/ver".to_owned(),
            method: "POST".to_owned(),
            headers: vec![
                ("Content-type".to_owned(), "text/html".to_owned()),
                ("Accept".to_owned(), "application/json".to_owned()),
            ],
            body: "".to_owned(),
        };
        assert!(command.run().is_ok());

        // Verify URL includes merged headers
        let url = Commands::run_url("test2", "ver");
        assert!(url.is_ok());
        let url_str = url.unwrap();
        assert!(url_str.contains("post 'http://localhost:8080/ver'"));
        assert!(url_str.contains("-H \"Accept: application/json\""));
        assert!(url_str.contains("-H \"Authorization: Bearer token\""));

        // Update endpoint URL
        let command = ManagerCommands::Update {
            collection: "test2".to_owned(),
            endpoint: "ver".to_owned(),
            url: "/ver2".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };
        assert!(command.run().is_ok());

        // Verify URL is updated
        let url = Commands::run_url("test2", "ver");
        assert!(url.is_ok());
        assert!(url.unwrap().contains("post 'http://localhost:8080/ver2'"));

        // Copy endpoint
        let command = ManagerCommands::Copy {
            collection: "test2".to_owned(),
            endpoint: "ver".to_owned(),
            to_col: false,
            new_name: "ver3".to_owned(),
        };
        assert!(command.run().is_ok());

        // Verify copied endpoint
        let url = Commands::run_url("test2", "ver3");
        assert!(url.is_ok());

        // Copy collection
        let command = ManagerCommands::Copy {
            collection: "test2".to_owned(),
            endpoint: "".to_owned(),
            to_col: false,
            new_name: "test3".to_owned(),
        };
        assert!(command.run().is_ok());

        // Update copied collection URL
        let command = ManagerCommands::Update {
            collection: "test3".to_owned(),
            endpoint: "".to_owned(),
            url: "http://localhost:8081".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };
        assert!(command.run().is_ok());

        // List to verify
        let command = ManagerCommands::List {
            col: "test3".to_owned(),
            endpoint: "".to_owned(),
            quiet: false,
            verbose: true,
        };
        assert!(command.run().is_ok());

        // Cleanup
        let _ = ManagerCommands::Delete {
            collection: "test2".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        }.run();
        let _ = ManagerCommands::Delete {
            collection: "test3".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        }.run();
    }

    // ==========================================
    // Unit tests - self-contained
    // ==========================================

    #[test]
    #[serial]
    fn test_delete_collection_not_found() {
        setup();
        let command = ManagerCommands::Delete {
            collection: "notfound".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        };
        assert!(command.run().is_err());
    }

    // ==========================================
    // HTTP request tests - require running server
    // Skip these if no server is available
    // ==========================================

    #[tokio::test]
    #[serial]
    #[ignore] // Requires localhost:8080 server
    async fn test_req_get() {
        setup();
        let request_data = RequestData {
            url: "http://localhost:8080/ver".to_owned(),
            headers: vec![
                ("Content-Type".to_owned(), "application/json".to_owned()),
                ("Accept".to_owned(), "application/json".to_owned()),
            ],
            body: "".to_owned(),
        };

        let command = RequestCommands::Get { data: request_data };
        let result = command.run(true, Vec::new(), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    #[ignore] // Requires localhost:8080 server
    async fn test_req_post() {
        setup();
        let request_data = RequestData {
            url: "http://localhost:8080/login".to_owned(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: r#"{"username": "test", "password": "test"}"#.to_owned(),
        };

        let command = RequestCommands::Post { data: request_data };
        let result = command.run(true, Vec::new(), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    #[ignore] // Requires localhost:8080 server
    async fn test_req_put() {
        setup();
        let request_data = RequestData {
            url: "http://localhost:8080/user".to_owned(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: r#"{"name": "test test"}"#.to_owned(),
        };

        let command = RequestCommands::Put { data: request_data };
        let result = command.run(true, Vec::new(), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    #[ignore] // Requires localhost:8080 server
    async fn test_req_delete() {
        setup();
        let request_data = RequestData {
            url: "http://localhost:8080/user?id=test".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };

        let command = RequestCommands::Delete { data: request_data };
        let result = command.run(true, Vec::new(), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    #[ignore] // Requires localhost:8080 server
    async fn test_req_patch() {
        setup();
        let request_data = RequestData {
            url: "http://localhost:8080/user?id=test".to_owned(),
            headers: vec![],
            body: r#"{"name": "test test"}"#.to_owned(),
        };

        let command = RequestCommands::Patch { data: request_data };
        let result = command.run(true, Vec::new(), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    #[ignore] // Requires localhost:8080 server
    async fn test_run_request() {
        setup();
        
        // First create the collection and endpoint
        let _ = ManagerCommands::Col {
            name: "test".to_owned(),
            url: "http://localhost:8080".to_owned(),
            headers: vec![],
        }.run();
        
        let _ = ManagerCommands::Endpoint {
            collection: "test".to_owned(),
            name: "ver".to_owned(),
            path: "/ver".to_owned(),
            method: "GET".to_owned(),
            headers: vec![("Content-type".to_owned(), "application/json".to_owned())],
            body: "".to_owned(),
        }.run();

        let result = Commands::run_request("test", "ver", &true, &Vec::new(), &false).await;
        assert!(result.is_ok());

        // Cleanup
        let _ = ManagerCommands::Delete {
            collection: "test".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        }.run();
    }
}

use crate::commands::manager::ManagerCommands;
use crate::commands::request::RequestCommands;
use colored::Colorize;

pub async fn run_tests(collection_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let collections = ManagerCommands::load_collections().unwrap_or_default();
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
                Ok(response) => {
                    // Print the test result in the same format as print_request_method
                    println!(
                        "[{}] {} - {}\n",
                        command.to_string().bold().bright_yellow(),
                        response.url().to_string().bold().bright_white(),
                        RequestCommands::colorize_status(response.status())
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
