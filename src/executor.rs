use std::process;

use crate::builtins::{
    handle_cd, handle_echo, handle_pwd, handle_type, is_internal_builtin,
};
use crate::parser::{parse_arguments, parse_redirection, Redirection};
use crate::utils::{get_executable_path, open_output_file};

pub fn execute_command(command: &str) -> bool {
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

fn handle_external_or_unknown(cmd: &str, parts: &[String], redirection: &Redirection) {
    if is_external_command(cmd) {
        run_external_command(cmd, &parts[1..], &redirection);
    } else {
        println!("{}: command not found", cmd);
    }
}

fn is_external_command(command: &str) -> bool {
    !is_internal_builtin(command) && get_executable_path(command).is_some()
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
