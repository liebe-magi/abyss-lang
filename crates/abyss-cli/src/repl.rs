//! The `cast` subcommand: an interactive REPL driver.
//!
//! Reads statements line by line, using brace counting to decide when a
//! multi-line statement is complete (the prompt indents accordingly, so
//! this must stay in sync with the language's block syntax). History is
//! persisted under `~/.abyss/`.

use abyss_core::{format::format_stmt, parser::parse};
use abyss_interpreter::{
    env::Value,
    eval::{EvalResult, evaluate},
    stdlib,
};
use colored::*;
use rustyline::Editor;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use std::fs;
use std::path::PathBuf;

use crate::commands::report_diagnostics;

/// Sets up the AbySS configuration directory in the user's home directory.
/// This directory is used to store configuration files such as history logs.
///
/// # Returns
/// A `PathBuf` representing the path to the AbySS directory.
fn setup_abyss_directory() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Unable to find home directory");
    let abyss_dir = home_dir.join(".abyss");

    if !abyss_dir.exists() {
        fs::create_dir_all(&abyss_dir).expect("Unable to create ~/.abyss directory");
    }

    abyss_dir
}

/// Returns the path to the AbySS history file stored in the AbySS directory.
///
/// # Returns
/// A `PathBuf` representing the path to the history file.
fn get_history_file_path() -> PathBuf {
    let abyss_dir = setup_abyss_directory();
    abyss_dir.join("abyss_history.log")
}

/// Starts the interactive AbySS interpreter, allowing the user to enter and execute AbySS code line by line.
///
/// # Arguments
/// * `debug` - A boolean flag to enable debug mode, which prints the AST of the parsed code.
pub fn start_interpreter(debug: bool) {
    println!("Starting AbySS interpreter...");
    println!("Type 'exit' or press Ctrl+D to exit the interpreter.\n");

    let mut current_session_code = String::new();
    let mut current_statement = String::new();
    let mut env = stdlib::create_global_environment();

    let history_path = get_history_file_path();
    let mut rl = Editor::<(), FileHistory>::new().expect("Error: Failed to create editor");
    let _ = rl.load_history(&history_path);
    let _ = rl.set_max_history_size(1000);

    loop {
        let prompt = format!(
            "AbySS > {}",
            "  ".repeat(
                current_statement.matches('{').count() - current_statement.matches('}').count()
            ),
        )
        .blue()
        .bold();
        let readline = rl.readline(&prompt.to_string());

        match readline {
            Ok(line) => {
                match line.trim() {
                    "exit" => {
                        println!("EXIT: Exiting interpreter...");
                        break;
                    }
                    "clear" => {
                        current_statement.clear();
                        continue;
                    }
                    "show" => {
                        println!("=== Current Session Code ===");
                        println!("{}", &current_session_code);
                        println!("============================");
                        current_statement.clear();
                        continue;
                    }
                    _ => {}
                }

                match rl.add_history_entry(line.as_str()) {
                    Ok(_) => {}
                    Err(err) => println!("Failed to add history: {:?}", err),
                }

                current_statement.push_str(&line);

                let open_braces = current_statement.matches('{').count();
                let close_braces = current_statement.matches('}').count();

                if open_braces == close_braces && current_statement.ends_with(';') {
                    let outcome = parse(&current_statement);
                    if report_diagnostics("<repl>", &current_statement, &outcome.diagnostics) {
                        current_statement.clear();
                        continue;
                    }

                    for ast in outcome.ast {
                        if debug {
                            println!("{}", format!("AST: {:?}", ast).yellow());
                        }

                        match evaluate(&ast, &mut env) {
                            Ok(result) => {
                                current_session_code.push_str(&format_stmt(&ast, 0));
                                current_session_code.push('\n');
                                match result {
                                    EvalResult::Data(Value::Omen(true)) => {
                                        println!("{}", "boon".green())
                                    }
                                    EvalResult::Data(Value::Omen(false)) => {
                                        println!("{}", "hex".green())
                                    }
                                    EvalResult::Data(Value::Arcana(n)) => {
                                        println!("{}", format!("{}", n).green())
                                    }
                                    EvalResult::Data(Value::Aether(n)) => {
                                        println!("{}", format!("{}", n).green())
                                    }
                                    EvalResult::Data(Value::Rune(r)) => {
                                        println!("{}", r.as_ref().green())
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                println!("{}", format!("Error: {}", e).red());
                            }
                        }
                    }

                    current_statement.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C: Restarting interpreter...");
                current_session_code.clear();
                current_statement.clear();
                env = stdlib::create_global_environment();
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D: Exiting interpreter...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_path)
        .expect("Error: Failed to save history");
}
