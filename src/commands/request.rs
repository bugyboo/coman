use std::io::{self, Write};
use std::fmt;
use clap::{Args, Subcommand};
use colored::{ColoredString, Colorize};
use reqwest::header::HeaderMap;
use reqwest::{redirect::Policy, ClientBuilder, StatusCode};
use serde_json::Value;
use futures::stream::StreamExt;

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
        Ok((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
        ))
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

    pub fn get_data(&self) -> &RequestData {  // assuming RequestData is the type of 'data'
        match self {
            Self::Get { data }
            | Self::Post { data }
            | Self::Put { data }
            | Self::Delete { data }
            | Self::Patch { data } => data,
        }
    }

    pub fn print_request_method(&self, url: &str, status: StatusCode ) {
        println!("\n[{}] {} - {}\n", self.to_string().bold().bright_yellow(),
            url.to_string().bold().bright_white(), Self::colorize_status( status ) );
    }

    fn print_request_headers(headers: &Vec<(String, String)>) {
        println!("{}", "Request Headers:".to_string().bold().bright_blue());
        for (key, value) in headers.iter() {
            println!("  {}: {:?}", key.to_string().bright_white(), value);
        }
    }

    fn print_request_body(body: &str) {
        println!("{}", "Request Body:".to_string().bold().bright_blue());
        println!("{}", body.italic());
    }

    async fn print_request_response(response: reqwest::Response, verbose: bool, stream: bool) -> Result<String, Box<dyn std::error::Error>> {
        if verbose {
            println!("{}", "Response Headers:".to_string().bold().bright_blue());
            for (key, value) in response.headers().iter() {
                println!("  {}: {:?}", key.to_string().bright_white(), value);
            }
        }

        if verbose {
            println!("\n{}", "Response Body:".to_string().bold().bright_blue());
        }

        if stream {
            // Get the stream of bytes
            let mut stream = response.bytes_stream();

            // Process each chunk as it arrives
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?; // Handle potential errors in the stream
                let text = String::from_utf8_lossy(&chunk); // Convert bytes to string
                print!("{}", text); // Print immediately as it arrives
                std::io::stdout().flush()?; // Ensure output is flushed to terminal
            }
        } else {

            let body = response.text().await?;
            //Try parsing the body as JSON
            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                let pretty = serde_json::to_string_pretty(&json)?;
                println!("{}", pretty.green() );
            } else {
                println!("{}", body.italic());
            }
        }

        Ok("".to_string())
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
                eprint!("Header value for key '{}' is missing data. Please provide the correct value: ", header.0);
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
            eprint!("Missing data at position {} - {}. Please provide the correct value: ", idx, body);
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

    pub fn build_header_map(headers: &[(String, String)]) -> HeaderMap {
        let mut header_map = HeaderMap::new();
        for (key, value) in headers {
            if let Ok(header_name) = key.parse::<reqwest::header::HeaderName>() {
                header_map.insert(header_name, value.parse().unwrap());
            }
        }
        header_map
    }

    async fn execute_request(&self, verbose: bool, stdin_input: String) -> Result<reqwest::Response, reqwest::Error> {

        let data = self.get_data();

        let current_url = Self::prompt_missing_body_data(data.url.clone());
        let headers = Self::prompt_missing_header_data(data.headers.clone());
        let body = if stdin_input.is_empty() {
            Self::prompt_missing_body_data(data.body.clone())
        } else {
            stdin_input
        };

        if verbose {
            Self::print_request_headers(&headers);
            Self::print_request_body(body.as_str());
        }

        let client = ClientBuilder::new()
        .redirect(Policy::none())
        .build()?;

        let headers = Self::build_header_map(&headers);

        match self {
            Self::Get { .. } => {
                client.get(&current_url)
                    .headers(headers)
                    .send()
                    .await
            },
            Self::Post { .. } => {
                client.post(&current_url)
                    .headers(headers)
                    .body(body)
                    .send()
                    .await
            },
            Self::Put { .. } => {
                client.put(&current_url)
                    .headers(headers)
                    .body(body)
                    .send()
                    .await
            },
            Self::Delete { .. } => {
                client.delete(&current_url)
                    .headers(headers)
                    .body(body)
                    .send()
                    .await
            },
            Self::Patch { .. } => {
                client.patch(&current_url)
                    .headers(headers)
                    .body(body)
                    .send()
                    .await
            },
        }
    }

    pub async fn run (&self, verbose: bool, stdin_input: String, stream: bool) -> Result<String, Box<dyn std::error::Error>> {

        let response = Self::execute_request(self, verbose, stdin_input).await;

        match response {
            Ok(resp) => {
                if verbose {
                    println!("{:?}", resp.version());
                    self.print_request_method(&resp.url().to_string(), resp.status());
                }
                Self::print_request_response(resp, verbose, stream).await
            },
            Err(err) => {
                Err(Box::new(err))
            }
        }
    }

}
