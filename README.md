# BinaryPatch
My Binary Patcher built with Rust.

## Overview
BinaryPatch is a tool designed to apply patches to binary files. It supports various methods for locating the patch offsets, including direct offsets, method names, and wildcard patterns.

## Features
- Apply patches to binary files using hex codes.
- Locate patch offsets using method names from a dump file.
- Support for wildcard pattern scanning.
- Configurable logging styles for detailed output.
- Handle multiple input and output files.
- Option to require files for patching or continue if not found.

## Usage
- **Configuration File**: Create a `config.json` file with the following structure:
    ```json
    {
        "BinaryPatch": {
            "log_style": 1,
            "files": [
                {
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

## Logging Styles
- `log_style: 0`: Minimal logging.
- `log_style: 1`: Detailed logging with colored output.

## Building
To build the project, use the following command:
```sh
cargo build --release
```
