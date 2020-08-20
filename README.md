# postman2openapi

Convert Postman collections to OpenAPI definitions.

## Installation

Installation is only available via Cargo at this time.  [Installing Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```
cargo install --git https://github.com/kevinswiber/postman2openapi
```

## Usage

```
USAGE:
    postman2openapi [OPTIONS] [input-file]

ARGS:
    <input-file>    The Postman collection to convert; data may also come from stdin

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <format>    The output format [default: yaml]  [possible values: yaml, json]
```

### Examples

```
postman2openapi collection.json > openapi.yaml
```

```
cat collection.json | postman2openapi -o json
```

## License

Apache License 2.0 (Apache-2.0)

Copyright © 2020 Kevin Swiber kswiber@gmail.com