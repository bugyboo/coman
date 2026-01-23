use std::io::{self, Read};

use clap::{CommandFactory, FromArgMatches, Parser};

use coman::cli::commands::Commands;
use coman::helper;

#[derive(Parser)]
#[command(name = "coman", about = "Simple API Manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin_input = Vec::new();
    if !atty::is(atty::Stream::Stdin) {
        io::stdin().read_to_end(&mut stdin_input)?;
    }

    let file_path = helper::get_file_path();

    let version: &'static str = Box::leak(
        format!(
            "version: {}\n (data file: {})",
            env!("CARGO_PKG_VERSION"),
            file_path
        )
        .into_boxed_str(),
    );

    let args = Cli::command().version(version).get_matches();

    let cli = Cli::from_arg_matches(&args)?;

    let result = cli.command.run(stdin_input).await;

    match result {
        Ok(_s) => {}
        Err(e) => {
            eprintln!("Failed to run command : {} \n {}", cli.command, e);
            std::process::exit(1);
        }
    }
    Ok(())
}
