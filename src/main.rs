use std::io::{self, Write};
use std::process;

const PROMPT: &str = "$ ";
const BUILTINS: [&str; 3] = ["echo", "exit", "type"];

fn main() {
    if let Err(e) = run_shell() {
        eprintln!("Shell error: {}", e);
        process::exit(1);
    }
}

fn run_shell() -> io::Result<()> {
    loop {
        print_prompt()?;

        let input = match read_input()? {
            Some(input) => input,
            None => break,
        };

        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        if !execute_command(command) {
            break;
        }
    }
    Ok(())
}

fn print_prompt() -> io::Result<()> {
    print!("{}", PROMPT);
    io::stdout().flush()
}


fn read_input() -> io::Result<Option<String>> {
    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;
    if bytes_read == 0 {
        return Ok(None);
    }
    Ok(Some(input))
}

fn execute_command(command: &str) -> bool {
    let parts: Vec<&str> = command.split_whitespace().collect();



    match parts.first() {
        Some(&"exit") => {
            process::exit(0);
        }
        Some(&"echo") => {
            if let Some(rest) = command.strip_prefix("echo").map(|s| s.trim()) {
                println!("{}", rest);
            }
        }
        Some(&"type") => {
            if BUILTINS.contains(&parts[1]) {
                println!("{} is a shell builtin", parts[1]);
            } else {
                println!("{}: not found", parts[1]);
            }
        }
        _ => {
            println!("{}: command not found", command);
        }
    }
    true
}
