use crate::{cli::manager::ManagerCommands, core::utils::merge_headers, helper, Method};
use colored::Colorize;

impl ManagerCommands {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let manager = Self::get_manager();

        match self {
            // List collections and endpoints
            Self::List {
                col,
                endpoint,
                quiet,
                verbose,
            } => {
                let collections = manager.get_collections().await;
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
                        manager.delete_collection(collection).await?;
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
                        manager.delete_endpoint(collection, endpoint).await?;
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
                    manager.copy_collection(collection, new_name).await?;
                } else if *to_col {
                    // Copy endpoint to another collection
                    manager
                        .copy_endpoint(collection, endpoint, new_name, Some(new_name))
                        .await?;
                } else {
                    // Copy endpoint with new name in same collection
                    manager
                        .copy_endpoint(collection, endpoint, new_name, None)
                        .await?;
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
                let mut col = manager
                    .get_collection(collection)
                    .await?
                    .ok_or("Collection not found")?;
                if endpoint.is_empty() {
                    // Update collection
                    let url_opt = if url.is_empty() {
                        None
                    } else {
                        Some(url.as_str())
                    };
                    let headers_opt = if headers.is_empty() {
                        None
                    } else {
                        Some(headers.clone())
                    };
                    col.url = url_opt.unwrap_or(&col.url).to_string();
                    col.headers =
                        merge_headers(col.headers.clone(), &headers_opt.unwrap_or(vec![]));
                    manager.update_add_collection(col).await?;
                } else {
                    // let mut ep = manager
                    //     .get_endpoint(collection, endpoint)
                    //     .await?
                    //     .ok_or("Endpoint not found")?;
                    let ep = col
                        .requests
                        .as_mut()
                        .and_then(|reqs| reqs.iter_mut().find(|r| r.name == *endpoint))
                        .ok_or("Endpoint not found")?;
                    // Update endpoint
                    if !url.is_empty() {
                        ep.endpoint = url.clone();
                    }
                    if !headers.is_empty() {
                        ep.headers = headers.clone();
                    }
                    let body_opt = if body.is_empty() {
                        None
                    } else if body.trim().is_empty() {
                        Some(String::new())
                    } else {
                        Some(body.to_string())
                    };

                    ep.body = if let Some(b) = body_opt {
                        Some(b)
                    } else {
                        ep.body.clone()
                    };
                    manager
                        .update_endpoint(
                            collection,
                            &ep.name,
                            Some(&ep.endpoint),
                            Some(ep.headers.clone()),
                            ep.body.clone(),
                        )
                        .await?;
                }
                println!("Collection updated successfully!");
            }

            // Add a new collection or update an existing one
            Self::Col { name, url, headers } => {
                manager.add_collection(name, url, headers.clone()).await?;
                println!("Collection added successfully!");
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
                let method: Method = method
                    .to_uppercase()
                    .parse()
                    .map_err(|_| format!("Invalid HTTP method: {}", method))?;

                let body_opt = if body.trim().is_empty() {
                    None
                } else {
                    Some(body.clone())
                };

                manager
                    .add_endpoint(collection, name, path, method, headers.clone(), body_opt)
                    .await?;
                println!("Endpoint added successfully!");
            }
        }

        Ok(())
    }
}
