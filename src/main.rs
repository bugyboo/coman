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

        #[clap(short = 'e', long = "endpoint", default_value = "", required = false)]
        endpoint: String,        

        #[clap(short = 'q', long = "quiet", default_value = "false")]
        quiet: bool,        

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

        #[clap(short, long, required = false, default_value = "false")]
        stream: bool,
    },

    #[command(about = "Running collections endpoints")]
    Run {
        collection: String,
        endpoint: String,

        #[clap(short, long, default_value = "false")]
        verbose: bool,

        #[clap(short, long, required = false, default_value = "false")]
        stream: bool,
    },

    #[command(about = "Print request URL with headers and body")]
    Url {
        collection: String,
        endpoint: String,
    },

    #[command(about = "Run tests")]
    Test {
        collection: String,
    },
}

impl fmt::Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Commands::List { col, endpoint, quiet, verbose } => write!(f, "List Command: {} - {} - {} - {}", col, endpoint, quiet, verbose),
            Commands::Man { command } => write!(f, "Man Command: {}", command),
            Commands::Req { command, verbose, stream} => {
                write!(f, "Req Command: {} (verbose: {}) (stream: {})", command, verbose, stream)
            },
            Commands::Run { collection, endpoint, verbose, stream } => {
                write!(f, "Run Command: collection: '{}', endpoint: '{}', verbose: {}, stream: {}",
                collection, endpoint, verbose, stream)
            },
            Commands::Url { collection, endpoint } => {
                write!(f, "Url Command: collection: '{}', endpoint: '{}'", collection, endpoint)
            },
            Commands::Test { collection } => {
                write!(f, "Test Command: collection: '{}'", collection)
            }
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

        let url = format!("{} '{}' {} {}", command.to_string().to_lowercase() , data.url, headers_url, body_flag);
        println!("coman req -v {}", url);

        Ok(url)
    }

    pub async fn run_request (collection: &str, endpoint: &str, verbose: &bool, stdin_input: &Vec<u8>, stream: &bool) -> Result<String, Box<dyn std::error::Error>> {

        if *verbose {
            println!("Running collection '{}' with endpoint '{}'", collection, endpoint);
        }

        let command = ManagerCommands::get_endpoint_command(&collection, &endpoint)
            .ok_or_else(|| format!("Endpoint not found: {}/{}", collection, endpoint))?;

        command.run(*verbose, stdin_input.to_owned(), *stream).await
    }

    async fn run(&self, stdin_input: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {

        match self {
            Commands::List { col, endpoint, quiet, verbose } => {
                ManagerCommands::List { col: col.clone(), endpoint: endpoint.clone(), verbose: *verbose, quiet: *quiet }.run()
            },
            Commands::Man { command } => {
                command.run()
            },
            Commands::Req { command, verbose, stream } => {
                command.run(*verbose, stdin_input, *stream).await
            },
            Commands::Run { collection, endpoint, verbose, stream } => {
                Self::run_request(collection, endpoint, verbose, &stdin_input, stream).await
            },
            Commands::Url { collection, endpoint } => {
                Self::run_url(collection, endpoint)
            },
            Commands::Test { collection } => {
                test::run_tests(collection).await
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut stdin_input = Vec::new();
    if !atty::is(atty::Stream::Stdin) {
        io::stdin().read_to_end(&mut stdin_input)?;
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
