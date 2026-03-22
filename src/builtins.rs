use std::{env, path::Path};

use crate::parser::Redirection;
use crate::utils::{get_executable_path, write_to_file};

pub const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

pub fn handle_echo(parts: &[String], redirection: &Redirection) {
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
        write_to_file(
            filename,
            &format!("{}\n", output),
            redirection.stdout_append,
        );
    } else {
        println!("{}", output);
    }
}

pub fn handle_pwd(redirection: &Redirection) {
    let output = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => format!("pwd: error retrieving current directory: {}", e),
    };

    if let Some(filename) = &redirection.stdout_file {
        write_to_file(
            filename,
            &format!("{}\n", output),
            redirection.stdout_append,
        );
    } else {
        println!("{}", output);
    }
}

pub fn handle_cd(parts: &[String]) {
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

pub fn handle_type(parts: &[String], redirection: &Redirection) {
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
        write_to_file(
            filename,
            &format!("{}\n", output),
            redirection.stdout_append,
        );
    } else {
        println!("{}", output);
    }
}

pub fn is_internal_builtin(command: &str) -> bool {
    BUILTINS.contains(&command)
}
