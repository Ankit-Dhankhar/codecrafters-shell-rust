use std::{fs, env};
use rustyline::completion::Pair;

pub fn complete_filename(partial: &str) -> Vec<Pair> {
    let mut matches = Vec::new();

    let expanded = if partial.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            partial.replacen("~/", &home, 1)
        } else {
            partial.to_string()
        }
    } else {
        partial.to_string()
    };

    let (dir_path, file_prefix) = if let Some(last_slash) = expanded.rfind('/') {
        let dir = if last_slash == 0 {
            "/"
        } else {
            &expanded[..last_slash]
        };
        let prefix = &expanded[last_slash + 1..];
        (dir.to_string(), prefix.to_string())
    } else {
        (".".to_string(), expanded.to_string())
    };

    let enteries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => return matches,
    };

    for entry in enteries.flatten() {
        let file_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        if file_name.starts_with(".") && !file_prefix.starts_with(".") {
            continue;
        }

        if file_name.starts_with(&file_prefix) {
            let full_path = entry.path();
            let is_dir = full_path.is_dir();

            let replacement = if partial.contains('/') {
                let dir_part = &partial[..partial.rfind('/').unwrap() + 1];
                if is_dir {
                    format!("{}{}/ ", dir_part, file_name)
                } else {
                    format!("{}{} ", dir_part, file_name)
                }
            } else {
                if is_dir {
                    format!("{}/ ", file_name)
                } else {
                    format!("{} ", file_name)
                }
            };

            matches.push(Pair {
                display: if is_dir {
                    format!("{}/", file_name)
                } else {
                    file_name.clone()
                },
                replacement,
            });
        }
    }
    matches.sort_by(|a, b| a.display.cmp(&b.display));
    matches
}

