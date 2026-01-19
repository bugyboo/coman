
<div align="center">

<pre>
  ____                            
 / ___|___  _ __ ___   __ _ _ __  
| |   / _ \| '_ ` _ \ / _` | '_ \ 
| |__| (_) | | | | | | (_| | | | |
 \____\___/|_| |_| |_|\__,_|_| |_|

</pre>
</div>

# Coman

Coman is a simple API manager designed to streamline API management and request sending. It can be used as a **CLI tool** or as a **Rust library** in your own projects.

## Key Features

- **Collections Management**: Store APIs grouped in collections where each collection can have a base URL.
- **Endpoint Management**: Each endpoint is relative to its parent collection's URL. Endpoints can have multiple headers and a body.
- **Header Merging**: Collections can have default headers used by their endpoints. If an endpoint defines the same header as its parent collection, the endpoint header will override the collection header.
- **Command Memory**: Coman has a few subcommands and options that are easy to remember.
- **Persist Collections**: Coman saves a JSON file in the home directory.
- **Pretty JSON output**: By default, API results are treated as JSON unless the streaming option is defined.
- **Library Support**: Use coman as a library in your Rust projects for programmatic API management.

## Table of Contents

- [Installation](#installation)
- [Usage as CLI](#usage-as-cli)
- [Usage as Library](#usage-as-library)
- [Main Commands](#main-commands)
- [Global Options](#global-options)
- [Command Details](#command-details)
- [Examples](#examples)
  - [Managing Collections](#managing-collections)
  - [Sending Requests](#sending-requests)
  - [Running Endpoints](#running-endpoints)
  - [Prompting for Missing Data](#prompting-for-missing-data)
  - [Pipe operation](#pipe-operation)
- [Additional Resources](#additional-resources)

## Installation

### Prerequisites

- Rust (latest stable version recommended)

### Installing as CLI

Install from crates.io:
```bash
cargo install coman
```

### Building from Source

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd coman
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. The binary will be available at `target/release/coman`.

Alternatively, you can run it directly with:
```bash
cargo run --release -- <args>
```

### Using as a Library

Add coman to your `Cargo.toml`:

```toml
[dependencies]
coman = { version = "1.2", default-features = false }  # Library only, no CLI deps
```

Or with default features (includes CLI dependencies):
```toml
[dependencies]
coman = "1.2"
```

## Usage as CLI

```bash
coman [OPTIONS] <COMMAND>
```

## Usage as Library

Coman can be used as a library for programmatic API collection management and HTTP requests:

```rust
use coman::{CollectionManager, HttpClient, Method};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a collection manager with a custom file path
    let manager = CollectionManager::new(Some("my-apis.json".to_string()));

    // Add a new collection
    manager.add_collection("my-api", "https://api.example.com", vec![])?;

    // Add an endpoint to the collection
    manager.add_endpoint(
        "my-api",
        "get-users",
        "/users",
        Method::Get,
        vec![("Authorization".to_string(), "Bearer token".to_string())],
        None,
    )?;

    // Make an HTTP request using the HttpClient
    let client = HttpClient::new();
    let response = client
        .get("https://api.example.com/users")
        .headers(vec![("Authorization".to_string(), "Bearer token".to_string())])
        .send()
        .await?;

    println!("Status: {}", response.status);
    println!("Body: {}", response.body);

    // Or execute a saved endpoint directly
    let response = client.execute_endpoint(&manager, "my-api", "get-users").await?;
    println!("Response: {}", response.body);

    Ok(())
}
```

### Library API Overview

**Core Types:**
- `CollectionManager` - Manage collections and endpoints
- `HttpClient` - Make HTTP requests
- `HttpRequest` - Build custom requests
- `HttpResponse` - Response with status, headers, body
- `Collection`, `Request`, `Method` - Data models

**CollectionManager Methods:**
- `new(file_path)` - Create manager with optional custom file
- `add_collection(name, url, headers)` - Add/update collection
- `add_endpoint(collection, name, path, method, headers, body)` - Add/update endpoint
- `delete_collection(name)` - Delete a collection
- `delete_endpoint(collection, endpoint)` - Delete an endpoint
- `get_collection(name)` - Get collection details
- `get_endpoint(collection, endpoint)` - Get endpoint details
- `list_collections()` - List all collections

**HttpClient Methods:**
- `get(url)`, `post(url)`, `put(url)`, `delete(url)`, `patch(url)` - Create requests
- `execute_endpoint(manager, collection, endpoint)` - Execute saved endpoint

## Main Commands

- **list**: List APIs Collections
- **man**: Managing APIs
- **req**: Sending requests
- **run**: Running collections endpoints
- **url**: Print request URL with headers and body
- **test**: Run tests on collections
- **help**: Print this message or the help of the given subcommand(s)

## Global Options

- `-h, --help`: Print help
- `-V, --version`: Print version

## Command Details

### List APIs Collections (`list`)

List all collections and endpoints.

**Usage**:
```bash
coman list [OPTIONS]
```

**Options**:
- `-c, --col <COL>`: Specify a collection (default: all)
- `-e, --endpoint <ENDPOINT>`: Specify an endpoint (default: all)
- `-q, --quiet`: Quiet mode
- `-v, --verbose`: Verbose output
- `-h, --help`: Print help

### Managing APIs (`man`)

Manage API collections and endpoints.

**Usage**:
```bash
coman man <COMMAND>
```

**Commands**:
- **list**: List collections and endpoints
- **update**: Update a collection or endpoint headers and body
- **delete**: Delete a collection or endpoint
- **copy**: Copy a collection or endpoint
- **col**: Add a new collection
- **endpoint**: Add a new endpoint to a collection
- **help**: Print this message or the help of the given subcommand(s)

**Options**:
- `-h, --help`: Print help

### Sending Requests (`req`)

Send HTTP requests.

**Usage**:
```bash
coman req [OPTIONS] <COMMAND>
```

**Commands**:
- **get**
- **post**
- **put**
- **delete**
- **patch**
- **help**: Print this message or the help of the given subcommand(s)

**Options**:
- `-v, --verbose`: Verbose output
- `-s, --stream`: Stream the request/response (read bytes from stdin and send as the request body or multipart data to the endpoint)
- `-h, --help`: Print help

### Running Collections Endpoints (`run`)

Run endpoints from collections.

**Usage**:
```bash
coman run [OPTIONS] <COLLECTION> <ENDPOINT>
```

**Options**:
- `-v, --verbose`: Verbose output
- `-s, --stream`: Stream the request/response (output response as bytes)
- `-h, --help`: Print help

### Print Request URL (`url`)

Print the request URL with headers and body.

**Usage**:
```bash
coman url <COLLECTION> <ENDPOINT>
```

**Options**:
- `-h, --help`: Print help

## Examples

### Managing Collections

- Add a new collection:
  ```bash
  coman man col myapi http://api.example.com
  coman man col myapi "http://api.example.com" -H "Content-Type: application/json" -H "x-api-key: xxx"
  ```

- Add an endpoint to a collection:
  ```bash
  coman man endpoint myapi users /users
  coman man endpoint myapi users "/users" -H "Content-Type: application/json" -m POST -b "Hello!"
  ```

- List all collections:
  ```bash
  coman list
  coman list -q
  ```

- List endpoints in a specific collection:
  ```bash
  coman list -c myapi
  coman list -vc myapi
  ```

### Sending Requests

- Send a GET request:
  ```bash
  coman req get http://api.example.com/users
  ```

- Send a POST request with a body:
  ```bash
  coman req post http://api.example.com/users -b '{"name": "John"}'
  ```

- Send a request with headers:
  ```bash
  coman req get http://api.example.com/users -H "Authorization: Bearer token"
  ```

### Running Endpoints

- Run an endpoint from a collection:
  ```bash
  coman run myapi users
  ```

### Prompting for Missing Data

Coman supports interactive prompts for missing data using the `:?` placeholder. When a header value or request body contains `:?`, Coman will prompt you to enter the value at runtime. This is useful for sensitive data like tokens or dynamic values that change between requests.

#### Missing Header Data

If a header value contains `:?`, Coman will prompt for the correct value:

- Create an endpoint with a placeholder in the header:
  ```bash
  coman man endpoint myapi secure "/protected" -H "Authorization: Bearer :?"
  ```

- When you run the endpoint, Coman will prompt:
  ```bash
  coman run myapi secure
  # Output: Header value for key 'Authorization' is missing data. Please provide the correct value: 
  # Enter your token and press Enter
  ```

#### Missing Body Data

If the request body contains `:?`, Coman will prompt for each placeholder:

- Create an endpoint with placeholders in the body:
  ```bash
  coman man endpoint myapi create-user "/users" -m POST -b '{"username": ":?", "email": ":?"}'
  ```

- When you run the endpoint, Coman will prompt for each `:?`:
  ```bash
  coman run myapi create-user
  # Output: Missing data at position 14 - {"username": ":?", "email": ":?"}. Please provide the correct value: 
  # Enter "john_doe" and press Enter
  # Output: Missing data at position 31 - {"username": "john_doe", "email": ":?"}. Please provide the correct value:
  # Enter "john@example.com" and press Enter
  ```

#### Direct Requests with Placeholders

You can also use placeholders in direct requests:

- URL with placeholder:
  ```bash
  coman req get "http://api.example.com/users/:?"
  # Prompts for the user ID
  ```

- Headers and body with placeholders:
  ```bash
  coman req post "http://api.example.com/login" -H "X-API-Key: :?" -b '{"password": ":?"}'
  # Prompts for API key, then password
  ```

> **Note**: Prompting is disabled when using the `-s` (stream) option to allow for non-interactive piped operations.

### Pipe operation

Coman supports reading request body from standard input when piping data. This is useful for sending JSON payloads or other data directly from files or other commands.

- Send JSON data from a file as the request body:
  ```bash
  cat data.json | coman run myapi send
  ```

- Pipe output from another command as the request body:
  ```bash
  echo '{"key": "value"}' | coman run myapi create
  coman run myapi stream -s < file.bin
  coman run myapi stream < file.bin // 'sent as multi-part'
  coman run myapi post-data < file.json // 'send as body'
  ```

When data is piped to coman, it will override any body defined in the endpoint configuration.

### Updating and Deleting Headers/Body

To remove a header or clear the body from a collection or endpoint, use the `update` command with an empty value:

- Remove a header from an endpoint:
  ```bash
  coman man update myapi users -H "Authorization:"
  ```

- Clear the body from an endpoint:
  ```bash
  coman man update myapi users -b ""
  ```

For more help, use the `help` command with any of the subcommands:

```bash
coman help
coman man --help
coman req --help
```



