use crate::commands;


pub async fn run (collection: String, endpoint: String, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    
    if verbose {
        println!("Running collection '{}' with endpoint '{}'", collection, endpoint);
    }

    let command = commands::manager::get_endpoint_command(&collection, &endpoint)
        .ok_or_else(|| format!("Endpoint not found: {}/{}", collection, endpoint))?;

    commands::request::run(command, verbose).await?;

    Ok(())
}