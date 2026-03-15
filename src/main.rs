use std::{
    env,
    fs,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process,
};


const PROMPT: &str = "$ ";
const BUILTINS: [&str; 4] = ["echo", "exit", "type", "pwd"];

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
        Some(&"pwd") => {
            println!("{}", env::current_dir().unwrap().to_string_lossy());
        }
        Some(&"type") => {
            if is_internal_builtin(parts[1]) {
                println!("{} is a shell builtin", parts[1]);
            } else if is_external_command(parts[1]) {
                println!("{} is {}", parts[1], get_executable_path(parts[1]).unwrap());
            } else {
                println!("{}: not found", parts[1]);
            }
        }
        Some(command) => {
            if is_external_command(command) {
                run_external_command(command, &parts[1..]);
            } else {
                println!("{}: command not found", command);
            }
        }
        _ => {
            println!("{}: command not found", command);
        }
    }
    true
}

fn is_internal_builtin(command: &str) -> bool {
    BUILTINS.contains(&command)
}

fn is_external_command(command: &str) -> bool {
    !is_internal_builtin(command) && get_executable_path(command).is_some()
}

fn get_executable_path(command: &str) -> Option<String> {
    let path_var = env::var("PATH").ok()?;

    for dir in path_var.split(':') {
        let full_path = Path::new(dir).join(command);
        if full_path.exists() && is_executable(&full_path) {
            return Some(full_path.to_string_lossy().to_string());
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

fn run_external_command(command: &str, args: &[&str]) -> bool {
    let mut command = process::Command::new(command);
    command.args(args);
    command.spawn().and_then(|mut child| child.wait())
    .map_or(false, |status| status.success())
}