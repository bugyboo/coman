use std::{collections::HashMap, fmt};

use clap::Subcommand;
use colored::Colorize;
use crate::{helper, models};

use super::request::{RequestCommands, RequestData};

#[derive(Clone)]
#[derive(Subcommand)]
pub enum ManagerCommands {
    #[clap(about = "List collections and endpoints")]
    List {
        #[clap(short = 'c', long = "col", default_value = "", required = false)]
        col: String,

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

        #[clap(
            short = 'b', long,
            default_value = "",
            required = false
        )]
        body: String,
    },
    #[clap(about = "Delete a collection or endpoint")]
    Delete {
        collection: String,

        #[clap(short = 'e', long, default_value = "", required = false)]
        endpoint: String,

        #[clap(short, long, default_value = "false")]
        yes: bool
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

        #[clap(
            short = 'm', long,
            default_value = "GET",
        )]
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

        #[clap(
            short = 'b', long,
            default_value = "",
            required = false
        )]
        body: String,
    },
}

impl fmt::Display for ManagerCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagerCommands::List { col, verbose } => write!(f, "List Command: col: '{}', verbose: {}", col, verbose),
            ManagerCommands::Update { collection, endpoint, url: _, headers, body } => {
                write!(f, "Update Command: collection: '{}', endpoint: '{}', headers: {:?}, body: '{}'",
                    collection, endpoint, headers, body)
            },
            ManagerCommands::Delete { collection, endpoint, yes } => {
                write!(f, "Delete Command: collection: '{}', endpoint: '{}', yes: {}", collection, endpoint, yes)
            },
            ManagerCommands::Copy { collection, endpoint,to_col, new_name } => {
                write!(f, "Copy Command: collection: '{}', endpoint: '{}', To Col {}, new_name: '{}'", collection, endpoint, to_col, new_name)
            },
            ManagerCommands::Col { name, url, headers } => {
                write!(f, "Col Command: name: '{}', url: '{}', headers: {:?}", name, url, headers)
            },
            ManagerCommands::Endpoint { collection, name, path, method, headers, body } => {
                write!(f, "Endpoint Command: collection: '{}', name: '{}', path: '{}', method: '{}', headers: {:?}, body: '{}'",
                    collection, name, path, method, headers, body)
            },
        }
    }
}

impl ManagerCommands {

    pub fn load_collections() -> Result<Vec<models::collection::Collection>, Box<dyn std::error::Error>> {
        match helper::read_json_from_file() {
            Ok(c) => Ok(c),
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        Ok(Vec::new())
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn get_endpoint_command(collection: &str, endpoint: &str) -> Option<RequestCommands> {
        let collections = Self::load_collections().unwrap_or_default();
        collections.iter().find(|col| col.name == collection).and_then(|col| {
            col.requests.as_ref()?.iter().find(|req| req.name == endpoint).map(|req| {
                let data = RequestData {
                    url: format!("{}{}", col.url, req.endpoint),
                    headers: {
                        let mut merged = std::collections::HashMap::new();
                        for (k, v) in &col.headers {
                            merged.insert(k.clone(), v.clone());
                        }
                        for (k, v) in &req.headers {
                            merged.insert(k.clone(), v.clone());
                        }
                        merged.into_iter().collect()
                    },
                    body: req.body.clone().unwrap_or_default()
                };
                match req.method {
                    models::collection::Method::GET => RequestCommands::Get { data },
                    models::collection::Method::POST => RequestCommands::Post { data },
                    models::collection::Method::DELETE => RequestCommands::Delete { data },
                    models::collection::Method::PATCH => RequestCommands::Patch { data },
                    models::collection::Method::PUT => RequestCommands::Put { data },
                }
            })
        })
    }

    fn merge_headers(existing: Vec<(String, String)>, new_headers: &[(String, String)]) -> Vec<(String, String)> {
        let mut merged: HashMap<String, String> = existing.into_iter().collect();
        for (key, value) in new_headers.iter() {
            if merged.contains_key(key) {
                if value == "" {
                    merged.remove(key);
                } else {
                    merged.entry(key.clone()).and_modify(|v| *v = value.clone());
                }
            } else {
                merged.insert(key.clone(), value.clone());
            }
        }
        merged.into_iter().collect()
    }

    pub fn run(&self) -> Result<String, Box<dyn std::error::Error>> {

        match self {

            // List collections and endpoints
            Self::List { col, verbose } => {
                let collections = Self::load_collections()?;
                if collections.is_empty() {
                    return Err("No collections found.".into());
                } else {
                    for collection in collections {
                        if col != "" && &collection.name != col {
                            continue;
                        }
                        println!("[{}] - {}", collection.name.bright_yellow(), collection.url);
                        if !collection.headers.is_empty() {
                            println!("  Headers:");
                            for (key, value) in &collection.headers {
                                println!("  {}: {}", key.bright_cyan(), value.bright_cyan());
                            };
                        }
                        if let Some(requests) = collection.requests {
                            for request in requests {
                                println!("  [{}] {} - {} - {} - {}",
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
                                            println!("    {}: {}", key.bright_cyan(), value.bright_cyan());
                                        };
                                    }
                                    // check if body present
                                    if !request.body.is_none() {
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
            },

            // Delete a collection or endpoint
            Self::Delete { collection, endpoint , yes} => {
                let mut collections = Self::load_collections()?;

                if  let Some(col) = collections.iter_mut().find(|c| c.name == *collection) {
                    if endpoint.is_empty() {
                        println!("Deleting collection '{}'", collection);
                        let confirm = if !yes {
                            helper::confirm("Are you sure you want to delete this collection?")
                        } else { true };
                        if confirm {
                            collections.retain(|c| c.name != *collection);
                        } else {
                            return Err("Deletion cancelled.".into());
                        }
                    } else {
                        if let Some(requests) = col.requests.as_mut() {
                            if let Some(_req) = requests.iter().find(|r| r.name == *endpoint) {
                                println!("Deleting endpoint '{}'", endpoint);
                                let confirm = if !yes {
                                    helper::confirm("Are you sure you want to delete this endpoint?")
                                } else { true };
                                if confirm {
                                    requests.retain(|r| r.name != *endpoint);
                                } else {
                                    return Err("Deletion cancelled.".into());
                                }
                            } else {
                                return Err("Endpoint not found.".into());
                            }
                        }
                    }
                } else {
                    return Err("Collection not found.".into());
                }
                let result = helper::write_json_to_file(&collections);
                match result {
                    Ok(_) => {
                        if endpoint.is_empty() {
                            println!("Collection deleted successfully!" )
                        } else {
                            println!("Endpoint deleted successfully!" )
                        }
                    },
                    Err(e) => eprintln!("Error writing collection: {}", e),
                }

            },

            // Copy a collection or endpoint
            Self::Copy { collection, endpoint, to_col, new_name } => {
                let mut collections = Self::load_collections()?;

                if  let Some(col) = collections.iter().find(|c| c.name == *collection) {
                    if endpoint.is_empty() {
                        let mut new_col = col.clone();
                        new_col.name = new_name.clone();
                        collections.push(new_col);
                    } else {
                        if let Some(req) = col.requests.as_ref().and_then(|r| r.iter().find(|r| r.name == *endpoint)) {
                            let mut new_req = req.clone();

                            if *to_col {
                                if let Some(to_collection) = collections.iter_mut().find(|c| c.name == *new_name) {
                                    let mut new_requests = to_collection.requests.clone().unwrap_or_default();
                                    new_requests.push(new_req);
                                    to_collection.requests = Some(new_requests);
                                } else {
                                    return Err("Copy to Collection not found.".into());
                                }
                            } else {
                                new_req.name = new_name.clone();
                                let mut new_requests = col.requests.clone().unwrap_or_default();
                                new_requests.push(new_req);

                                if let Some(col_mut) = collections.iter_mut().find(|c| c.name == *collection) {
                                    col_mut.requests = Some(new_requests);
                                }
                            }

                        } else {
                            return Err("Endpoint not found.".into());
                        }
                    }
                } else {
                    return Err("Collection not found.".into());
                }

                match helper::write_json_to_file(&collections) {
                    Ok(_) => println!("Copy command successful!"),
                    Err(e) => eprintln!("Error writing collections: {}", e),
                }
            },

            // Update a collection or endpoint headers and body
            Self::Update { collection, endpoint,url, headers, body } => {
                let collections = Self::load_collections()?;
                let mut found = false;
                let collections: Vec<models::collection::Collection> = collections.into_iter().map(|mut c| {
                    if c.name == *collection {
                        found = true;
                        let requests = c.requests.unwrap_or_default();
                        if endpoint.is_empty() {
                            if !url.is_empty() {
                                c.url = url.to_string();
                            }
                            if !headers.is_empty() {
                                c.headers = Self::merge_headers(c.headers, headers);
                            }
                            c.requests = Some(requests);
                            c
                        } else {
                            let requests: Vec<models::collection::Request> = requests.into_iter().map(|mut r| {
                                if r.name == *endpoint {
                                    if !url.is_empty() {
                                        r.endpoint = url.to_string();
                                    }
                                    if !headers.is_empty() {
                                        r.headers = Self::merge_headers(r.headers , &headers);
                                    }
                                    if !body.is_empty() {
                                        r.body = Some(body.clone());
                                    }
                                }
                                r
                            }).collect();
                            c.requests = Some(requests);
                            c
                        }
                    } else {
                        c
                    }
                }).collect();
                if !found {
                    return Err("Collection not found.".into());
                }
                let result = helper::write_json_to_file(&collections);
                match result {
                    Ok(_) => println!("Collection updated successfully!" ),
                    Err(e) => eprintln!("Error writing collections: {}", e),
                }
            }

            // Add a new collection or update an existing one
            Self::Col { name, url, headers } => {
                let mut collections = Self::load_collections()?;
                // Check if a collection with the same name already exists
                if let Some(col) = collections.iter_mut().find(|c| c.name == *name) {
                    eprintln!("Collection with name '{}' already exists.", name);
                    col.url = url.to_string();
                    col.headers = headers.to_vec();
                    let result = helper::write_json_to_file(&collections);
                    match result {
                        Ok(_) => println!("Collection updated successfully!"),
                        Err(e) => eprintln!("Error updating collection: {}", e),
                    }
                } else {
                    let collection = models::collection::Collection {
                        name: name.to_string(),
                        url: url.to_string(),
                        headers: headers.to_vec(),
                        requests: None,
                    };
                    collections.push(collection);
                    let result = helper::write_json_to_file(&collections);
                    match result {
                        Ok(_) => println!("Collection added successfully!"),
                        Err(e) => eprintln!("Error writing collections: {}", e),
                    }
                }
            },

            // Add a new endpoint to a collection or update an existing one
            Self::Endpoint { collection, name, path, method, headers, body } => {
                let collections = Self::load_collections()?;
                let mut found = false;
                let collections: Vec<models::collection::Collection> = collections.into_iter().map(|c| {
                    if c.name == *collection {
                        found = true;
                        let request = models::collection::Request {
                            name: name.clone(),
                            endpoint: path.clone(),
                            method: method.to_uppercase().parse().unwrap_or_else(|_| {
                                panic!("Invalid HTTP method: {}", method);
                            }),
                            headers: headers.clone(),
                            body: if body.trim().is_empty() {
                                None
                            } else {
                                Some(body.clone())
                            },
                        };
                        let requests = c.requests.unwrap_or_default();
                        let requests: Vec<models::collection::Request> = requests
                        .into_iter()
                        .filter(|r| r.name != *name)
                        .chain(std::iter::once(request))
                        .collect();
                        models::collection::Collection {
                            name: c.name,
                            url: c.url,
                            headers: c.headers,
                            requests: Some(requests),
                        }
                    } else {
                        c
                    }
                }).collect();
                if !found {
                    return Err("Collection not found.".into());
                }
                let result = helper::write_json_to_file(&collections);
                match result {
                    Ok(_) => println!("Endpoint added successfully!" ),
                    Err(e) => eprintln!("Error writing collections: {}", e),
                }
            },
        }

        Ok("".to_string())
    }

}
