# BinaryPatch
My Binary Patcher built in Rust.

## Overview
BinaryPatch is a tool designed to apply patches to binary files. It supports various methods for locating the patch offsets, including direct offsets, method names, and wildcard patterns.

## Features
- Apply patches to binary files using hex codes.
- Locate patch offsets using method names from a dump file.
- Support for wildcard pattern scanning.
- Configurable logging styles for detailed output.

## Usage
- **Configuration File**: Create a `config.json` file with the following structure:
    ```json
    {
        "Binary": {
            "input_file": "path/to/input/file",
            "output_file": "path/to/output/file",
            "patches": [
                {
                    "method_name": "methodName",
                    "hex_code": "hex values"
                },
                {
                    "offset": "0x1234",
                    "hex_code": "hex values"
                },
                {
                    "wildcard": "?? ?? ??",
                    "hex_code": "hex values"
                }
            ],
            "dump_path": "path/to/dump.cs",
            "log_style": 1
        }
    }
    ```
> [!NOTE]
> Please give me some ideal imporve of this config or Patcher code.
> Thank You!