use tauri::State;

use crate::database::models::{ClipboardItem, SaveClipboardItem};
use crate::database::repository;
use crate::AppState;

#[tauri::command]
pub fn list_clipboard_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardItem>, String> {
    state
        .database
        .with_connection(|connection| repository::list_history(connection, limit.unwrap_or(200)))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_library_items(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardItem>, String> {
    state
        .database
        .with_connection(|connection| repository::list_library(connection, limit.unwrap_or(500)))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_clipboard_item(
    state: State<'_, AppState>,
    item: SaveClipboardItem,
) -> Result<ClipboardItem, String> {
    state
        .database
        .with_connection(|connection| repository::save(connection, item))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_clipboard_item(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<ClipboardItem>, String> {
    state
        .database
        .with_connection(|connection| repository::load(connection, &id))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_clipboard_item(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    state
        .database
        .with_connection(|connection| repository::delete(connection, &id))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_clipboard_favorite(
    state: State<'_, AppState>,
    id: String,
    favorite: bool,
) -> Result<bool, String> {
    state
        .database
        .with_connection(|connection| repository::update_favorite(connection, &id, favorite))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_clipboard_notes(
    state: State<'_, AppState>,
    id: String,
    notes: String,
) -> Result<bool, String> {
    state
        .database
        .with_connection(|connection| repository::update_notes(connection, &id, &notes))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_clipboard_history(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .database
        .with_connection_mut(repository::clear_clipboard)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_library_data(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .database
        .with_connection_mut(repository::clear_library)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_all_data(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .database
        .with_connection_mut(repository::clear_all)
        .map_err(|error| error.to_string())
}
