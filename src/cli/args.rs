use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "Hexsaly",
    about = "A tool to patch binary files based on a configuration file.\nMade by Veha0001.",
    version,
    author
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        short,
        long,
        help = "Path to the config file",
        default_value = "config.json"
    )]
    pub config: Option<PathBuf>,

    #[arg(short = 'i', long, help = "Select an index of file to patch")]
    pub inf: Option<usize>,

    #[arg(
        short = 'b',
        long = "binary",
        help = "Custom input and output paths (format: \"input;output\")",
        value_parser = parse_binary_paths,
        requires = "inf"
    )]
    pub binary: Option<(String, String)>,

    #[arg(
        short = 'e',
        long = "example-config",
        help = "Print an example config file",
        conflicts_with = "config"
    )]
    pub example_config: bool,

    #[cfg(windows)]
    #[arg(short = 'k', long, help = "No Pause")]
    pub no_pause: bool,
}

// Add this function to parse the binary paths
fn parse_binary_paths(arg: &str) -> Result<(String, String), String> {
    let paths: Vec<&str> = arg.split(';').collect();
    if paths.len() != 2 {
        return Err(String::from("Binary paths must be in format: \"input;output\""));
    }
    
    let input = paths[0].trim();
    let output = paths[1].trim();
    
    if input.is_empty() || output.is_empty() {
        return Err(String::from("Input and output paths cannot be empty"));
    }
    
    Ok((input.to_string(), output.to_string()))
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Extract card from binary file
    Getcard {
        #[arg(help = "Path to the input binary file")]
        input: PathBuf,

        #[arg(help = "Offset in the binary file (hex with 0x or decimal)")]
        offset: String,

        #[arg(help = "Number of bytes to read", default_value = "128")]
        length: usize,
    },
}
