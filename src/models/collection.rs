use core::fmt;
use std::str::FromStr;

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,   
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub url: String,
    pub headers: Vec<(String, String)>,    
    pub requests: Option<Vec<Request>>,
}

#[derive(Debug, Serialize, Deserialize)]
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
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(()),
        }
    }
}