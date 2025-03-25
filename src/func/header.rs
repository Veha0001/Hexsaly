use colored::*;
use inquire::{Confirm, Select};
use serde_json::{self, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::Path;
pub fn display_menu(files: &[Value], default_index: Option<usize>) -> Result<usize, io::Error> {
    let options: Vec<String> = files
        .iter()
        .map(|file_config| {
            let input = file_config["input"].as_str().unwrap_or("Unknown");
            let title = file_config["title"].as_str().unwrap_or(input);
            title.to_string()
        })
        .collect();

    let mut select = Select::new("Select a file to patch:", options).with_vim_mode(true);

    if let Some(index) = default_index {
        select = select.with_starting_cursor(index);
    }

    match select.raw_prompt() {
        Ok(selection) => Ok(selection.index),
        Err(_) => {
            println!("{}", "Operation cancelled by user.".yellow());
            std::process::exit(0);
        }
    }
}

pub fn read_config(
    config_path: &Path,
) -> Result<(Vec<Value>, bool, bool), Box<dyn std::error::Error>> {
    let config_metadata = fs::metadata(config_path)?;
    if config_metadata.len() > 10 * 1024 * 1024 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Config file is too large",
        )));
    }

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let config: Value = serde_json::from_reader(reader)?;

    if !config.is_object()
        || !config["Hexsaly"].is_object()
        || !config["Hexsaly"]["files"].is_array()
    {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid config file structure",
        )));
    }

    let files = config["Hexsaly"]["files"]
        .as_array()
        .ok_or("Missing files in config")?
        .clone();
    let log_style = config["Hexsaly"]["style"].as_bool().unwrap_or(true);
    let use_menu = config["Hexsaly"]["menu"].as_bool().unwrap_or(true);

    Ok((files, log_style, use_menu))
}

pub fn write_example_config() -> Result<(), Box<dyn std::error::Error>> {
    let example_config = r#"{
    "Hexsaly": {
        "style": true,
        "menu": false,
        "files": [
            {
                "title": "Example File",
                "input": "example.bin",
                "output": "example_patched.bin",
                "patches": [
                    {
                        "method_name": "ExampleMethodNameFromIl2cpp_dump.cs",
                        "hex_insert": "90 90 90 90 90"
                    },
                    {
                        "wildcard": "90 ?? 90 90",
                        "hex_replace": "90 90 90 90"
                    },
                    {
                        "offset": "0x1234",
                        "hex_insert": "90 90 90 90"
                    }
                ],
                "dump_cs": "dump.cs",
                "require": false
            }
        ]
    }
}"#;

    // Print the example config in green color
    println!("{}", example_config.green());

    // Ask the user if they want to save the example config to a file named 'example_config.json'
    let ask_to_save = Confirm::new(
        "Do you want to save this example config to a file named 'example_config.json'?",
    )
    .with_default(false)
    .prompt()?;

    // If the user confirms, create the file and write the example config to it
    if ask_to_save {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .truncate(false)
            .open("example_config.json")?;
        file.write_all(example_config.as_bytes())?;
        println!(
            "{}",
            "Example config saved as 'example_config.json'.".cyan()
        );
        println!(
            "{}",
            "Please rename 'example_config.json' to 'config.json' for Hexsaly to use it.".cyan()
        );
    }

    Ok(())
}

pub fn validate_patch_structure(patch: &Value, log_style: bool) -> bool {
    // Count the number of read methods (offset, wildcard, method_name)
    let read_count = [
        patch.get("offset").is_some(),
        patch.get("wildcard").is_some(),
        patch.get("method_name").is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    // Count the number of hex methods (hex_replace, hex_insert)
    let hex_count = [
        patch.get("hex_replace").is_some(),
        patch.get("hex_insert").is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if read_count != 1 || hex_count != 1 {
        if log_style {
            println!("{}","[ERROR] Invalid patch structure. Must have exactly two things: offset/wildcard/method_name and hex_replace/hex_insert.".red());
        } else {
            println!("{}","Error: Invalid patch structure. Must have exactly two things: offset/wildcard/method_name and hex_replace/hex_insert.".red());
        }
        return false;
    }
    true
}
