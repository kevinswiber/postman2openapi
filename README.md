# postman2openapi

Convert Postman collections to OpenAPI definitions.

[![Build status](https://github.com/kevinswiber/postman2openapi/workflows/ci/badge.svg)](https://github.com/kevinswiber/postman2openapi/actions)

**Try it on the Web: https://kevinswiber.github.io/postman2openapi/**

- [CLI](#cli)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Examples](#examples)
- [JavaScript Library](#javascript-library)
  - [Installation](#installation-1)
  - [Usage](#usage-1)
  - [JavaScript API](#javascript-api)
- [License](#license)

## CLI

### Installation

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
cargo install postman2openapi --bin --features binary
```

To install from the latest on GitHub, use:

```
cargo install --git https://github.com/kevinswiber/postman2openapi --features binary
```

### Usage

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

## JavaScript library

### Installation

```
npm install postman2openapi
```

### Usage

```js
const collection = require('./collection'); // any Postman collection JSON file
const { transpile } = require('postman2openapi');

// Returns a JavaScript object representation of the OpenAPI definition.
const openapi = transpile(collection);

console.log(JSON.stringify(openapi, null, 2));
```

### JavaScript API

#### transpile(collection: object): object

- collection - An object representing the Postman collection.
- _returns_ - an OpenAPI definition as a JavaScript object.

## License

Apache License 2.0 (Apache-2.0)
