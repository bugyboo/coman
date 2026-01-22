//! CLI commands for making HTTP requests
//!
//! This module provides the command-line interface for making HTTP requests,
//! including progress bars, colored output, and interactive prompts.

use crate::cli::request_data::RequestData;
use crate::HttpResponse;
use clap::Subcommand;
use colored::{ColoredString, Colorize};
use serde_json::Value;
use std::fmt;
use std::io::{self, Write};

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
            Self::Get { .. } => write!(f, "GET"),
            Self::Post { .. } => write!(f, "POST"),
            Self::Put { .. } => write!(f, "PUT"),
            Self::Delete { .. } => write!(f, "DELETE"),
            Self::Patch { .. } => write!(f, "PATCH"),
        }
    }
}

impl RequestCommands {
    pub fn get_data(&self) -> &RequestData {
        // assuming RequestData is the type of 'data'
        match self {
            Self::Get { data }
            | Self::Post { data }
            | Self::Put { data }
            | Self::Delete { data }
            | Self::Patch { data } => data,
        }
    }

    pub fn print_request_method(&self, url: &str, status: u16, elapsed: u128) {
        println!(
            "\n[{}] {} - {} ({} ms)\n",
            self.to_string().bold().bright_yellow(),
            url.to_string().bold().bright_white(),
            Self::colorize_status(status.to_string().parse().unwrap()),
            elapsed
        );
    }

    pub fn print_request_headers(headers: &[(String, String)]) {
        println!("{}", "Request Headers:".to_string().bold().bright_blue());
        for (key, value) in headers.iter() {
            println!("  {}: {:?}", key.to_string().bright_white(), value);
        }
    }

    pub fn print_request_body(body: &str) {
        println!("{}", "Request Body:".to_string().bold().bright_blue());
        println!("{}", body.italic());
    }

    pub async fn print_request_response(
        response: &HttpResponse,
        verbose: bool,
        stream: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if verbose && !stream {
            println!("{}", "Response Headers:".to_string().bold().bright_blue());
            for (key, value) in response.headers.iter() {
                println!("  {}: {:?}", key.to_string().bright_white(), value);
            }
            println!("\n{}", "Response Body:".to_string().bold().bright_blue());
        }

        if !stream {
            //Try parsing the body as JSON
            if let Ok(json) = response.json::<Value>() {
                let pretty = serde_json::to_string_pretty(&json)?;
                println!("{}", pretty.green());
            } else {
                println!("{}", response.body.italic());
            }
        }

        Ok(())
    }

    pub fn colorize_status(status: u16) -> ColoredString {
        match status {
            200..=299 => status.to_string().bold().bright_green(),
            300..=499 => status.to_string().bold().bright_yellow(),
            500..=599 => status.to_string().bold().bright_red(),
            _ => status.to_string().white(),
        }
    }

    pub fn prompt_missing_header_data(mut headers: Vec<(String, String)>) -> Vec<(String, String)> {
        for header in headers.iter_mut() {
            if header.1.contains(":?") {
                eprint!(
                    "Header value for key '{}' is missing data. Please provide the correct value: ",
                    header.0
                );
                io::stdout().flush().ok();
                let mut new_value = String::new();
                std::io::stdin()
                    .read_line(&mut new_value)
                    .expect("Failed to read header value");
                header.1 = new_value.trim().to_string();
            }
        }
        headers
    }

    pub fn prompt_missing_body_data(mut body: String) -> String {
        while let Some(idx) = body.find(":?") {
            eprint!(
                "Missing data at position {} - {}. Please provide the correct value: ",
                idx, body
            );
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

    /// Checks if the Vec<u8> is valid UTF-8 (likely text) or not (binary).
    pub fn is_text_data(data: &[u8]) -> bool {
        std::str::from_utf8(data).is_ok()
    }
}
