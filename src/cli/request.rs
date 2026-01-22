//! CLI commands for making HTTP requests
//!
//! This module provides the command-line interface for making HTTP requests,
//! including progress bars, colored output, and interactive prompts.

use crate::core::http_client::{HttpClient, HttpMethod};
use crate::HttpResponse;
use clap::{Args, Subcommand};
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use infer;
use reqwest::multipart::Part;
use serde_json::Value;
use std::fmt;
use std::io::{self, Write};
use std::time::Duration;

#[derive(Args, Clone, Debug)]
pub struct RequestData {
    pub url: String,

    #[clap(
        short = 'H',
        long = "header",
        value_parser = RequestData::parse_header,
        value_name = "KEY:VALUE",
        num_args = 1..,
        required = false
    )]
    pub headers: Vec<(String, String)>,

    #[clap(short, long, default_value = "", required = false)]
    pub body: String,
}

impl RequestData {
    pub fn parse_header(s: &str) -> Result<(String, String), String> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid header format: '{}'. Use KEY:VALUE", s));
        }
        Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
    }
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

    fn print_request_headers(headers: &[(String, String)]) {
        println!("{}", "Request Headers:".to_string().bold().bright_blue());
        for (key, value) in headers.iter() {
            println!("  {}: {:?}", key.to_string().bright_white(), value);
        }
    }

    fn print_request_body(body: &str) {
        println!("{}", "Request Body:".to_string().bold().bright_blue());
        println!("{}", body.italic());
    }

    async fn print_request_response(
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

    fn prompt_missing_header_data(mut headers: Vec<(String, String)>) -> Vec<(String, String)> {
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

    fn prompt_missing_body_data(mut body: String) -> String {
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
    fn is_text_data(data: &[u8]) -> bool {
        std::str::from_utf8(data).is_ok()
    }

    pub async fn execute_request(
        &self,
        verbose: bool,
        stdin_input: Vec<u8>,
        stream: bool,
    ) -> Result<(HttpResponse, u128), Box<dyn std::error::Error>> {
        let data = self.get_data();

        let current_url = if !stream {
            Self::prompt_missing_body_data(data.url.clone())
        } else {
            data.url.clone()
        };

        let headers = if !stream {
            Self::prompt_missing_header_data(data.headers.clone())
        } else {
            data.headers.clone()
        };

        let is_text = Self::is_text_data(&stdin_input);
        let body = if stdin_input.is_empty() {
            Self::prompt_missing_body_data(data.body.clone())
        } else if is_text {
            // Convert to string for text processing
            let text = String::from_utf8_lossy(&stdin_input).to_string();
            Self::prompt_missing_body_data(text)
        } else {
            // Binary: skip text prompts, use as-is (but reqwest body will handle bytes)
            String::new() // Placeholder; we'll use bytes directly in the request
        };

        let part = if !stream && !stdin_input.is_empty() && !is_text {
            // Binary data from stdin
            let kind = infer::get(&stdin_input).ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unknown file type",
                ))
            })?;
            let mime_type = kind.mime_type();
            let extension = kind.extension();
            let filename = format!("file.{}", extension);
            Part::bytes(stdin_input.clone())
                .file_name(filename)
                .mime_str(mime_type)?
        } else if !stream && !stdin_input.is_empty() && is_text {
            // Text data from stdin
            Part::text(String::from_utf8_lossy(&stdin_input).to_string())
        } else {
            // Use body string
            Part::bytes(body.clone().into_bytes())
        };

        if verbose && !stream {
            Self::print_request_headers(&headers);
            Self::print_request_body(body.as_str());
        }

        let client = HttpClient::new()
            .with_follow_redirects(false)
            .with_timeout(Duration::from_secs(120));

        let method = match self {
            Self::Get { .. } => HttpMethod::Get,
            Self::Post { .. } => HttpMethod::Post,
            Self::Put { .. } => HttpMethod::Put,
            Self::Delete { .. } => HttpMethod::Delete,
            Self::Patch { .. } => HttpMethod::Patch,
        };

        let pb = ProgressBar::new_spinner();

        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} {elapsed} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        pb.enable_steady_tick(Duration::from_millis(80));
        pb.set_message("Executing Request...");

        let start = std::time::Instant::now();

        let resp = if stream {
            let body_bytes = if !stdin_input.is_empty() {
                stdin_input
            } else {
                body.clone().into_bytes()
            };
            client
                .request(method, &current_url)
                .headers(headers.into_iter().collect())
                .body_bytes(body_bytes)
                .send_streaming(|chunk| {
                    std::io::stdout().write_all(&chunk)?;
                    std::io::stdout().flush().unwrap();
                    Ok(())
                })
                .await
        } else if is_text {
            client
                .request(method, &current_url)
                .headers(headers.into_iter().collect())
                .body(String::from_utf8_lossy(&stdin_input).as_ref())
                .send()
                .await
        } else {
            client
                .request(method, &current_url)
                .headers(headers.into_iter().collect())
                .send_multipart(part)
                .await
        };

        let elapsed = start.elapsed().as_millis();

        match resp {
            Ok(response) => {
                pb.finish_with_message("Request completed");
                Ok((response, elapsed))
            }
            Err(err) => {
                pb.finish_with_message("Request failed");
                Err(Box::new(err))
            }
        }
    }

    pub async fn run(
        &self,
        verbose: bool,
        stdin_input: Vec<u8>,
        stream: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = Self::execute_request(self, verbose, stdin_input, stream).await;

        match response {
            Ok((resp, elapsed)) => {
                if verbose && !stream {
                    println!("{:?}", resp.version);
                    self.print_request_method(&resp.url, resp.status, elapsed);
                }
                Self::print_request_response(&resp, verbose, stream).await
            }
            Err(err) => Err(err),
        }
    }
}
