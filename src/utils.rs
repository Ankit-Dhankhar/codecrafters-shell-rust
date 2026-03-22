use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
};

pub fn get_executable_path(command: &str) -> Option<String> {
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

pub fn open_output_file(filename: &str, append: bool) -> fs::File {
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(filename)
        .unwrap()
}

pub fn write_to_file(filename: &str, content: &str, append: bool) {
    let mut file = open_output_file(filename, append);
    write!(file, "{}", content).unwrap();
}

pub fn get_all_executable_paths() -> Vec<String> {
    let path_var = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => return Vec::new(),
    };

    let executables: Vec<String> = path_var
        .split(":")
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && is_executable(&path) {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    executables
}
