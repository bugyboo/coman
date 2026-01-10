
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

Coman is a simple API manager designed to streamline API management and request sending. Its key features include:

- **Collections Management**: Store APIs grouped in collections where each collection can have a base URL.
- **Endpoint Management**: Each endpoint is relative to its parent collection's URL. Endpoints can have multiple headers and a body.
- **Header Merging**: Collections can have default headers used by their endpoints. If an endpoint defines the same header as its parent collection, the endpoint header will override the collection header.
- **Command Memory**: Coman has a few subcommands and options that are easy to remember.
- **Persist Collections**: Coman saves a JSON file in the home directory.
- **Pretty JSON output**: By default, API results are treated as JSON unless the streaming option is defined.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Main Commands](#main-commands)
- [Global Options](#global-options)
- [Command Details](#command-details)
- [Examples](#examples)
- [Additional Resources](#additional-resources)

## Installation

### Prerequisites

- Rust (latest stable version recommended)

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

## Usage

```bash
coman [OPTIONS] <COMMAND>
```

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

### Limitations

- Coman can't delete headers or body after created. Instead, you can delete the endpoint.

For more help, use the `help` command with any of the subcommands:

```bash
coman help
coman man --help
coman req --help
```



