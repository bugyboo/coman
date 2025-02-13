use crate::commands;

use super::request::RequestCommands;


pub async fn run (collection: String, endpoint: String, verbose: bool, stdin_input: String) -> Result<(), Box<dyn std::error::Error>> {
    
    if verbose {
        println!("Running collection '{}' with endpoint '{}'", collection, endpoint);
    }

    let mut command = commands::manager::get_endpoint_command(&collection, &endpoint)
        .ok_or_else(|| format!("Endpoint not found: {}/{}", collection, endpoint))?;

    // Access command data using pattern matching
    let data = match &mut command {
        RequestCommands::Get { data }
        | RequestCommands::Post { data }
        | RequestCommands::Put { data }
        | RequestCommands::Delete { data }
        | RequestCommands::Patch { data } => data,
    };

    if !stdin_input.is_empty() {
        data.body = stdin_input;
    }    

    commands::request::run(command, verbose).await?;

    Ok(())
}