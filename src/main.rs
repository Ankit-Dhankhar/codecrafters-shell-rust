use std::io::{self, Write};
use std::process;

const PROMPT: &str = "$ ";

fn main() {
    if let Err(e) = run_shell() {
        eprintln!("Shell error: {}", e);
        process::exit(1);
    }
}

fn run_shell() -> io::Result<()> {
    loop {
        print_prompt()?;

        let input = read_input()?;

        let command = input.trim();

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


fn read_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
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
        _ => {
            println!("{}: command not found", command);
        }
    }
    true
}
