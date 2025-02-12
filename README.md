# Coman

Coman is a simple API manager designed to streamline API management and request sending. Its key features include:

- **Collections Management**: Store APIs grouped in collections where each collection can have a base URL.
- **Endpoint Management**: Each endpoint is relative to its parent collection's URL. Endpoints can have multiple headers and a body.
- **Header Merging**: Collections can have default headers used by their endpoints. If an endpoint defines the same header as its parent collection, the endpoint header will override the collection header.
- **Command Memory**: Coman has a few subcommands and options easy to remember.
- **Presist Collection**: Coman saves a json file on home directory.

## Main Commands

- **man**: Managing APIs
- **req**: Sending requests
- **run**: Running collections endpoints
- **url**: Print request URL with headers and body
- **help**: Print this message or the help of the given subcommand(s)

## Global Options

- `-h, --help`     Print help
- `-V, --version`  Print version

## Details for Each Command

### Managing APIs (`man`)

Commands:
- **list**: List all collections and endpoints. 
    ` Use: -c for a specific collection.`
- **delete**: Delete a collection or endpoint. 
    `Example: coman delete colname -c endpoint. if endpoint not specified then delete collection.`
- **col**: Add a new collection. 
    `Example: coman man col test http://localhost.`
- **endpoint**: Add a new endpoint to a collection. 
    `Example: coman man endpoint test /hello.`
- **help**: Print this message or the help of the given subcommand(s).

Options:
- `-v, --verbose`  
- `-h, --help`     Print help

### Running Collections Endpoints (`run`)

**Usage**:
- **collection-name** **endpoint-name**
Options:
- `-v, --verbose`

### Sending Requests (`req`)

**Usage**:  
Commands:
- **get**
    - `Example: coman req get http://localhost/hello`
    - `Example: coman req get http://localhost/hello -H "Accept: application/json`
- **post**
    - `Example: coman req post http://localhost/hello -b "World!"`    
- **put**     
- **delete**  
- **patch**   

Options:
- `-v, --verbose`  
- `-h, --help`     Print help

## Additional Resources

For more help, use the `help` command with any of the subcommands:

