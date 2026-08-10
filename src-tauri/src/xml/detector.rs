use serde::Serialize;

use super::parser::parse_filemaker_xml;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
    pub format: String,
    pub object_type: String,
    pub confidence: u8,
    pub evidence: Vec<String>,
}

pub fn detect(xml: &str) -> DetectionResult {
    let document = match parse_filemaker_xml(xml) {
        Ok(document) => document,
        Err(error) => {
            return DetectionResult {
                format: "UNKNOWN".to_owned(),
                object_type: "Unknown".to_owned(),
                confidence: 0,
                evidence: vec![error.to_string()],
            };
        }
    };

    let names: Vec<&str> = document
        .descendants()
        .filter(|node| node.is_element())
        .map(|node| node.tag_name().name())
        .collect();

    let detected = if names.contains(&"Script") {
        ("XMSC", "Script", "Script element found")
    } else if names.contains(&"Step") {
        ("XMSS", "Step", "Step element found without Script wrapper")
    } else if names
        .iter()
        .any(|name| matches!(*name, "BaseTable" | "Table"))
    {
        ("XMTB", "Table", "table definition element found")
    } else if names.contains(&"Field") {
        ("XMFD", "Field", "Field element found")
    } else if names
        .iter()
        .any(|name| matches!(*name, "Layout" | "Object"))
    {
        ("XML2", "Layout", "layout object element found")
    } else if names
        .iter()
        .any(|name| matches!(*name, "CustomFunction" | "Function"))
    {
        ("XMFN", "Custom Function", "custom function element found")
    } else if names.contains(&"Theme") {
        ("XMTH", "Theme", "Theme element found")
    } else {
        (
            "UNKNOWN",
            "Unknown",
            "no known FileMaker object signature found",
        )
    };

    DetectionResult {
        format: detected.0.to_owned(),
        object_type: detected.1.to_owned(),
        confidence: if detected.0 == "UNKNOWN" { 0 } else { 90 },
        evidence: vec![detected.2.to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_script_clipboard_xml() {
        let result = detect("<fmxmlsnippet><Script><Step /></Script></fmxmlsnippet>");
        assert_eq!(result.format, "XMSC");
        assert_eq!(result.object_type, "Script");
    }

    #[test]
    fn returns_unknown_for_valid_unrecognized_xml() {
        let result = detect("<fmxmlsnippet><SomethingNew /></fmxmlsnippet>");
        assert_eq!(result.format, "UNKNOWN");
    }
}
