use super::prompt_builder::{KnowledgePromptBuilder, PromptBuildInput};
use super::provider::AiProviderRequest;

const SYSTEM_INSTRUCTIONS: &str = r#"You are the VRTEX FM Engine AI Assistant, a FileMaker development specialist.
Analyze requirements, design changes, and propose FileMaker Clipboard XML when appropriate.
Do not claim that a change was applied. VRTEX FM Engine performs validation, approval, and delivery.
Never expose credentials or assume that external state was changed."#;

const EXPECTED_RESPONSE_SCHEMA: &str = r#"Return a concise Change Plan before generated XML.
If XML is generated, return exactly one complete fmxmlsnippet in a fenced xml block.
Use FileMaker Clipboard XML casing and attributes exactly.
The result remains a proposal until VRTEX validation and user approval pass."#;

pub fn build_prompt(request: &AiProviderRequest) -> String {
    let rag_context = if request.rag_context.is_empty() {
        "No matching legacy RAG documents were found.".to_owned()
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
    let project_context = format!(
        "Project: {project_id}\nArchitecture: SRC preserves external specifications, CORE transforms internal models, UI is data-source independent.\n\nCurrent XML:\n```xml\n{current_xml}\n```\n\nLegacy RAG context:\n{rag_context}",
        project_id = request.project_id,
    );
    let task_instructions = format!(
        "Mode: {mode}\nDry Run: {dry_run}\nSelected format: {format}\n\n- Inspect current structure before proposing changes.\n- Preserve existing objects and data.\n- Never perform destructive changes automatically.",
        mode = request.mode,
        dry_run = if request.dry_run { "ON" } else { "OFF" },
        format = request.format.as_deref().unwrap_or("auto"),
    );

    // Phase 2 owns task classification and pack selection. The dedicated builder is
    // integrated now, while the current request path deliberately selects no pack.
    KnowledgePromptBuilder
        .build(PromptBuildInput {
            system_instructions: SYSTEM_INSTRUCTIONS,
            selected_knowledge_packs: &[],
            project_context: &project_context,
            task_instructions: &task_instructions,
            user_request: &request.user_prompt,
            expected_response_schema: EXPECTED_RESPONSE_SCHEMA,
        })
        .content
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
        assert!(prompt.contains("SYSTEM INSTRUCTIONS"));
        assert!(prompt.contains("PROJECT CONTEXT"));
        assert!(prompt.contains("SELECTED KNOWLEDGE PACKS"));
        assert!(prompt.contains("USER REQUEST"));
        assert!(prompt.contains("EXPECTED RESPONSE SCHEMA"));
        assert!(prompt.contains("Dry Run: ON"));
    }
}
