use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMakerStatus {
    detected: bool,
    version: Option<String>,
    display_name: String,
}

#[tauri::command]
pub fn detect_filemaker() -> FileMakerStatus {
    detect_filemaker_process()
}

#[cfg(target_os = "windows")]
fn detect_filemaker_process() -> FileMakerStatus {
    let detected = is_filemaker_running();
    FileMakerStatus {
        detected,
        version: None,
        display_name: if detected {
            "FileMaker Pro"
        } else {
            "FileMaker"
        }
        .to_owned(),
    }
}

#[cfg(target_os = "windows")]
fn is_filemaker_running() -> bool {
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    // ToolHelp reads the process list directly and, unlike spawning `tasklist`,
    // never opens a console window while the app is polling FileMaker status.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    let mut detected = false;
    let mut has_process = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_process {
        let name_length = entry
            .szExeFile
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(entry.szExeFile.len());
        let process_name = String::from_utf16_lossy(&entry.szExeFile[..name_length]);
        if is_filemaker_process_name(&process_name) {
            detected = true;
            break;
        }
        has_process = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe { CloseHandle(snapshot) };
    detected
}

#[cfg(target_os = "windows")]
fn is_filemaker_process_name(process_name: &str) -> bool {
    matches!(
        process_name.to_ascii_lowercase().as_str(),
        "filemaker pro.exe" | "filemakerpro.exe" | "filemaker.exe"
    )
}

#[cfg(not(target_os = "windows"))]
fn detect_filemaker_process() -> FileMakerStatus {
    FileMakerStatus {
        detected: false,
        version: None,
        display_name: "FileMaker".to_owned(),
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::is_filemaker_process_name;

    #[test]
    fn recognizes_filemaker_executable_names_case_insensitively() {
        assert!(is_filemaker_process_name("FileMaker Pro.exe"));
        assert!(is_filemaker_process_name("FILEMAKERPRO.EXE"));
        assert!(is_filemaker_process_name("filemaker.exe"));
        assert!(!is_filemaker_process_name("filemaker-server.exe"));
        assert!(!is_filemaker_process_name("tasklist.exe"));
    }
}
