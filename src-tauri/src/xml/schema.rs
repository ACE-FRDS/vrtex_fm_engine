use std::collections::HashSet;

use roxmltree::{Document, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIssue {
    pub code: &'static str,
    pub message: String,
}

pub const STEP_DEFINITIONS: &[(u32, &str)] = &[
    (1, "Perform Script"),
    (6, "Go to Layout"),
    (7, "New Record/Request"),
    (9, "Delete Record/Request"),
    (10, "Delete All Records"),
    (17, "Go to Field"),
    (22, "Enter Find Mode"),
    (23, "Show All Records"),
    (27, "Show Omitted Only"),
    (28, "Perform Find"),
    (34, "Close File"),
    (39, "Sort Records"),
    (50, "Select All"),
    (51, "Revert Record/Request"),
    (61, "Insert Text"),
    (62, "Pause/Resume Script"),
    (68, "If"),
    (69, "Else"),
    (70, "End If"),
    (71, "Loop"),
    (72, "Exit Loop If"),
    (73, "End Loop"),
    (75, "Commit Records/Requests"),
    (76, "Set Field"),
    (79, "Freeze Window"),
    (85, "Allow User Abort"),
    (86, "Set Error Capture"),
    (87, "Show Custom Dialog"),
    (89, "# (comment)"),
    (91, "Replace Field Contents"),
    (97, "Set Zoom Level"),
    (99, "Go to Portal Row"),
    (103, "Exit Script"),
    (104, "Delete Portal Row"),
    (121, "Close Window"),
    (122, "New Window"),
    (125, "Else If"),
    (133, "Open Record/Request"),
    (141, "Set Variable"),
    (145, "Go to Object"),
    (146, "Set Web Viewer"),
    (147, "Set Field By Name"),
    (160, "Insert from URL"),
    (167, "Refresh Object"),
    (175, "Perform JavaScript in Web Viewer"),
    (188, "Get File Exists"),
    (191, "Open Data File"),
    (193, "Read from Data File"),
    (196, "Close Data File"),
    (203, "Execute FileMaker Data API"),
];

pub fn validate_filemaker_structure(
    document: &Document<'_>,
    expected_format: Option<&str>,
) -> Vec<SchemaIssue> {
    let mut issues = Vec::new();
    let root = document.root_element();

    if root.attribute("type") != Some("FMObjectList") {
        issues.push(schema_issue(
            "ROOT_TYPE_INVALID",
            "fmxmlsnippet requires type=\"FMObjectList\"",
        ));
    }

    let direct_scripts: Vec<_> = element_children(root, "Script").collect();
    let direct_steps: Vec<_> = element_children(root, "Step").collect();
    let format = expected_format.map(str::to_ascii_uppercase);

    match format.as_deref() {
        Some("XMSC") => {
            if direct_scripts.is_empty() {
                issues.push(schema_issue(
                    "XMSC_SCRIPT_REQUIRED",
                    "XMSC requires at least one direct Script child",
                ));
            }
            if !direct_steps.is_empty() {
                issues.push(schema_issue(
                    "XMSC_STEP_WRAPPER_INVALID",
                    "XMSC cannot contain Step elements directly under fmxmlsnippet",
                ));
            }
        }
        Some("XMSS") => {
            if direct_steps.is_empty() {
                issues.push(schema_issue(
                    "XMSS_STEP_REQUIRED",
                    "XMSS requires at least one direct Step child",
                ));
            }
            if !direct_scripts.is_empty() {
                issues.push(schema_issue(
                    "XMSS_SCRIPT_WRAPPER_INVALID",
                    "XMSS cannot contain a Script wrapper",
                ));
            }
        }
        _ => {}
    }

    for script in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "Script")
    {
        validate_script(script, &mut issues);
    }

    let observed_ids: HashSet<u32> = STEP_DEFINITIONS.iter().map(|(id, _)| *id).collect();
    for step in document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "Step")
    {
        validate_step(step, &observed_ids, &mut issues);
    }

    issues
}

fn validate_script(script: Node<'_, '_>, issues: &mut Vec<SchemaIssue>) {
    for attribute in [
        "includeInMenu",
        "SiriShortcutVisible",
        "runFullAccess",
        "id",
        "name",
    ] {
        if script.attribute(attribute).is_none() {
            issues.push(schema_issue(
                "SCRIPT_ATTRIBUTE_MISSING",
                format!("Script is missing required attribute {attribute}"),
            ));
        }
    }

    validate_positive_id(script, "Script", issues);
    validate_boolean_attribute(script, "includeInMenu", issues);
    validate_boolean_attribute(script, "SiriShortcutVisible", issues);
    validate_boolean_attribute(script, "runFullAccess", issues);
}

fn validate_step(step: Node<'_, '_>, observed_ids: &HashSet<u32>, issues: &mut Vec<SchemaIssue>) {
    for attribute in ["enable", "id", "name"] {
        if step.attribute(attribute).is_none() {
            issues.push(schema_issue(
                "STEP_ATTRIBUTE_MISSING",
                format!("Step is missing required attribute {attribute}"),
            ));
        }
    }

    validate_boolean_attribute(step, "enable", issues);
    let id = validate_positive_id(step, "Step", issues);

    if step
        .ancestors()
        .skip(1)
        .any(|ancestor| ancestor.is_element() && ancestor.tag_name().name() == "Step")
    {
        issues.push(schema_issue(
            "STEP_NESTED",
            "FileMaker script steps must be a flat sequence; use control steps such as End If and End Loop",
        ));
    }

    if let Some(id) = id {
        let name = step.attribute("name").unwrap_or_default();

        if name.starts_with("<unknown> [") || name.starts_with("<不明> [") {
            issues.push(schema_issue(
                "STEP_ID_UNKNOWN",
                format!("Step ID {id} is not recognized by FileMaker"),
            ));
            return;
        }

        if let Some((expected_id, _)) = STEP_DEFINITIONS
            .iter()
            .find(|(_, canonical_name)| *canonical_name == name)
        {
            if *expected_id != id {
                issues.push(schema_issue(
                    "STEP_ID_NAME_MISMATCH",
                    format!("Step name {name} uses ID {expected_id}, not {id}"),
                ));
                return;
            }
        }

        if !observed_ids.contains(&id) {
            // Unknown IDs remain forward-compatible. The validator only reports
            // structural errors; newer FileMaker versions may introduce IDs.
            return;
        }

        if id == 1 && step.attribute("name") == Some("Halt Script") {
            issues.push(schema_issue(
                "STEP_ID_NAME_MISMATCH",
                "Step ID 1 is Perform Script, not Halt Script",
            ));
        }
    }
}

fn validate_positive_id(
    node: Node<'_, '_>,
    element_name: &str,
    issues: &mut Vec<SchemaIssue>,
) -> Option<u32> {
    let value = node.attribute("id")?;
    match value.parse::<u32>() {
        Ok(id) if id > 0 => Some(id),
        _ => {
            issues.push(schema_issue(
                "OBJECT_ID_INVALID",
                format!("{element_name} id must be a positive integer"),
            ));
            None
        }
    }
}

fn validate_boolean_attribute(node: Node<'_, '_>, attribute: &str, issues: &mut Vec<SchemaIssue>) {
    let Some(value) = node.attribute(attribute) else {
        return;
    };
    if !matches!(value, "True" | "False") {
        issues.push(schema_issue(
            "BOOLEAN_ATTRIBUTE_INVALID",
            format!(
                "{} attribute {attribute} must be True or False",
                node.tag_name().name()
            ),
        ));
    }
}

fn element_children<'a, 'input: 'a>(
    node: Node<'a, 'input>,
    name: &'a str,
) -> impl Iterator<Item = Node<'a, 'input>> + 'a {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

fn schema_issue(code: &'static str, message: impl Into<String>) -> SchemaIssue {
    SchemaIssue {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(xml: &str, format: &str) -> Vec<SchemaIssue> {
        let document = Document::parse(xml).expect("test XML should parse");
        validate_filemaker_structure(&document, Some(format))
    }

    #[test]
    fn accepts_captured_xmsc_shape() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Script includeInMenu=\"True\" SiriShortcutVisible=\"False\" ",
            "runFullAccess=\"False\" id=\"7\" name=\"Example\">",
            "<Step enable=\"True\" id=\"86\" name=\"エラー処理\">",
            "<Set state=\"True\"></Set>",
            "<DisableStepCollapsed state=\"False\"></DisableStepCollapsed>",
            "</Step></Script></fmxmlsnippet>"
        );
        assert!(validate(xml, "XMSC").is_empty());
    }

    #[test]
    fn accepts_captured_xmss_shape() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Step enable=\"True\" id=\"86\" name=\"エラー処理\">",
            "<Set state=\"True\"></Set>",
            "<DisableStepCollapsed state=\"False\"></DisableStepCollapsed>",
            "</Step></fmxmlsnippet>"
        );
        assert!(validate(xml, "XMSS").is_empty());
    }

    #[test]
    fn rejects_invalid_script_step_shape() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Script id=\"1\" name=\"Bad\">",
            "<Step enable=\"True\" id=\"1\" name=\"Halt Script\" />",
            "</Script></fmxmlsnippet>"
        );
        let issues = validate(xml, "XMSC");
        assert!(issues
            .iter()
            .any(|issue| issue.code == "SCRIPT_ATTRIBUTE_MISSING"));
        assert!(issues
            .iter()
            .any(|issue| issue.code == "STEP_ID_NAME_MISMATCH"));
    }

    #[test]
    fn rejects_nested_steps() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Step enable=\"True\" id=\"71\" name=\"Loop\">",
            "<Step enable=\"True\" id=\"73\" name=\"End Loop\" />",
            "</Step></fmxmlsnippet>"
        );
        assert!(validate(xml, "XMSS")
            .iter()
            .any(|issue| issue.code == "STEP_NESTED"));
    }

    #[test]
    fn rejects_known_step_name_with_another_id() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Step enable=\"True\" id=\"6\" name=\"Perform Script\" />",
            "</fmxmlsnippet>"
        );
        assert!(validate(xml, "XMSS")
            .iter()
            .any(|issue| issue.code == "STEP_ID_NAME_MISMATCH"));
    }

    #[test]
    fn rejects_filemaker_unknown_step_marker() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Step enable=\"True\" id=\"2\" name=\"&lt;不明&gt; [2]\" />",
            "</fmxmlsnippet>"
        );
        assert!(validate(xml, "XMSS")
            .iter()
            .any(|issue| issue.code == "STEP_ID_UNKNOWN"));
    }

    #[test]
    fn accepts_localized_name_for_an_observed_id() {
        let xml = concat!(
            "<fmxmlsnippet type=\"FMObjectList\">",
            "<Step enable=\"True\" id=\"1\" name=\"スクリプト実行\" />",
            "</fmxmlsnippet>"
        );
        assert!(validate(xml, "XMSS").is_empty());
    }
}
