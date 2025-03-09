use colored::*;
use clap::Parser;
use regex::Regex;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use inquire::Select;

#[cfg(windows)]
mod windows_console {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::wincon::SetConsoleTitleW;

    pub fn set_console_title(title: &str) {
        let wide: Vec<u16> = OsStr::new(title)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();
        unsafe {
            SetConsoleTitleW(wide.as_ptr());
        }
    }
}
#[cfg(windows)]
fn pause() {
    if Args::parse().bypass_pause {
        return;
    }
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
#[cfg(not(windows))]
fn pause() {
    // No-op on non-Windows platforms
}
#[cfg(not(windows))]
mod windows_console {
    pub fn set_console_title(_title: &str) {
        // No-op on non-Windows platforms
    }
}

fn replace_hex_at_offset(
    data: &mut Vec<u8>,
    offset: usize,
    repl: &str,
    log_style: bool,
) -> Result<(), String> {
    let bytes: Vec<u8> = repl
        .split_whitespace()
        .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    if offset + bytes.len() > data.len() {
        return Err(format!(
            "Replacement exceeds data size at offset 0x{:X}",
            offset
        ));
    }

    data[offset..offset + bytes.len()].copy_from_slice(&bytes);
    log_offset(offset, log_style, "Patching");
    Ok(())
}

fn insert_hex_at_offset(
    data: &mut Vec<u8>,
    offset: usize,
    repl: &str,
    log_style: bool,
) -> Result<(), String> {
    let bytes: Vec<u8> = repl
        .split_whitespace()
        .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    if offset > data.len() {
        return Err(format!(
            "Insertion exceeds data size at offset 0x{:X}",
            offset
        ));
    }

    data.splice(offset..offset, bytes.iter().cloned());
    log_offset(offset, log_style, "Inserting");
    Ok(())
}

fn log_offset(offset: usize, log_style: bool, action: &str) {
    if log_style {
        println!("{}", format!("[OFFSET] At: 0x{:X}", offset).cyan());
    } else {
        println!("{}", format!("{} at Offset: 0x{:X}", action, offset).cyan());
    }
}

fn wildcard_pattern_scan(data: &[u8], pattern: &str, log_style: bool) -> Option<usize> {
    let pattern_bytes: Vec<Option<u8>> = pattern
        .split_whitespace()
        .map(|s| {
            if s == "??" {
                None
            } else {
                Some(u8::from_str_radix(s, 16).ok()?)
            }
        })
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

fn log_pattern_found(pattern: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[FOUND] Match for pattern: {}", pattern.blue()).green()
        );
    }
}

fn find_offset_by_method_name(
    method_name: &str,
    dump_path: &str,
    log_style: bool,
) -> Result<Option<usize>, io::Error> {
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

fn log_method_found(method_name: &str, offset: usize, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[FOUND] Method name: {}", method_name.blue()).green()
        );
    } else {
        println!(
            "{}",
            format!("Found {} at Offset: 0x{:X}", method_name, offset).green()
        );
    }
}

fn log_no_offset_found(method_name: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[WARNING] No offset found for {}.", method_name.yellow()).bold()
        );
    } else {
        println!(
            "{}",
            format!("Warning: No offset found for {}.", method_name).yellow()
        );
    }
}

fn apply_patch(
    data: &mut Vec<u8>,
    offset: usize,
    patch: &Value,
    log_style: bool,
) -> Result<(), String> {
    if offset >= data.len() {
        return Err(format!(
            "Error: Offset 0x{:X} is out of range for the input file.",
            offset
        )
        .red()
        .to_string());
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

fn log_patch_action(action: &str, hex: &str, log_style: bool) {
    if log_style {
        println!("{}", format!("[PATCH] {} with: {}", action, hex).purple());
    }
}

fn patch_code(
    input: &str,
    output: &str,
    patch_list: &Value,
    dump_path: Option<&str>,
    log_style: bool,
) -> Result<(), io::Error> {
    // Check if input file exists and is readable
    if !std::path::Path::new(input).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input file '{}' does not exist", input),
        ));
    }
    let input_metadata = std::fs::metadata(input)?;
    if input_metadata.permissions().readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Input file '{}' is not readable", input),
        ));
    }

    // Check if output file is writable
    if std::path::Path::new(output).exists() {
        let output_metadata = std::fs::metadata(output)?;
        if output_metadata.permissions().readonly() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Output file '{}' is not writable", output),
            ));
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
                if let Some(offset) =
                    find_offset_by_method_name(method_name.as_str().unwrap(), dump_path, log_style)?
                {
                    offset
                } else {
                    log_patch_skip(method_name.as_str().unwrap(), "Method not found", log_style);
                    continue;
                }
            } else {
                log_patch_skip(
                    method_name.as_str().unwrap(),
                    "dump_path is required for method_name patches",
                    log_style,
                );
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
            if let Some(offset) =
                wildcard_pattern_scan(&data, wildcard.as_str().unwrap(), log_style)
            {
                offset
            } else {
                log_patch_skip(
                    wildcard.as_str().unwrap(),
                    "Wildcard pattern not found",
                    log_style,
                );
                continue;
            }
        } else {
            log_patch_skip(
                "unknown",
                "Patch does not contain a valid method_name, offset, or wildcard",
                log_style,
            );
            continue;
        };

        // Apply the patch at the calculated offset
        if let Err(e) = apply_patch(&mut data, offset, patch, log_style) {
            log_patch_error("Applying patch", &e, log_style);
        }
    }

    if data.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No data to write to output file",
        ));
    }

    // Open output file with write permissions
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)?;
    output_file.write_all(&data)?;

    log_patch_done(output, log_style);
    Ok(())
}

fn log_patch_skip(item: &str, reason: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[WARNING] {}. Skipping patch: {}", reason, item)
                .yellow()
                .red()
        );
    } else {
        println!(
            "{}",
            format!("Warning: {}. Skipping patch: {}", reason, item).yellow()
        );
    }
}

fn log_patch_error(item: &str, error: &str, log_style: bool) {
    if log_style {
        println!("{}", format!("[ERROR] {}: {}", item, error).red());
    } else {
        println!("{}", format!("Error: {}: {}", item, error).red());
    }
}

fn log_patch_done(output: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[DONE] Patched file saved as: '{}'.", output).green()
        );
    } else {
        println!("{}", format!("Patched to: '{}'.", output).green());
    }
}

fn display_menu(files: &[Value]) -> Result<usize, io::Error> {
    let options: Vec<String> = files.iter().map(|file_config| {
        let input = file_config["input"].as_str().unwrap_or("Unknown");
        let title = file_config["title"].as_str().unwrap_or(input);
        format!("{}", title)
    }).collect();

    let selection = Select::new("Select a file to patch:", options)
        .with_vim_mode(true)
        .raw_prompt()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid selection"))?
        .index;

    Ok(selection)
}

#[derive(Debug, clap::Parser)]
#[command(name = "Patcher", about = "A tool to patch binary files based on a configuration file", version, author)]
struct Args {
  #[arg(short, long, help = "Path to the config file", default_value = "config.json")]
  config: String,

  #[cfg(windows)]
  #[arg(short = 'k', long, help = "Bypass Pause")]
  bypass_pause: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    windows_console::set_console_title("Binary Patcher");
    // Enable ANSI color codes on Windows
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();
    // Parse command-line arguments for custom config file
    let args = Args::parse(); 
    let config_path = args.config;

    if !std::path::Path::new(config_path.as_str()).exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Config file does not exist",
        )));
    }

    // Validate and read the config file
    let config_metadata = std::fs::metadata(config_path.clone())?;
    if config_metadata.len() > 10 * 1024 * 1024 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Config file is too large",
        )));
    }

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let config: Value = serde_json::from_reader(reader)?;

    // Validate the JSON structure
    if !config.is_object()
        || !config["BinaryPatch"].is_object()
        || !config["BinaryPatch"]["files"].is_array()
    {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid config file structure",
        )));
    }

    let files = config["BinaryPatch"]["files"]
        .as_array()
        .ok_or("Missing files in config")?;
    let log_style = config["BinaryPatch"]["style"].as_bool().unwrap_or(true);
    let use_menu = config["BinaryPatch"]["menu"].as_bool().unwrap_or(false);

    let file_configs = if use_menu {
        let selected_index = display_menu(files)?;
        vec![files[selected_index].clone()]
    } else {
        files.clone()
    };

    for file_config in file_configs {
        let input = file_config["input"]
            .as_str()
            .ok_or("Missing input in config")?;
        let output = file_config["output"]
            .as_str()
            .ok_or("Missing output in config")?;
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
    pause();
    Ok(())
}
