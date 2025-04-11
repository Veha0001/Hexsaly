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
