use serde::Serialize;

use super::parser::{parse_filemaker_xml, XmlError};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewStep {
    pub index: usize,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPreview {
    pub name: String,
    pub steps: Vec<PreviewStep>,
}

pub fn build_script_preview(xml: &str) -> Result<ScriptPreview, XmlError> {
    let document = parse_filemaker_xml(xml)?;
    let script = document
        .descendants()
        .find(|node| node.has_tag_name("Script"));
    let name = script
        .and_then(|node| node.attribute("name"))
        .unwrap_or("Untitled Script")
        .to_owned();
    let steps = document
        .descendants()
        .filter(|node| node.has_tag_name("Step"))
        .enumerate()
        .map(|(index, node)| PreviewStep {
            index: index + 1,
            name: node.attribute("name").unwrap_or("Unknown Step").to_owned(),
            enabled: node
                .attribute("enable")
                .map(|value| value != "False")
                .unwrap_or(true),
        })
        .collect();
    Ok(ScriptPreview { name, steps })
}
