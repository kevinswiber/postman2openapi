# postman2openapi-cli

Convert Postman collections to OpenAPI definitions.

## Installation

[Archives of precompiled binaries for postman2openapi are available for Windows,
macOS and Linux.](https://github.com/kevinswiber/postman2openapi/releases)

Linux binaries are static executables. Windows binaries are available either as
built with MinGW (GNU) or with Microsoft Visual C++ (MSVC). When possible,
prefer MSVC over GNU, but you'll need to have the [Microsoft VC++ 2015
redistributable](https://www.microsoft.com/en-us/download/details.aspx?id=48145)
installed.

For Rust developers, installation is also available via Cargo. [Installing Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

To install the latest published version on crates.io, use:

```
cargo install postman2openapi-cli
```

To install from the latest on GitHub, use:

```
cargo install --git https://github.com/kevinswiber/postman2openapi postman2openapi-cli
```

## Usage

```
USAGE:
    postman2openapi [OPTIONS] [input-file]

ARGS:
    <input-file>    The Postman collection to convert; data may also come from stdin

OPTIONS:
    -f, --output-format <format>    The output format [default: yaml] [possible values: yaml, json]
    -h, --help                      Print help information
    -V, --version                   Print version information
```

#### Examples

```
postman2openapi collection.json > openapi.yaml
```

```
cat collection.json | postman2openapi -f json
```

## License

Apache License 2.0 (Apache-2.0)
