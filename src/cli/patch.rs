use crate::func::header::validate_patch_structure;
use crate::func::logger::*;
use colored::*;
use regex::Regex;
use serde_json::{self, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};

pub fn replace_hex_at_offset(
    data: &mut [u8],
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

pub fn insert_hex_at_offset(
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

pub fn wildcard_pattern_scan(
    data: &[u8],
    pattern: &str,
    log_style: bool,
) -> Option<(usize, Vec<u8>)> {
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
        // Return both offset and matched bytes
        return Some((i, data[i..i + pattern_bytes.len()].to_vec()));
    }
    None
}

pub fn find_offset_by_method_name(
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

pub fn apply_patch(
    data: &mut Vec<u8>,
    offset: usize,
    patch: &Value,
    wildcard_bytes: Option<&[u8]>,
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

    let process_hex = |hex: &str, matched: Option<&[u8]>| -> Result<String, String> {
        if let (Some(pos), Some(matched)) =
            (patch.get("position").and_then(|p| p.as_u64()), matched)
        {
            let pos = pos as usize;
            if pos > matched.len() {
                return Err("Position exceeds wildcard pattern length".into());
            }

            // Take bytes from wildcard up to position
            let matched_hex: String = matched[..pos]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            // Combine with hex_replace/insert values
            Ok(format!("{} {}", matched_hex, hex))
        } else {
            Ok(hex.to_string())
        }
    };

    if let Some(hex_replace) = patch.get("hex_replace") {
        let processed_hex = process_hex(hex_replace.as_str().unwrap(), wildcard_bytes)?;
        replace_hex_at_offset(data, offset, &processed_hex, log_style)?;
        log_patch_action("Replaced", &processed_hex, log_style);
    } else if let Some(hex_insert) = patch.get("hex_insert") {
        let processed_hex = process_hex(hex_insert.as_str().unwrap(), wildcard_bytes)?;
        insert_hex_at_offset(data, offset, &processed_hex, log_style)?;
        log_patch_action("Inserted", &processed_hex, log_style);
    } else {
        return Err("Patch must contain either 'hex_replace' or 'hex_insert'.".into());
    }

    Ok(())
}

pub fn patch_code(
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
    let input_metadata = fs::metadata(input)?;
    if input_metadata.permissions().readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Input file '{}' is not readable", input),
        ));
    }

    // Check if output file is writable
    if std::path::Path::new(output).exists() {
        let output_metadata = fs::metadata(output)?;
        if output_metadata.permissions().readonly() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Output file '{}' is not writable", output),
            ));
        }
    } else {
        // Try to create the output file to check if it's writable
        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output)?;
    }

    // Open input file with read permissions
    let mut input_file = OpenOptions::new().read(true).open(input)?;
    let mut data = Vec::new();
    input_file.read_to_end(&mut data)?;

    for patch in patch_list.as_array().unwrap() {
        if !validate_patch_structure(patch, log_style) {
            return Ok(());
        }

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
            if let Some((offset, matched_bytes)) =
                wildcard_pattern_scan(&data, wildcard.as_str().unwrap(), log_style)
            {
                if let Err(e) =
                    apply_patch(&mut data, offset, patch, Some(&matched_bytes), log_style)
                {
                    log_patch_error("Applying patch", &e, log_style);
                }
                continue;
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

        // Apply the patch at the calculated offset (for non-wildcard cases)
        if let Err(e) = apply_patch(&mut data, offset, patch, None, log_style) {
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


pub fn get_card(
    input: &str,
    offset_str: &str,
    length: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert offset string (hex or decimal) to usize
    let offset = if offset_str.to_lowercase().starts_with("0x") {
        usize::from_str_radix(&offset_str[2..], 16)?
    } else {
        offset_str.parse::<usize>()?
    };

    // Read the file
    let mut file = File::open(input)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Validate offset and length
    if offset >= buffer.len() {
        return Err(format!("Offset 0x{:X} is beyond file size", offset).into());
    }

    let end = std::cmp::min(offset + length, buffer.len());
    let card_data = &buffer[offset..end];

    // Print the card data in different formats
    println!(
        "\n{}",
        format!("Card data at offset 0x{:X}:", offset).yellow()
    );

    // I am sure somethings off of this function :)
    //
    // Print hex values
    print!("{} ", "HEX:".yellow());
    for byte in card_data.iter() {
        print!("{}", format!("{:02X} ", byte).cyan());
    }
    println!();

    // Print ASCII representation
    print!("{} ", "ASCII:".yellow());
    for &byte in card_data {
        if byte.is_ascii() && !byte.is_ascii_control() {
            print!("{}", (byte as char).to_string().green());
        } else {
            print!("{}", ".".red());
        }
    }
    println!();
    Ok(())
}
