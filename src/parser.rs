pub struct Redirection {
    pub stdout_file: Option<String>,
    pub stdout_append: bool,
    pub stderr_file: Option<String>,
    pub stderr_append: bool,
}

pub fn parse_redirection(parts: Vec<String>) -> (Vec<String>, Redirection) {
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

pub fn parse_arguments(input: &str) -> Vec<String> {
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
