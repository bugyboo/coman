//! CLI commands for managing collections and endpoints
//!
//! This module provides the command-line interface for managing API collections,
//! delegating the actual work to the core CollectionManager.

use clap::Subcommand;
use std::fmt;

use crate::core::collection_manager::CollectionManager;
use crate::core::errors::CollectionError;
use crate::models::collection::Method;

use super::request::RequestCommands;
use super::request_data::RequestData;

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
    pub fn get_manager() -> CollectionManager {
        CollectionManager::default()
    }

    /// Get a RequestCommands for running an endpoint from a collection
    pub fn get_endpoint_command(col_name: &str, ep_name: &str) -> Option<RequestCommands> {
        let mut manager = Self::get_manager();
        let col = manager.get_collection(col_name).ok()?;
        // let req = manager.get_endpoint(collection, endpoint).ok()?;
        let req = col
            .get_request(ep_name)
            .ok_or_else(|| CollectionError::EndpointNotFound(ep_name.to_string()))
            .ok()?;
        let data = RequestData {
            url: format!("{}{}", col.url, req.endpoint),
            headers: manager.get_endpoint_headers(col_name, ep_name),
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
}
