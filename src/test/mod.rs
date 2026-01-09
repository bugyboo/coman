
#[cfg(test)]
pub mod tests {

    use crate::{commands::{manager::ManagerCommands, request::{RequestCommands, RequestData}}, Commands};

    #[test]
    fn test_01_create_collection() {
        let command = ManagerCommands::Col { name: "test".to_owned(),
            url: "http://localhost:8080".to_owned(), headers: vec![] };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[test]
    fn test_02_create_endpoint() {
        let command = ManagerCommands::Endpoint {
            collection: "test".to_owned(),
            name: "ver".to_owned(),
            path: "/ver".to_owned(),
            method: "GET".to_owned(),
            headers: vec![("Content-type".to_owned(), "application/json".to_owned())],
            body: "".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());

        let url = Commands::run_url("test", "ver");

        assert!(url.is_ok() && url.unwrap().contains("get http://localhost:8080/ver -H \"Content-type: application/json\""));
    }

    #[test]
    fn test_03_list_collections() {
        let command = ManagerCommands::List { col: "test".to_owned(), endpoint: "".to_owned(), quiet: false, verbose: true };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_04_run_req() {
        let collection = "test";
        let endpoint = "ver";
        let verbose = true;
        let stdin_input = Vec::new();
        let stream = false;

        let result = Commands::run_request(
            collection,
            endpoint,
            &verbose,
            &stdin_input,
            &stream
        );

        assert!(result.await.is_ok());
    }

    #[test]
    fn test_05_run_url() {

        let result = Commands::run_url(
            "test",
            "ver"
        );

        assert!(result.is_ok())

    }

    #[test]
    fn test_06_delete_collection_not_found() {
        let command = ManagerCommands::Delete {
            collection: "notfound".to_owned(),
            endpoint: "".to_owned(),
            yes: true,
        };

        let result = command.run();

        assert!(result.is_err());
    }

    #[test]
    fn test_07_delete_endpoint() {
        let command = ManagerCommands::Delete {
            collection: "test".to_owned(),
            endpoint: "ver".to_owned(),
            yes: true,
        };

        let result = command.run();

        assert!(result.is_ok());

        let url = Commands::run_url("test", "ver");

        assert!(url.is_err());
    }

    #[test]
    fn test_08_delete_collection() {

        let command = ManagerCommands::Delete {
            collection: "test".to_owned(),
            endpoint: "".to_owned(),
            yes: true
        };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_09_req_get() {

        let request_data = RequestData {
            url: "http://localhost:8080/ver".to_owned(),
            headers: vec![
                ("Content-Type".to_owned(), "application/json".to_owned()),
                ("Accept".to_owned(), "application/json".to_owned())
            ],
            body: "".to_owned(),
        };

        let command = RequestCommands::Get {
            data: request_data
        };

        let stdin_input = Vec::new();

        let result = command.run(true, stdin_input, false).await;

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_10_req_post() {

        let request_data = RequestData {
            url: "http://localhost:8080/login".to_owned(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: format!("{{\"username\": \"test\", \"password\": \"test\"}}"),
        };

        let command = RequestCommands::Post {
            data: request_data
        };

        let stdin_input = Vec::new();

        let result = command.run(true, stdin_input, false).await;

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_11_req_put() {

        let request_data = RequestData {
            url: "http://localhost:8080/user".to_owned(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: format!("{{\"name\": \"test test\"}}"),
        };

        let command = RequestCommands::Put {
            data: request_data
        };

        let stdin_input = Vec::new();

        let result = command.run(true, stdin_input, false).await;

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_12_req_delete() {

        let request_data = RequestData {
            url: "http://localhost:8080/user?id=test".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };

        let command = RequestCommands::Delete {
            data: request_data
        };

        let stdin_input = Vec::new();

        let result = command.run(true, stdin_input, false).await;

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_13_req_patch() {

        let request_data = RequestData {
            url: "http://localhost:8080/user?id=test".to_owned(),
            headers: vec![],
            body: format!("{{\"name\": \"test test\"}}"),
        };

        let command = RequestCommands::Patch {
            data: request_data
        };

        let stdin_input = Vec::new();

        let result = command.run(true, stdin_input, false).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_14_create_collection_with_headers() {
        let command = ManagerCommands::Col { name: "test2".to_owned(),
            url: "http://localhost:8080".to_owned(),
            headers: vec![
                ("Authorization".to_owned(), "Bearer token".to_owned()),
                ("Content-type".to_owned(), "application/json".to_owned())
            ]
        };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[test]
    fn test_15_create_endpoint_with_header_and_body() {
        let command = ManagerCommands::Endpoint {
            collection: "test2".to_owned(),
            name: "ver".to_owned(),
            path: "/ver".to_owned(),
            method: "POST".to_owned(),
            headers: vec![
                ("Content-type".to_owned(), "text/html".to_owned()),
                ("Accept".to_owned(), "application/json".to_owned())
            ],
            body: "".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());

        let url = Commands::run_url("test2", "ver");

        assert!(url.is_ok() && url.as_ref().unwrap().contains("post http://localhost:8080/ver")
            && url.as_ref().unwrap().contains("-H \"Accept: application/json\"")
            && url.as_ref().unwrap().contains("-H \"Authorization: Bearer token\"")
            && url.as_ref().unwrap().contains("-H \"Content-type: text/html\"")
        );
    }

    #[test]
    fn test_16_update_endpoint() {
        let command = ManagerCommands::Update {
            collection: "test2".to_owned(),
            endpoint: "ver".to_owned(),
            url: "/ver2".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());

        let url = Commands::run_url("test2", "ver");

        assert!(url.is_ok() && url.as_ref().unwrap().contains("post http://localhost:8080/ver2")
            && url.as_ref().unwrap().contains("-H \"Accept: application/json\"")
            && url.as_ref().unwrap().contains("-H \"Authorization: Bearer token\"")
            && url.as_ref().unwrap().contains("-H \"Content-type: text/html\"")
        );
    }

    #[test]
    fn test_17_copy_endpoint() {
        let command = ManagerCommands::Copy {
            collection: "test2".to_owned(),
            endpoint: "ver".to_owned(),
            to_col: false,
            new_name: "ver3".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());

        let url = Commands::run_url("test2", "ver3");

        assert!(url.is_ok() && url.as_ref().unwrap().contains("post http://localhost:8080/ver2")
            && url.as_ref().unwrap().contains("-H \"Accept: application/json\"")
            && url.as_ref().unwrap().contains("-H \"Authorization: Bearer token\"")
            && url.as_ref().unwrap().contains("-H \"Content-type: text/html\"")
        );
    }


    #[test]
    fn test_18_copy_collection() {
        let command = ManagerCommands::Copy {
            collection: "test2".to_owned(),
            endpoint: "".to_owned(),
            to_col: false,
            new_name: "test3".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[test]
    fn test_19_update_collection() {
        let command = ManagerCommands::Update {
            collection: "test3".to_owned(),
            endpoint: "".to_owned(),
            url: "http://localhost:8081".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[test]
    fn test_20_list_collections() {
        let command = ManagerCommands::List { col: "test3".to_owned(), endpoint: "".to_owned(), quiet: false, verbose: true };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_21_run_req() {
        let collection = "test2";
        let endpoint = "ver";
        let verbose = true;
        let stdin_input = Vec::new();
        let stream = false;

        let result = Commands::run_request(
            collection,
            endpoint,
            &verbose,
            &stdin_input,
            &stream,
        );

        assert!(result.await.is_ok());
    }

    #[test]
    fn test_22_delete_collection() {

        let command = ManagerCommands::Delete {
            collection: "test2".to_owned(),
            endpoint: "".to_owned(),
            yes: true
        };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[test]
    fn test_23_delete_collection() {

        let command = ManagerCommands::Delete {
            collection: "test3".to_owned(),
            endpoint: "".to_owned(),
            yes: true
        };

        let result = command.run(); 

        assert!(result.is_ok());

    }
}

use crate::commands::manager::ManagerCommands;
use crate::commands::request::RequestCommands;
use colored::Colorize;

pub async fn run_tests(collection_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let collections = ManagerCommands::load_collections().unwrap_or_default();
    let collection = collections.iter().find(|col| col.name == collection_name)
        .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;

    if let Some(requests) = &collection.requests {
        for request in requests {
            let command = ManagerCommands::get_endpoint_command(collection_name, &request.name)
                .ok_or_else(|| format!("Endpoint '{}' not found in collection '{}'", request.name, collection_name))?;

            let stdin_input = Vec::new();
            // Run the request
            match command.execute_request(false, stdin_input, false).await {
                Ok(response) => {
                    // Print the test result in the same format as print_request_method
                    println!("[{}] {} - {}\n", command.to_string().bold().bright_yellow(),
                        response.url().to_string().bold().bright_white(), RequestCommands::colorize_status(response.status()));
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
