//! CLI commands for managing collections and endpoints
//!
//! This module provides the command-line interface for managing API collections,
//! delegating the actual work to the core CollectionManager.

use std::fmt;

use crate::core::collection_manager::CollectionManager;
use crate::helper;
use crate::models::collection::Method;
use clap::Subcommand;
use colored::Colorize;

use super::request::{RequestCommands, RequestData};

#[derive(Clone, Subcommand)]
pub enum ManagerCommands {
    #[clap(about = "List collections and endpoints")]
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
    #[clap(about = "Update a collection or endpoint headers and body")]
    Update {
        collection: String,

        #[clap(short = 'e', long, default_value = "", required = false)]
        endpoint: String,

        #[clap(short = 'u', long, default_value = "", required = false)]
        url: String,

        #[clap(
            short = 'H',
            long = "header",
            value_parser = RequestData::parse_header,
            value_name = "KEY:VALUE",
            num_args = 1..,
            required = false
        )]
        headers: Vec<(String, String)>,

        #[clap(short = 'b', long, default_value = "", required = false)]
        body: String,
    },
    #[clap(about = "Delete a collection or endpoint")]
    Delete {
        collection: String,

        #[clap(short = 'e', long, default_value = "", required = false)]
        endpoint: String,

        #[clap(short, long, default_value = "false")]
        yes: bool,
    },
    #[clap(about = "Copy a collection or endpoint")]
    Copy {
        collection: String,

        #[clap(short = 'e', long, default_value = "", required = false)]
        endpoint: String,

        #[clap(short = 'c', long, default_value = "false", required = false)]
        to_col: bool,

        new_name: String,
    },
    #[clap(about = "Add a new collection")]
    Col {
        name: String,
        url: String,

        #[clap(
            short = 'H',
            long = "header",
            value_parser = RequestData::parse_header,
            value_name = "KEY:VALUE",
            num_args = 1..,
            required = false
        )]
        headers: Vec<(String, String)>,
    },
    #[clap(about = "Add a new endpoint to a collection")]
    Endpoint {
        collection: String,
        name: String,
        path: String,

        #[clap(short = 'm', long, default_value = "GET")]
        method: String,

        #[clap(
            short = 'H',
            long = "header",
            value_parser = RequestData::parse_header,
            value_name = "KEY:VALUE",
            num_args = 1..,
            required = false
        )]
        headers: Vec<(String, String)>,

        #[clap(short = 'b', long, default_value = "", required = false)]
        body: String,
    },
}

impl fmt::Display for ManagerCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagerCommands::List {
                col,
                endpoint,
                quiet,
                verbose,
            } => write!(
                f,
                "List Command: col: '{}', endpoint: '{}', quiet: {}, verbose: {}",
                col, endpoint, quiet, verbose
            ),
            ManagerCommands::Update {
                collection,
                endpoint,
                url: _,
                headers,
                body,
            } => {
                write!(
                    f,
                    "Update Command: collection: '{}', endpoint: '{}', headers: {:?}, body: '{}'",
                    collection, endpoint, headers, body
                )
            }
            ManagerCommands::Delete {
                collection,
                endpoint,
                yes,
            } => {
                write!(
                    f,
                    "Delete Command: collection: '{}', endpoint: '{}', yes: {}",
                    collection, endpoint, yes
                )
            }
            ManagerCommands::Copy {
                collection,
                endpoint,
                to_col,
                new_name,
            } => {
                write!(
                    f,
                    "Copy Command: collection: '{}', endpoint: '{}', To Col {}, new_name: '{}'",
                    collection, endpoint, to_col, new_name
                )
            }
            ManagerCommands::Col { name, url, headers } => {
                write!(
                    f,
                    "Col Command: name: '{}', url: '{}', headers: {:?}",
                    name, url, headers
                )
            }
            ManagerCommands::Endpoint {
                collection,
                name,
                path,
                method,
                headers,
                body,
            } => {
                write!(f, "Endpoint Command: collection: '{}', name: '{}', path: '{}', method: '{}', headers: {:?}, body: '{}'",
                    collection, name, path, method, headers, body)
            }
        }
    }
}

impl ManagerCommands {
    /// Get the default collection manager
    fn get_manager() -> CollectionManager {
        CollectionManager::default()
    }

    /// Get a RequestCommands for running an endpoint from a collection
    pub fn get_endpoint_command(collection: &str, endpoint: &str) -> Option<RequestCommands> {
        let manager = Self::get_manager();
        let col = manager.get_collection(collection).ok()?;
        let req = manager.get_endpoint(collection, endpoint).ok()?;

        let data = RequestData {
            url: format!("{}{}", col.url, req.endpoint),
            headers: manager
                .get_endpoint_headers(collection, endpoint)
                .unwrap_or_default(),
            body: req.body.clone().unwrap_or_default(),
        };

        Some(match req.method {
            Method::Get => RequestCommands::Get { data },
            Method::Post => RequestCommands::Post { data },
            Method::Delete => RequestCommands::Delete { data },
            Method::Patch => RequestCommands::Patch { data },
            Method::Put => RequestCommands::Put { data },
        })
    }

    pub fn run(&self) -> Result<String, Box<dyn std::error::Error>> {
        let manager = Self::get_manager();

        match self {
            // List collections and endpoints
            Self::List {
                col,
                endpoint,
                quiet,
                verbose,
            } => {
                let collections = manager.load_collections()?;
                if collections.is_empty() {
                    return Err("No collections found.".into());
                } else {
                    for collection in collections {
                        if !col.is_empty() && &collection.name != col {
                            continue;
                        }
                        println!(
                            "[{}] - {}",
                            collection.name.bright_magenta(),
                            collection.url
                        );
                        if *quiet {
                            continue;
                        }
                        if !collection.headers.is_empty() {
                            println!("  Headers:");
                            for (key, value) in &collection.headers {
                                println!("  {}: {}", key.bright_cyan(), value.bright_cyan());
                            }
                        }
                        if let Some(requests) = collection.requests {
                            for request in requests {
                                if !endpoint.is_empty() && &request.name != endpoint {
                                    continue;
                                }
                                println!(
                                    "  [{}] {} - {} - {} - {}",
                                    request.name.bright_yellow(),
                                    request.method.to_string().bright_green(),
                                    request.endpoint.bright_white(),
                                    request.headers.len(),
                                    request.body.as_ref().map_or(0, |b| b.len())
                                );
                                if *verbose {
                                    // check if headers present
                                    if !request.headers.is_empty() {
                                        println!("    Headers:");
                                        for (key, value) in &request.headers {
                                            println!(
                                                "    {}: {}",
                                                key.bright_cyan(),
                                                value.bright_cyan()
                                            );
                                        }
                                    }
                                    // check if body present
                                    if request.body.is_some() {
                                        println!("    Body:");
                                        if let Some(body) = &request.body {
                                            println!("    {}", body.bright_cyan());
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Delete a collection or endpoint
            Self::Delete {
                collection,
                endpoint,
                yes,
            } => {
                if endpoint.is_empty() {
                    // Deleting a collection
                    println!("Deleting collection '{}'", collection);
                    let confirm = if !yes {
                        helper::confirm("Are you sure you want to delete this collection?")
                    } else {
                        true
                    };
                    if confirm {
                        manager.delete_collection(collection)?;
                        println!("Collection deleted successfully!");
                    } else {
                        return Err("Deletion cancelled.".into());
                    }
                } else {
                    // Deleting an endpoint
                    println!("Deleting endpoint '{}'", endpoint);
                    let confirm = if !yes {
                        helper::confirm("Are you sure you want to delete this endpoint?")
                    } else {
                        true
                    };
                    if confirm {
                        manager.delete_endpoint(collection, endpoint)?;
                        println!("Endpoint deleted successfully!");
                    } else {
                        return Err("Deletion cancelled.".into());
                    }
                }
            }

            // Copy a collection or endpoint
            Self::Copy {
                collection,
                endpoint,
                to_col,
                new_name,
            } => {
                if endpoint.is_empty() {
                    // Copy collection
                    manager.copy_collection(collection, new_name)?;
                } else if *to_col {
                    // Copy endpoint to another collection
                    manager.copy_endpoint(collection, endpoint, new_name, Some(new_name))?;
                } else {
                    // Copy endpoint with new name in same collection
                    manager.copy_endpoint(collection, endpoint, new_name, None)?;
                }
                println!("Copy command successful!");
            }

            // Update a collection or endpoint headers and body
            Self::Update {
                collection,
                endpoint,
                url,
                headers,
                body,
            } => {
                if endpoint.is_empty() {
                    // Update collection
                    let url_opt = if url.is_empty() { None } else { Some(url.as_str()) };
                    let headers_opt = if headers.is_empty() {
                        None
                    } else {
                        Some(headers.clone())
                    };
                    manager.update_collection(collection, url_opt, headers_opt)?;
                } else {
                    // Update endpoint
                    let url_opt = if url.is_empty() { None } else { Some(url.as_str()) };
                    let headers_opt = if headers.is_empty() {
                        None
                    } else {
                        Some(headers.clone())
                    };
                    let body_opt = if body.is_empty() {
                        Some(String::new()) // Empty body clears the existing body
                    } else {
                        Some(body.clone())
                    };
                    manager.update_endpoint(collection, endpoint, url_opt, headers_opt, body_opt)?;
                }
                println!("Collection updated successfully!");
            }

            // Add a new collection or update an existing one
            Self::Col { name, url, headers } => {
                let exists = manager.get_collection(name).is_ok();
                manager.add_collection(name, url, headers.clone())?;
                if exists {
                    eprintln!("Collection with name '{}' already exists.", name);
                    println!("Collection updated successfully!");
                } else {
                    println!("Collection added successfully!");
                }
            }

            // Add a new endpoint to a collection or update an existing one
            Self::Endpoint {
                collection,
                name,
                path,
                method,
                headers,
                body,
            } => {
                let method: Method = method.to_uppercase().parse().map_err(|_| {
                    format!("Invalid HTTP method: {}", method)
                })?;
                
                let body_opt = if body.trim().is_empty() {
                    None
                } else {
                    Some(body.clone())
                };

                manager.add_endpoint(collection, name, path, method, headers.clone(), body_opt)?;
                println!("Endpoint added successfully!");
            }
        }

        Ok("".to_string())
    }
}
