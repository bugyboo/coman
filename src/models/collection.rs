use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub name: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub requests: Option<Vec<Request>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub name: String,
    pub endpoint: String,
    pub method: Method,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            _ => Err(()),
        }
    }
}
