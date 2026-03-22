use std::{io, process};

use rustyline::{CompletionType, Config, Editor, error::ReadlineError};

mod builtins;
mod completer;
mod executor;
mod parser;
mod trie;
mod utils;

use completer::ShellCompleter;
use executor::execute_command;

const PROMPT: &str = "$ ";

fn main() {
    if let Err(e) = run_shell() {
        eprintln!("Shell error: {}", e);
        process::exit(1);
    }
}

fn run_shell() -> io::Result<()> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl = Editor::with_config(config).expect("failed to create editor");
    rl.set_helper(Some(ShellCompleter::new()));

    loop {
        match rl.readline(PROMPT) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let command = line.trim();
                if command.is_empty() {
                    continue;
                }

                if !execute_command(command) {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(error) => {
                eprintln!("Error: {:?}", error);
                break;
            }
        }
    }
    Ok(())
}
