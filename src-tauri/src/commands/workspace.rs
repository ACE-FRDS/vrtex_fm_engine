use std::path::Path;

const WORKSPACE_EXTENSION: &str = "vfe-workspace";
const MAX_WORKSPACE_BYTES: usize = 128 * 1024 * 1024;

#[tauri::command]
pub fn save_workspace_file(path: String, contents: String) -> Result<(), String> {
    if contents.len() > MAX_WORKSPACE_BYTES {
        return Err("Workspace file exceeds the 128 MB limit".to_string());
    }

    let destination = Path::new(&path);
    if !destination.is_absolute() {
        return Err("Workspace destination must be an absolute path".to_string());
    }
    if destination.extension().and_then(|value| value.to_str()) != Some(WORKSPACE_EXTENSION) {
        return Err(format!("Workspace files must use .{WORKSPACE_EXTENSION}"));
    }
    if !destination.parent().is_some_and(Path::is_dir) {
        return Err("Workspace destination folder does not exist".to_string());
    }

    std::fs::write(destination, contents)
        .map_err(|error| format!("Failed to save workspace file: {error}"))
}

#[tauri::command]
pub fn read_workspace_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}
