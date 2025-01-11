use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use serde_json::{Value, json};
use regex::Regex;
use colored::*;

#[cfg(windows)]
mod windows_console {
    use winapi::um::wincon::SetConsoleTitleW;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    pub fn set_console_title(title: &str) {
        let wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0).into_iter()).collect();
        unsafe {
            SetConsoleTitleW(wide.as_ptr());
        }
    }
}

#[cfg(not(windows))]
mod windows_console {
    pub fn set_console_title(_title: &str) {
        // No-op on non-Windows platforms
    }
}

fn replace_hex_at_offset(data: &mut Vec<u8>, offset: usize, repl: &str, log_style: u8) -> Result<(), String> {
    let bytes: Vec<u8> = repl.split_whitespace()
        .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    if offset + bytes.len() > data.len() {
        return Err(format!("Replacement exceeds data size at offset 0x{:X}", offset));
    }

    data[offset..offset + bytes.len()].copy_from_slice(&bytes);
    log_offset(offset, log_style, "Patching");
    Ok(())
}

fn insert_hex_at_offset(data: &mut Vec<u8>, offset: usize, repl: &str, log_style: u8) -> Result<(), String> {
    let bytes: Vec<u8> = repl.split_whitespace()
        .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    if offset > data.len() {
        return Err(format!("Insertion exceeds data size at offset 0x{:X}", offset));
    }

    data.splice(offset..offset, bytes.iter().cloned());
    log_offset(offset, log_style, "Inserting");
    Ok(())
}

fn log_offset(offset: usize, log_style: u8, action: &str) {
    if log_style == 1 {
        println!("{}", format!("[OFFSET] At: 0x{:X}", offset).cyan());
    } else {
        println!("{}", format!("{} at Offset: 0x{:X}", action, offset).cyan());
    }
}

fn wildcard_pattern_scan(data: &[u8], pattern: &str, log_style: u8) -> Option<usize> {
    let pattern_bytes: Vec<Option<u8>> = pattern.split_whitespace()
        .map(|s| if s == "??" { None } else { Some(u8::from_str_radix(s, 16).ok()?) })
        .collect();

    'outer: for i in 0..=data.len() - pattern_bytes.len() {
        for (j, &pat_byte) in pattern_bytes.iter().enumerate() {
            if let Some(pat_byte) = pat_byte {
                if data[i + j] != pat_byte {
                    continue 'outer;
                }
            }
        }
        log_pattern_found(pattern, log_style);
        return Some(i);
    }
    None
}

fn log_pattern_found(pattern: &str, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[FOUND] Match for pattern: {}", pattern.blue()).green());
    }
}

fn find_offset_by_method_name(method_name: &str, dump_path: &str, log_style: u8) -> Result<Option<usize>, io::Error> {
    let file = File::open(dump_path)?;
    let reader = BufReader::new(file);
    let offset_regex = Regex::new(r"Offset:\s*0x([0-9A-Fa-f]+)").unwrap();

    let mut previous_line = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.contains(method_name) {
            if let Some(caps) = offset_regex.captures(&previous_line) {
                let offset = usize::from_str_radix(&caps[1], 16).unwrap();
                log_method_found(method_name, offset, log_style);
                return Ok(Some(offset));
            } else {
                log_no_offset_found(method_name, log_style);
                return Ok(None);
            }
        }
        previous_line = line;
    }
    Ok(None)
}

fn log_method_found(method_name: &str, offset: usize, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[FOUND] Method name: {}", method_name.blue()).green());
    } else {
        println!("{}", format!("Found {} at Offset: 0x{:X}", method_name, offset).green());
    }
}

fn log_no_offset_found(method_name: &str, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[WARNING] No offset found for {}.", method_name.yellow()).bold());
    } else {
        println!("{}", format!("Warning: No offset found for {}.", method_name).yellow());
    }
}

fn apply_patch(data: &mut Vec<u8>, offset: usize, patch: &Value, log_style: u8) -> Result<(), String> {
    if offset >= data.len() {
        return Err(format!("Error: Offset 0x{:X} is out of range for the input file.", offset).red().to_string());
    }

    if let Some(hex_replace) = patch.get("hex_replace") {
        replace_hex_at_offset(data, offset, hex_replace.as_str().unwrap(), log_style)?;
        log_patch_action("Replaced", hex_replace.as_str().unwrap(), log_style);
    } else if let Some(hex_insert) = patch.get("hex_insert") {
        insert_hex_at_offset(data, offset, hex_insert.as_str().unwrap(), log_style)?;
        log_patch_action("Inserted", hex_insert.as_str().unwrap(), log_style);
    } else {
        return Err("Patch must contain either 'hex_replace' or 'hex_insert'.".into());
    }

    Ok(())
}

fn log_patch_action(action: &str, hex: &str, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[PATCH] {} with: {}", action, hex).purple());
    }
}

fn patch_code(input: &str, output: &str, patch_list: &Value, dump_path: Option<&str>, log_style: u8) -> Result<(), io::Error> {
    // Check if input file exists and is readable
    if !std::path::Path::new(input).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Input file '{}' does not exist", input)));
    }
    let input_metadata = std::fs::metadata(input)?;
    if input_metadata.permissions().readonly() {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!("Input file '{}' is not readable", input)));
    }

    // Check if output file is writable
    if std::path::Path::new(output).exists() {
        let output_metadata = std::fs::metadata(output)?;
        if output_metadata.permissions().readonly() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!("Output file '{}' is not writable", output)));
        }
    } else {
        // Try to create the output file to check if it's writable
        let _ = OpenOptions::new().write(true).create(true).open(output)?;
    }

    // Open input file with read permissions
    let mut input_file = OpenOptions::new().read(true).open(input)?;
    let mut data = Vec::new();
    input_file.read_to_end(&mut data)?;

    for patch in patch_list.as_array().unwrap() {
        let offset = if let Some(method_name) = patch.get("method_name") {
            if let Some(dump_path) = dump_path {
                if let Some(offset) = find_offset_by_method_name(method_name.as_str().unwrap(), dump_path, log_style)? {
                    offset
                } else {
                    log_patch_skip(method_name.as_str().unwrap(), "Method not found", log_style);
                    continue;
                }
            } else {
                log_patch_skip(method_name.as_str().unwrap(), "dump_path is required for method_name patches", log_style);
                continue;
            }
        } else if let Some(offset_str) = patch.get("offset").and_then(|v| v.as_str()) {
            match usize::from_str_radix(offset_str.trim_start_matches("0x"), 16) {
                Ok(offset) => offset,
                Err(e) => {
                    log_patch_error(offset_str, &e.to_string(), log_style);
                    continue;
                }
            }
        } else if let Some(wildcard) = patch.get("wildcard") {
            if let Some(offset) = wildcard_pattern_scan(&data, wildcard.as_str().unwrap(), log_style) {
                offset
            } else {
                log_patch_skip(wildcard.as_str().unwrap(), "Wildcard pattern not found", log_style);
                continue;
            }
        } else {
            log_patch_skip("unknown", "Patch does not contain a valid method_name, offset, or wildcard", log_style);
            continue;
        };

        // Apply the patch at the calculated offset
        if let Err(e) = apply_patch(&mut data, offset, patch, log_style) {
            log_patch_error("Applying patch", &e, log_style);
        }
    }

    // Open output file with write permissions
    let mut output_file = OpenOptions::new().write(true).create(true).truncate(true).open(output)?;
    output_file.write_all(&data)?;

    log_patch_done(output, log_style);
    Ok(())
}

fn log_patch_skip(item: &str, reason: &str, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[WARNING] {}. Skipping patch: {}", reason, item).yellow().red());
    } else {
        println!("{}", format!("Warning: {}. Skipping patch: {}", reason, item).yellow());
    }
}

fn log_patch_error(item: &str, error: &str, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[ERROR] {}: {}", item, error).red());
    } else {
        println!("{}", format!("Error: {}: {}", item, error).red());
    }
}

fn log_patch_done(output: &str, log_style: u8) {
    if log_style == 1 {
        println!("{}", format!("[DONE] Patched file saved as: '{}'.", output).green());
    } else {
        println!("{}", format!("Patched to: '{}'.", output).green());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    windows_console::set_console_title("Binary Patcher");
    // Parse command-line arguments for custom config file
    let args: Vec<String> = std::env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config.json"
    };
    // Add help and version argments
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        println!("Usage: {} [config.json]", args[0]);
        return Ok(());
    } else if args.len() > 1 && (args[1] == "-v" || args[1] == "--version") {
        println!("Binary Patcher v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Create a default config file if it doesn't exist
    if !std::path::Path::new(config_path).exists() {
        let default_config = json!({
            "BinaryPatch": {
                "files": [
                    {
                        "input": "input.bin",
                        "output": "output.bin",
                        "patches": [
                            {
                                "offset": "0x123ABC",
                                "hex_insert": "B8 85 47 DE 63 C3"
                            },
                            {
                                "wildcard": "48 83 EC ?? 80 3D ?? ?? ?? ?? ?? 75 ?? 8B ",
                                "hex_replace": "B8 85 47 DE 63 C3"
                            },
                            {
                                "method_name": "SomeMethodNameFromDumpCsFile",
                                "hex_replace": "B8 85 47 DE 63 C3"
                            }
                        ],
                        "dump_cs": "dump.cs",
                        "require": false
                    }
                ],
                "log_style": 1
            }
        });
        let mut file = File::create(config_path)?;
        file.write_all(serde_json::to_string_pretty(&default_config)?.as_bytes())?;
        println!("Created default config file: '{}'", config_path);
        return Ok(());
    }
    // Validate and read the config file
    let config_metadata = std::fs::metadata(config_path)?;
    if config_metadata.len() > 10 * 1024 * 1024 {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Config file is too large")));
    }

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let config: Value = serde_json::from_reader(reader)?;
    
    // Validate the JSON structure
    if !config.is_object() || !config["BinaryPatch"].is_object() || !config["BinaryPatch"]["files"].is_array() {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Invalid config file structure")));
    }

    let files = config["BinaryPatch"]["files"].as_array().ok_or("Missing files in config")?;
    let log_style = config["BinaryPatch"]["log_style"].as_u64().unwrap_or(0) as u8;

    for file_config in files {
        let input = file_config["input"].as_str().ok_or("Missing input in config")?;
        let output = file_config["output"].as_str().ok_or("Missing output in config")?;
        let patch_list = &file_config["patches"];
        let dump_cs = file_config["dump_cs"].as_str();
        let require = file_config["require"].as_bool().unwrap_or(false);

        if let Err(e) = patch_code(input, output, patch_list, dump_cs, log_style) {
            eprintln!("{}", format!("Error: {}", e).red());
            if require {
                return Err(Box::new(e));
            }
        }
    }

    // Prevent the console from closing too quickly
    #[cfg(windows)]
    {
        println!("Press any key to exit...");
        let _ = std::io::stdin().read(&mut [0u8]).unwrap();
    }
    Ok(())
}
