use chrono::Utc;
use rusqlite::{params, Connection, Result};

const INITIAL_SCHEMA: &str = include_str!("../../migrations/001_initial.sql");
const LIBRARY_STATE_SCHEMA: &str = include_str!("../../migrations/002_library_state.sql");
const HISTORY_STATE_SCHEMA: &str = include_str!("../../migrations/003_history_state.sql");
const AI_ASSISTANT_SCHEMA: &str = include_str!("../../migrations/004_ai_assistant.sql");
const KNOWLEDGE_PACK_SCHEMA: &str = include_str!("../../migrations/005_knowledge_packs.sql");
const INITIAL_VERSION: i64 = 1;

pub fn run(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;
    let applied: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
        [INITIAL_VERSION],
        |row| row.get(0),
    )?;
    if !applied {
        connection.execute_batch(INITIAL_SCHEMA)?;
        connection.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
            params![INITIAL_VERSION, Utc::now().to_rfc3339()],
        )?;
    }
    apply(connection, 2, LIBRARY_STATE_SCHEMA)?;
    apply(connection, 3, HISTORY_STATE_SCHEMA)?;
    apply(connection, 4, AI_ASSISTANT_SCHEMA)?;
    apply(connection, 5, KNOWLEDGE_PACK_SCHEMA)?;
    Ok(())
}

fn apply(connection: &Connection, version: i64, sql: &str) -> Result<()> {
    let applied: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
        [version],
        |row| row.get(0),
    )?;
    if !applied {
        connection.execute_batch(sql)?;
        connection.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
            params![version, Utc::now().to_rfc3339()],
        )?;
    }
    Ok(())
}
