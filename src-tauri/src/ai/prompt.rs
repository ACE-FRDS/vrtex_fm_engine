use super::provider::AiProviderRequest;

pub fn build_prompt(request: &AiProviderRequest) -> String {
    let rag_context = if request.rag_context.is_empty() {
        "No matching RAG documents were found.".to_owned()
    } else {
        request
            .rag_context
            .iter()
            .enumerate()
            .map(|(index, source)| format!("[{}] {}", index + 1, source))
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    let current_xml = request
        .current_xml
        .as_deref()
        .filter(|xml| !xml.trim().is_empty())
        .unwrap_or("No XML is currently selected.");
    format!(
        r#"SYSTEM CONTEXT
You are the Vertex FM ENGINE AI Assistant, a FileMaker development specialist.
Analyze requirements, design changes, and propose FileMaker Clipboard XML when appropriate.
Do not claim that a change was applied. Vertex FM ENGINE performs validation, approval, and delivery.

PROJECT CONTEXT
Project: {project_id}
Architecture: SRC preserves external specifications, CORE transforms internal models, UI is data-source independent.

TASK CONTEXT
Mode: {mode}
Dry Run: {dry_run}
Selected format: {format}

FILEMAKER CONTEXT
Current XML:
```xml
{current_xml}
```

RAG CONTEXT
{rag_context}

USER REQUEST
{user_prompt}

EXECUTION POLICY
- Inspect current structure before proposing changes.
- Preserve existing objects and data.
- Never perform destructive changes automatically.
- Return a concise Change Plan before generated XML.
- If XML is generated, return one complete fmxmlsnippet in a fenced xml block.
- Use FileMaker Clipboard XML casing and attributes exactly.
- The result remains a proposal until Vertex validation and user approval pass.
"#,
        project_id = request.project_id,
        mode = request.mode,
        dry_run = if request.dry_run { "ON" } else { "OFF" },
        format = request.format.as_deref().unwrap_or("auto"),
        current_xml = current_xml,
        rag_context = rag_context,
        user_prompt = request.user_prompt,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separates_prompt_layers() {
        let prompt = build_prompt(&AiProviderRequest {
            provider: "openai".to_owned(),
            model: "gpt-5.6-terra".to_owned(),
            project_id: "VertexProject".to_owned(),
            mode: "DESIGN".to_owned(),
            dry_run: true,
            format: Some("XMSC".to_owned()),
            current_xml: Some("<fmxmlsnippet />".to_owned()),
            rag_context: vec!["XMSC rule".to_owned()],
            user_prompt: "Review this".to_owned(),
        });
        assert!(prompt.contains("SYSTEM CONTEXT"));
        assert!(prompt.contains("PROJECT CONTEXT"));
        assert!(prompt.contains("RAG CONTEXT"));
        assert!(prompt.contains("USER REQUEST"));
        assert!(prompt.contains("Dry Run: ON"));
    }
}
