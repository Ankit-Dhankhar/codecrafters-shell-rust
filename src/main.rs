use std::{
    env,
    fs,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process,
};


const PROMPT: &str = "$ ";
const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

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
    let parts = parse_arguments(command);

    match parts.first().map(String::as_str) {
        Some("exit") => process::exit(0),
        Some("echo") => handle_echo(&parts),
        Some("pwd") => handle_pwd(),
        Some("cd") => handle_cd(&parts),
        Some("type") => handle_type(&parts),
        Some(cmd) => handle_external_or_unknown(cmd, &parts),
        None => {}
    }
    true
}

fn parse_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut token_in_progress = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                token_in_progress = true;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                token_in_progress = true;
            }
            _ if ch.is_whitespace() && !in_single_quotes && !in_double_quotes => {
                if token_in_progress {
                    args.push(std::mem::take(&mut current));
                    token_in_progress = false;
                }
            }
            _ => {
                current.push(ch);
                token_in_progress = true;
            }
        }
    }

    if token_in_progress {
        args.push(current);
    }

    args
}

fn handle_echo(parts: &[String]) {
    println!(
        "{}",
        parts
            .iter()
            .skip(1)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    );
}

fn handle_pwd() {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("pwd: error retrieving current directory: {}", e),
    }
}

fn handle_cd(parts: &[String]) {
    let target_raw = parts.get(1).map(String::as_str).unwrap_or("/");

    let target_path = if target_raw == "~" {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        target_raw.to_string()
    };

    let path = Path::new(&target_path);

    if path.is_dir() {
        if let Err(e) = env::set_current_dir(&path) {
            println!("cd: {}: {}", target_path, e);
        }
    } else {
        println!("cd: {}: No such file or directory", target_path);
    }
}

fn handle_type(parts: &[String]) {
    if let Some(cmd) = parts.get(1) {
        if is_internal_builtin(cmd) {
            println!("{} is a shell builtin", cmd);
        } else if let Some(path) = get_executable_path(cmd) {
            println!("{} is {}", cmd, path);
        } else {
            println!("{}: not found", cmd);
        }
    }
}

fn handle_external_or_unknown(cmd: &str, parts: &[String]) {
    if is_external_command(cmd) {
        run_external_command(cmd, &parts[1..]);
    } else {
        println!("{}: command not found", cmd);
    }
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

fn run_external_command(command: &str, args: &[String]) -> bool {
    let mut command = process::Command::new(command);
    command.args(args);
    command.spawn().and_then(|mut child| child.wait())
    .map_or(false, |status| status.success())
}