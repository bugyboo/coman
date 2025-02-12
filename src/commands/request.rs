use std::io::{self, Write};
use std::fmt;
use clap::{Args, Subcommand};
use colored::{ColoredString, Colorize};
use reqwest::{redirect::Policy, ClientBuilder, StatusCode};
use serde_json::Value;

use crate::helper::{build_header_map, parse_header};

#[derive(Args, Clone, Debug)]
pub struct RequestData {
    pub url: String,

    #[clap(
        short = 'H', 
        long = "header",
        value_parser = parse_header,
        value_name = "KEY:VALUE",
        num_args = 1..,
        required = false
    )]
    pub headers: Vec<(String, String)>,

    #[clap(short, long, default_value = "", required = false)]      
    pub body: String     
}

#[derive(Subcommand, Clone, Debug)]
pub enum RequestCommands {
    Get { 
        #[clap(flatten)]
        data: RequestData,
    },
    Post {
        #[clap(flatten)]
        data: RequestData,
    },
    Put {
        #[clap(flatten)]
        data: RequestData,
    },
    Delete {
        #[clap(flatten)]
        data: RequestData,
    },
    Patch {
        #[clap(flatten)]
        data: RequestData,
    },
}

impl fmt::Display for RequestCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestCommands::Get { .. } => write!(f, "GET"),
            RequestCommands::Post { .. } => write!(f, "POST"),
            RequestCommands::Put { .. } => write!(f, "PUT"),
            RequestCommands::Delete { .. } => write!(f, "DELETE"),
            RequestCommands::Patch { .. } => write!(f, "PATCH"),
        }
    }
}

fn print_request_method(method: RequestCommands, url: &str, status: StatusCode ) {
    println!("\n[{}] {} - {}\n", method.to_string().bold().bright_yellow(), 
        url.to_string().bold().bright_white(), colorize_status( status ) );
}

fn print_request_headers(headers: Vec<(String, String)>) {
    println!("{}", "Request Headers:".to_string().bold().bright_blue());
    for (key, value) in headers.iter() {
        println!("  {}: {:?}", key.to_string().bright_white(), value);
    }
}

fn print_request_body(body: &str) {
    println!("{}", "Request Body:".to_string().bold().bright_blue());
    println!("{}", body.italic());
}

async fn print_request_response(response: reqwest::Response, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("{}", "Response Headers:".to_string().bold().bright_blue());
        for (key, value) in response.headers().iter() {
            println!("  {}: {:?}", key.to_string().bright_white(), value);
        }
    }
    
    let body = response.text().await?;

    if verbose {
        println!("\n{}", "Response Body:".to_string().bold().bright_blue());
    }

    // Try parsing the body as JSON
    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        let pretty = serde_json::to_string_pretty(&json)?;
        println!("{}", pretty.green() );
    } else {
        println!("{}", body.italic());
    }

    Ok(())
}

fn colorize_status(status: StatusCode) -> ColoredString {
    match status.as_u16() {
        200..=299 => status.to_string().bold().bright_green(),
        300..=399 => status.to_string().bold().bright_yellow(),
        400..=499 => status.to_string().bold().bright_red(),
        500..=599 => status.to_string().bold().bright_magenta(),
        _ => status.to_string().white(),
    }
}

fn prompt_missing_header_data(mut headers: Vec<(String, String)>) -> Vec<(String, String)> {
    for header in headers.iter_mut() {
        if header.1.contains(":?") {
            print!("Header value for key '{}' is missing data. Please provide the correct value: ", header.0);
            io::stdout().flush().ok();
            let mut new_value = String::new();
            std::io::stdin().read_line(&mut new_value).expect("Failed to read header value");
            header.1 = new_value.trim().to_string();
        }
    }
    headers
}

fn prompt_missing_body_data(mut body: String) -> String {
    while let Some(idx) = body.find(":?") {
        print!("Missing data at position {} - {}. Please provide the correct value: ", idx, body);
        io::stdout().flush().ok();
        let mut replacement = String::new();
        std::io::stdin()
            .read_line(&mut replacement)
            .expect("Failed to read body placeholder");
        let replacement = replacement.trim();
        body.replace_range(idx..idx + 2, replacement);
    }
    body
}

async fn execute_request(command: RequestCommands, verbose: bool) -> Result<reqwest::Response, reqwest::Error> {

    let data = match &command {
        RequestCommands::Get { data }
        | RequestCommands::Post { data }
        | RequestCommands::Put { data }
        | RequestCommands::Delete { data }
        | RequestCommands::Patch { data } => data,
    };

    let current_url = prompt_missing_body_data(data.url.clone());
    let headers = prompt_missing_header_data(data.headers.clone());
    let headers = build_header_map(&headers);
    let body = prompt_missing_body_data(data.body.clone());

    if verbose {
        print_request_headers(headers.iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string())).collect());
        print_request_body(body.as_str());
    }

    let client = ClientBuilder::new()
    .redirect(Policy::none())
    .build()?;
    
    match command {
        RequestCommands::Get { .. } => {
            client.get(&current_url)
                .headers(headers)
                .send()
                .await
        },
        RequestCommands::Post { .. } => {
            client.post(&current_url)
                .headers(headers)
                .body(body)
                .send()
                .await
        },
        RequestCommands::Put { .. } => {
            client.put(&current_url)
                .headers(headers)
                .body(body)
                .send()
                .await
        },
        RequestCommands::Delete { .. } => {
            client.delete(&current_url)
                .headers(headers)
                .body(body)
                .send()
                .await
        },
        RequestCommands::Patch { .. } => {
            client.patch(&current_url)
                .headers(headers)
                .body(body)
                .send()
                .await
        },
    }
}

pub async fn run (command: RequestCommands, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {

    if verbose {
        println!("Verbose mode enabled");
    }

    let response = execute_request(command.clone(), verbose).await;

    match response {
        Ok(resp) => {
            if verbose {
                print_request_method(command, &resp.url().to_string(), resp.status());                
            }
            print_request_response(resp, verbose).await?;
        },
        Err(err) => {
            eprintln!("Request failed: {}", err);
        }
    }

    Ok(())
}