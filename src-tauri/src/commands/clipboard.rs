use crate::clipboard::{platform_provider, ClipboardPayload};
use crate::xml::validator;

#[tauri::command]
pub fn get_filemaker_clipboard() -> Result<ClipboardPayload, String> {
    platform_provider()
        .get_filemaker_clipboard()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_filemaker_clipboard(format: String, xml: String) -> Result<(), String> {
    let validation = validator::validate(&xml, Some(&format));
    if !validation.valid {
        let messages = validation
            .issues
            .iter()
            .filter(|issue| matches!(issue.level, crate::xml::validator::ValidationLevel::Error))
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("FileMaker XML validation failed: {messages}"));
    }
    platform_provider()
        .set_filemaker_clipboard(&format, &xml)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_xml_before_touching_the_live_clipboard() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Script id=\"1\" name=\"Invalid\">",
            "<Step enable=\"True\" id=\"1\" name=\"Halt Script\" />",
            "</Script></fmxmlsnippet>"
        );
        let error = set_filemaker_clipboard("XMSC".to_owned(), xml.to_owned())
            .expect_err("invalid XML must be rejected before clipboard access");
        assert!(error.contains("validation failed"));
        assert!(error.contains("Step ID 1 is Perform Script"));
    }
}
