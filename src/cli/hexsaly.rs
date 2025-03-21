use crate::cli::args::Args;
use crate::cli::patch::*;
use crate::func::header::*;
use clap::Parser;
use colored::*;
use std::fs;
#[cfg(windows)]
pub fn pause() {
    use crossterm::event::{read, Event, KeyCode};
    use std::io::{self, Write};
    if Args::parse().no_pause {
        return;
    }
    let mut stdout = io::stdout();
    write!(stdout, "Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    loop {
        if let Ok(Event::Key(event)) = read() {
            if event.code == KeyCode::Enter {
                break;
            }
        }
    }
}
#[cfg(not(windows))]
pub fn pause() {

    // No-op on non-Windows platforms
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.example_config {
        return print_an_example_config();
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

    let file_configs = if use_menu {
        let selected_index = display_menu(&files)?;
        vec![files[selected_index].clone()]
    } else {
        files
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
