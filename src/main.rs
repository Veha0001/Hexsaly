use crossterm::{self, execute, terminal};
use hexsaly::app::cli;
use std::io;

fn main() {
    // Terminal
    execute!(io::stdout(), terminal::SetTitle("Hexsaly")).unwrap();

    // Enable ANSI color codes on Windows
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();
    cli::run().unwrap();
}
