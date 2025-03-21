use std::path::PathBuf;
// Code for parsing command line arguments
#[derive(Debug, clap::Parser)]
#[command(
    name = "Hexsaly",
    about = "A tool to patch binary files based on a configuration file.\nMade by Veha0001.",
    version,
    author
)]
pub struct Args {
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
