use std::io::{self, Read};

use clap::{Parser, Subcommand};

mod commands;
mod models;
mod helper;

use commands::manager::ManagerCommands;
use commands::request::RequestCommands;

#[derive(Parser)]
#[command(name = "coman", about = "Simple API Manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {

    #[command(about = "Managing APIs")]
    Man {
        #[command(subcommand)]
        command: ManagerCommands,

        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    #[command(about = "Sending requests")]
    Req {
        #[command(subcommand)]
        command: RequestCommands,

        #[clap(short, long, default_value = "false")]
        verbose: bool,        
    },

    #[command(about = "Running collections endpoints")]
    Run {
        collection: String,
        endpoint: String,

        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    #[command(about = "Print request URL with headers and body")]
    Url {
        collection: String,
        endpoint: String,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {   

    let mut stdin_input = String::new();
    if !atty::is(atty::Stream::Stdin) {
        io::stdin().read_to_string(&mut stdin_input)?;
    }     
    
    let args = Cli::parse();
    
    let result = match args.command {
        Commands::Man { command, verbose } => {
            commands::manager::run(command, verbose)
        },
        Commands::Req { command, verbose } => {
            commands::request::run(command, verbose).await                  
        },
        Commands::Run { collection, endpoint, verbose } => {
            commands::run::run(collection, endpoint, verbose, stdin_input).await
        },
        Commands::Url { collection, endpoint } => {
            commands::url::run(collection, endpoint).await
        }

    };

    result

}
