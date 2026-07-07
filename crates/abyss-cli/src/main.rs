mod commands;
mod repl;

use clap::{Parser, Subcommand};
use std::fs;

use commands::{execute_format, execute_script};
use repl::start_interpreter;

#[derive(Parser)]
#[command(name = "abyss")]
#[command(about = "AbySS: Advanced-scripting by Symbolic Syntax", long_about = None)]
#[command(version = concat!("v", env!("CARGO_PKG_VERSION")))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a .aby script file
    Invoke {
        /// The path to the script file
        script: String,
        /// Arguments handed to the script as the `invocation` scroll
        /// (everything after `--`)
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Start the interactive interpreter
    Cast {
        /// Enable debug mode
        #[arg(long)]
        debug: bool,
    },
    /// Format the input script file
    Align {
        /// The path to the script file
        script: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Invoke { script, args } => {
            if let Ok(contents) = fs::read_to_string(script) {
                let code = execute_script(&contents, args);
                if code != 0 {
                    std::process::exit(code);
                }
            } else {
                eprintln!("Error: Could not read the script file.");
                std::process::exit(1);
            }
        }
        Commands::Cast { debug } => {
            start_interpreter(*debug);
        }
        Commands::Align { script } => {
            if let Ok(contents) = fs::read_to_string(script) {
                execute_format(&contents);
            } else {
                eprintln!("Error: Could not read the script file.");
            }
        }
    }
}
