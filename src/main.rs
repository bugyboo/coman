use std::{fmt, io};
use std::io::Read;

use clap::{Parser, Subcommand};

mod commands;
mod models;
mod helper;
mod test;

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

    #[command(about = "List APIs Collections")]
    List {
        #[clap(short = 'c', long = "col", default_value = "", required = false)]
        col: String,

        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    #[command(about = "Managing APIs")]
    Man {
        #[command(subcommand)]
        command: ManagerCommands,
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

impl fmt::Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Commands::List { col, verbose } => write!(f, "List Command: {} - {}", col, verbose),
            Commands::Man { command } => write!(f, "Man Command: {}", command),
            Commands::Req { command, verbose } => {
                write!(f, "Req Command: {} (verbose: {})", command, verbose)
            },
            Commands::Run { collection, endpoint, verbose } => {
                write!(f, "Run Command: collection: '{}', endpoint: '{}', verbose: {}", collection, endpoint, verbose)
            },
            Commands::Url { collection, endpoint } => {
                write!(f, "Url Command: collection: '{}', endpoint: '{}'", collection, endpoint)
            },
        }
    }
}

impl Commands {

    pub fn run_url (collection: &str, endpoint: &str) -> Result<String, Box<dyn std::error::Error>> {

        let command = ManagerCommands::get_endpoint_command(&collection, &endpoint)
            .ok_or_else(|| format!("Endpoint not found: {}/{}", collection, endpoint))?;

        let data = command.get_data();

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

        let url = format!("{} {} {} {}", command.to_string().to_lowercase() , data.url, headers_url, body_flag);
        println!("coman req -v {}", url);

        Ok(url)
    }

    pub async fn run_request (collection: &str, endpoint: &str, verbose: &bool, stdin_input: &str) -> Result<String, Box<dyn std::error::Error>> {

        if *verbose {
            println!("Running collection '{}' with endpoint '{}'", collection, endpoint);
        }

        let command = ManagerCommands::get_endpoint_command(&collection, &endpoint)
            .ok_or_else(|| format!("Endpoint not found: {}/{}", collection, endpoint))?;

        command.run(*verbose, stdin_input.to_owned()).await
    }

    async fn run(&self, stdin_input: String) -> Result<String, Box<dyn std::error::Error>> {

        match self {
            Commands::List { col, verbose } => {
                ManagerCommands::List { col: col.clone(), verbose: *verbose }.run()
            },
            Commands::Man { command } => {
                command.run()
            },
            Commands::Req { command, verbose } => {
                command.run(*verbose, stdin_input).await
            },
            Commands::Run { collection, endpoint, verbose } => {
                Self::run_request(collection, endpoint, verbose, &stdin_input).await
            },
            Commands::Url { collection, endpoint } => {
                Self::run_url(collection, endpoint)
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut stdin_input = String::new();
    if !atty::is(atty::Stream::Stdin) {
        io::stdin().read_to_string(&mut stdin_input)?;
    }

    let args = Cli::parse();

    let result = args.command.run(stdin_input).await;

    match result {
        Ok(_s) => {},
        Err(e) => {
            eprintln!("Failed to run command : {} \n {}", args.command, e);
            std::process::exit(1);
        }
    }
    Ok(())

}
