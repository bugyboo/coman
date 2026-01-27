use std::{io::Write, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::multipart::Part;

use crate::{cli::request::RequestCommands, HttpClient, HttpMethod, HttpResponse};

impl RequestCommands {
    pub async fn execute_request(
        &self,
        verbose: bool,
        stdin_input: Vec<u8>,
        stream: bool,
    ) -> Result<(HttpResponse, u128), Box<dyn std::error::Error>> {
        let data = self.get_data();

        let current_url = if !stream {
            RequestCommands::prompt_missing_body_data(data.url.clone())
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
                    std::io::stdout().write_all(chunk)?;
                    std::io::stdout().flush().unwrap();
                    Ok(())
                })
                .await
        } else if is_text {
            let body_text = if !stdin_input.is_empty() {
                String::from_utf8_lossy(&stdin_input).to_string()
            } else {
                body
            };            
            client
                .request(method, &current_url)
                .headers(headers.into_iter().collect())
                .body(&body_text)
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
