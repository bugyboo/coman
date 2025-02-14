
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
            headers: vec![],
            body: "".to_owned(),
        };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[test]
    fn test_03_list_collections() {
        let command = ManagerCommands::List { col: "test".to_owned(), verbose: true };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_04_run_req() {
        let collection = "test";
        let endpoint = "ver";
        let verbose = true;
        let stdin_input = "";

        let result = Commands::run_request(
            collection,
            endpoint,
            &verbose,
            stdin_input,
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

    #[tokio::test]
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

        let result = command.run(true, "".to_owned()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_10_req_post() {

        let request_data = RequestData {
            url: "http://localhost:8080/login".to_owned(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: format!("{{\"username\": \"test\", \"password\": \"test\"}}"),
        };

        let command = RequestCommands::Post {
            data: request_data
        };

        let result = command.run(true, "".to_owned()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_11_req_put() {

        let request_data = RequestData {
            url: "http://localhost:8080/user".to_owned(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: format!("{{\"name\": \"test test\"}}"),
        };

        let command = RequestCommands::Put {
            data: request_data
        };

        let result = command.run(true, "".to_owned()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_12_req_delete() {

        let request_data = RequestData {
            url: "http://localhost:8080/user?id=test".to_owned(),
            headers: vec![],
            body: "".to_owned(),
        };

        let command = RequestCommands::Delete {
            data: request_data
        };

        let result = command.run(true, "".to_owned()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_13_req_patch() {

        let request_data = RequestData {
            url: "http://localhost:8080/user?id=test".to_owned(),
            headers: vec![],
            body: format!("{{\"name\": \"test test\"}}"),
        };

        let command = RequestCommands::Patch {
            data: request_data
        };

        let result = command.run(true, "".to_owned()).await;

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
    fn test_15_create_endpoint_wuth_header_and_body() {
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
    }

    #[test]
    fn test_16_list_collections() {
        let command = ManagerCommands::List { col: "test2".to_owned(), verbose: true };

        let result = command.run();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_17_run_req() {
        let collection = "test2";
        let endpoint = "ver";
        let verbose = true;
        let stdin_input = "";

        let result = Commands::run_request(
            collection,
            endpoint,
            &verbose,
            stdin_input,
        );

        assert!(result.await.is_ok());
    }

    #[test]
    fn test_18_delete_collection() {

        let command = ManagerCommands::Delete {
            collection: "test2".to_owned(),
            endpoint: "".to_owned(),
            yes: true
        };

        let result = command.run();

        assert!(result.is_ok());
    }
}
