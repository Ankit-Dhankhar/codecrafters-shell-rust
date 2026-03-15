use std::{
    env, fs,
    fs::OpenOptions,
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

struct Redirection {
    stdout_file: Option<String>,
    stdout_append: bool,
    stderr_file: Option<String>,
    stderr_append: bool,
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
    let (parts, redirection) = parse_redirection(parts);

    match parts.first().map(String::as_str) {
        Some("exit") => process::exit(0),
        Some("echo") => handle_echo(&parts, &redirection),
        Some("pwd") => handle_pwd(&redirection),
        Some("cd") => handle_cd(&parts),
        Some("type") => handle_type(&parts, &redirection),
        Some(cmd) => handle_external_or_unknown(cmd, &parts, &redirection),
        None => {}
    }
    true
}

fn parse_redirection(parts: Vec<String>) -> (Vec<String>, Redirection) {
    let mut command_parts = Vec::new();
    let mut redirection = Redirection {
        stdout_file: None,
        stdout_append: false,
        stderr_file: None,
        stderr_append: false,
    };

    let mut iter = parts.into_iter().peekable();

    while let Some(part) = iter.next() {
        if part == ">" || part == "1>" {
            if let Some(filename) = iter.next() {
                redirection.stdout_file = Some(filename);
            }
        } else if part == ">>" || part == "1>>" {
            if let Some(filename) = iter.next() {
                redirection.stdout_file = Some(filename);
                redirection.stdout_append = true;
            }
        } else if part == "2>" {
            if let Some(filename) = iter.next() {
                redirection.stderr_file = Some(filename);
            }
        } else if part == "2>>" {
            if let Some(filename) = iter.next() {
                redirection.stderr_file = Some(filename);
                redirection.stderr_append = true;
            }
        } else {
            command_parts.push(part);
        }
    }
    (command_parts, redirection)
}

fn parse_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut token_in_progress = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            token_in_progress = true;
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if !in_single_quotes => {
                escape_next = true;
                token_in_progress = true;
            }
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

fn handle_echo(parts: &[String], redirection: &Redirection) {
    let output = format!(
        "{}",
        parts
            .iter()
            .skip(1)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    );

    if let Some(filename) = &redirection.stderr_file {
        write_to_file(filename, "", redirection.stderr_append);
    }

    if let Some(filename) = &redirection.stdout_file {
        write_to_file(filename, &format!("{}\n", output), redirection.stdout_append);
    } else {
        println!("{}", output);
    }
}

fn handle_pwd(redirection: &Redirection) {
    let output = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => format!("pwd: error retrieving current directory: {}", e),
    };

    if let Some(filename) = &redirection.stdout_file {
        write_to_file(filename, &format!("{}\n", output), redirection.stdout_append);
    } else {
        println!("{}", output);
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

fn handle_type(parts: &[String], redirection: &Redirection) {
    let mut output = String::new();
    if let Some(cmd) = parts.get(1) {
        if is_internal_builtin(cmd) {
            output.push_str(&format!("{} is a shell builtin", cmd));
        } else if let Some(path) = get_executable_path(cmd) {
            output.push_str(&format!("{} is {}", cmd, path));
        } else {
            output.push_str(&format!("{}: not found", cmd));
        }
    }

    if let Some(filename) = &redirection.stdout_file {
        write_to_file(filename, &format!("{}\n", output), redirection.stdout_append);
    } else {
        println!("{}", output);
    }
}

fn handle_external_or_unknown(cmd: &str, parts: &[String], redirection: &Redirection) {
    if is_external_command(cmd) {
        run_external_command(cmd, &parts[1..], &redirection);
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

fn open_output_file(filename: &str, append: bool) -> fs::File {
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(filename)
        .unwrap()
}

fn write_to_file(filename: &str, content: &str, append: bool) {
    let mut file = open_output_file(filename, append);
    write!(file, "{}", content).unwrap();
}

fn run_external_command(command: &str, args: &[String], redirection: &Redirection) -> bool {
    let mut cmd = process::Command::new(command);
    cmd.args(args);

    if let Some(filename) = &redirection.stdout_file {
        cmd.stdout(open_output_file(filename, redirection.stdout_append));
    }

    if let Some(filename) = &redirection.stderr_file {
        cmd.stderr(open_output_file(filename, redirection.stderr_append));
    }

    cmd.spawn()
        .and_then(|mut child| child.wait())
        .map_or(false, |status| status.success())
}
