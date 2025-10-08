use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "Hexsaly", about = "Binary Patcher", version, author)]
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
    /// Open a binary file for patching
    Open {
        #[arg(help = "Input binary path")]
        input: PathBuf,

        #[arg(short = 'i', long = "int", help = "Patch config index")]
        index: Option<usize>,
    },
}
