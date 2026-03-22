use std::{
    env, fs,
    fs::OpenOptions,
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
    let mut executables = Vec::new();

    let path_var =  match env::var("PATH") {
        Ok(path) => path,
        Err(_) => return executables,
    };

    for dir in path_var.split(':') {
        let dir_path = Path::new(dir);

        // Skip if directory doesn't exist or can't be read
        let enteries = match fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in enteries.flatten() {
            let path = entry.path();

            // Only include files (not directories) that are executable
            if path.is_file() && is_executable(&path) {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        executables.push(name_str.to_string());
                    }
                }
            }
        }
    }
    executables.sort();
    executables.dedup();

    executables
}
