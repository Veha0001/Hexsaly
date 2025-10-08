# Hexsaly

Hexsaly applies binary patches defined in config files.

[![Static Badge](https://img.shields.io/badge/build-Nightly-brightgreen?style=for-the-badge&logo=rust&logoColor=%23ff9e64&labelColor=%23292e42&color=%233b4261)](https://nightly.link/Veha0001/Hexsaly/workflows/ci/main?preview)

## Features

- Apply patches to binary files using hex codes.
- Locate patch offsets using method names from a dump file.
- Support for wildcard pattern scanning.
- Configurable logging styles for detailed output.
- Handle multiple input and output files.
- Option to require files for patching or continue if not found.

## Usage

**Configuration File**: Create a `config.json` file with the following structure:

```json
{
  "Hexsaly": {
    "menu": false,
    "style": true,
    "files": [
      {
        "title": "Example",
        "dump_cs": "path/to/dump.cs",
        "input": "path/to/input/file",
        "output": "path/to/output/file",
        "require": false,
        "patches": [
          {
            "method_name": "methodName",
            "hex_replace": "B8 85 47 DE 63 C3"
          },
          {
            "offset": "0x1234",
            "hex_replace": "B8 85 47 DE 63 C3"
          },
          {
            "wildcard": "?? ?? ??",
            "hex_insert": "hex values"
          }
        ]
      }
    ]
  }
}
```

## Building & Install

To build the project, use the following command:

```sh
cargo build --release
```

To install the project, use the following command:

```sh
cargo install --git https://github.com/Veha0001/Hexsaly
```

### Usages

Create a `config.json` file then open/run the `hexsaly`.
For get help of the command:

```sh
hexsaly -h
```
