
use crate::commands::{self, request::RequestCommands};

pub async fn run (collection: String, endpoint: String) -> Result<(), Box<dyn std::error::Error>> {

    let command = commands::manager::get_endpoint_command(&collection, &endpoint)
        .ok_or_else(|| format!("Endpoint not found: {}/{}", collection, endpoint))?;

    let data = match &command {
        RequestCommands::Get { data }
        | RequestCommands::Post { data }
        | RequestCommands::Put { data }
        | RequestCommands::Delete { data }
        | RequestCommands::Patch { data } => data,
    };

    let headers_url = data
        .headers
        .iter()
        .map(|(key, value)| format!("-H \"{}: {}\"", key, value.to_string()))
        .collect::<Vec<_>>()
        .join(" ");

    let body_flag = if !data.body.is_empty() {
            format!("-b '{}'", data.body)
        } else {
            String::new()
        };

    println!("coman req -v {} {} {} {}", command.to_string().to_lowercase() , data.url, headers_url, body_flag);

    Ok(())
}