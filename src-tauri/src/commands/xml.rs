use crate::xml::validator::ValidationReport;
use crate::xml::{detector, preview, validator};

#[tauri::command]
pub fn detect_xml_format(xml: String) -> detector::DetectionResult {
    detector::detect(&xml)
}

#[tauri::command]
pub fn validate_filemaker_xml(xml: String, format: Option<String>) -> ValidationReport {
    validator::validate(&xml, format.as_deref())
}

#[tauri::command]
pub fn preview_filemaker_xml(xml: String) -> Result<preview::ScriptPreview, String> {
    preview::build_script_preview(&xml).map_err(|error| error.to_string())
}
