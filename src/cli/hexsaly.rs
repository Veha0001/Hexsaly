use crate::cli::args::{Args, Commands};
use crate::cli::patch::*;
use crate::func::header::*;
use clap::Parser;
use colored::*;
use std::fs;

#[cfg(windows)]
pub fn pause() {
    if Args::parse().no_pause {
        return;
    }
    use std::io::{self, Read, Write};

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[cfg(not(windows))]
pub fn pause() {
    // No-op on non-Windows platforms
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.example_config {
        return write_example_config();
    }

    if !args
        .config
        .as_ref()
        .expect("Config path is not set")
        .exists()
    {
        eprintln!("{}", "Error: Config file not found.\n ".red());
        println!("Use --example-config to generate a sample config file.");
        println!("For more details, run with --help.\n");
        pause();
        return Ok(());
    }

    let config_path = fs::canonicalize(args.config.as_ref().expect("Config path is not set"))?;
    let (files, log_style, use_menu) = read_config(&config_path)?;

    let file_configs = if let Some(Commands::Open { ref input, index }) = args.command {
        let input_str = input.to_str().ok_or("Invalid input path")?;

        if let Some(idx) = index {
            let mut config = files.get(idx).ok_or("Invalid index")?.clone();
            config["input"] = input_str.into();
            config["output"] = input_str.into();
            vec![config]
        } else {
            let selected_index = display_menu(&files, None)?;
            let mut config = files[selected_index].clone();
            config["input"] = input_str.into();
            config["output"] = input_str.into();
            vec![config]
        }
    } else if use_menu {
        let selected_index = display_menu(&files, None)?;
        vec![files[selected_index].clone()]
    } else {
        files
    };

    for file_config in file_configs {
        let input = file_config["input"]
            .as_str()
            .or_else(|| file_config["open"].as_str())
            .ok_or("Missing input or open in config")?;

        let output = if let Some(Commands::Open { input, .. }) = &args.command {
            input.to_str().ok_or("Invalid input path")?
        } else {
            file_config["output"].as_str().unwrap_or(input)
        };

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
