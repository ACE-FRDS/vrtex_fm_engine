use tauri::State;

use crate::ai::{
    build_prompt, provider_status, run_provider, test_provider_connection, AiConnectionTest,
    AiProviderRequest, AiProviderResponse, AiProviderStatus,
};
use crate::database::ai_models::{
    AiMessage, AiSession, AiSessionDetail, CreateAiSession, RagDocument, SaveAiMessage,
    UpdateAiSession,
};
use crate::database::ai_repository;
use crate::AppState;

#[tauri::command]
pub fn list_ai_sessions(
    state: State<'_, AppState>,
    project_id: String,
    limit: Option<usize>,
) -> Result<Vec<AiSession>, String> {
    state
        .database
        .with_connection(|connection| {
            ai_repository::list_sessions(connection, &project_id, limit.unwrap_or(100))
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_ai_session(
    state: State<'_, AppState>,
    session: CreateAiSession,
) -> Result<AiSession, String> {
    state
        .database
        .with_connection(|connection| ai_repository::create_session(connection, session))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_ai_session(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<AiSessionDetail>, String> {
    state
        .database
        .with_connection(|connection| ai_repository::load_detail(connection, &id))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_ai_message(
    state: State<'_, AppState>,
    message: SaveAiMessage,
) -> Result<AiMessage, String> {
    state
        .database
        .with_connection(|connection| ai_repository::save_message(connection, message))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_ai_session(
    state: State<'_, AppState>,
    id: String,
    changes: UpdateAiSession,
) -> Result<AiSession, String> {
    state
        .database
        .with_connection(|connection| ai_repository::update_session(connection, &id, changes))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn search_ai_rag(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<RagDocument>, String> {
    state
        .database
        .with_connection(|connection| {
            ai_repository::search_rag(connection, &query, limit.unwrap_or(8))
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_ai_provider_status(state: State<'_, AppState>) -> Vec<AiProviderStatus> {
    provider_status(&state.credentials)
}

#[tauri::command]
pub fn save_openai_api_key(
    state: State<'_, AppState>,
    api_key: String,
) -> Result<AiProviderStatus, String> {
    state.credentials.save_openai_key(&api_key)?;
    provider_status(&state.credentials)
        .into_iter()
        .find(|provider| provider.id == "openai")
        .ok_or_else(|| "OpenAI Providerが見つかりません".to_owned())
}

#[tauri::command]
pub fn delete_openai_api_key(state: State<'_, AppState>) -> Result<bool, String> {
    state.credentials.delete_openai_key()
}

#[tauri::command]
pub async fn test_ai_provider_connection(
    state: State<'_, AppState>,
    provider: String,
) -> Result<AiConnectionTest, String> {
    test_provider_connection(&provider, &state.credentials).await
}

#[tauri::command]
pub async fn run_ai_assistant(
    state: State<'_, AppState>,
    request: AiProviderRequest,
) -> Result<AiProviderResponse, String> {
    let prompt = build_prompt(&request);
    run_provider(&request, prompt, &state.credentials).await
}
