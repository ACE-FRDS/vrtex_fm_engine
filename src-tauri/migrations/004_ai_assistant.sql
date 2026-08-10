CREATE TABLE IF NOT EXISTS ai_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    mode TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    dry_run INTEGER NOT NULL DEFAULT 1 CHECK (dry_run IN (0, 1)),
    risk_level TEXT NOT NULL DEFAULT 'LOW',
    status TEXT NOT NULL DEFAULT 'active',
    generated_xml TEXT NOT NULL DEFAULT '',
    validation_status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_sessions_project_updated
    ON ai_sessions(project_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS ai_messages (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES ai_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_messages_session_created
    ON ai_messages(session_id, created_at ASC);

CREATE TABLE IF NOT EXISTS rag_documents (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source_type TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rag_documents_source
    ON rag_documents(source_type, updated_at DESC);

INSERT OR IGNORE INTO rag_documents
    (id, title, content, source_type, tags, created_at, updated_at)
VALUES
    ('fm-clipboard-root', 'FileMaker Clipboard XML root',
     'FileMaker Clipboard XML uses fmxmlsnippet type="FMObjectList". XMSC contains direct Script children. XMSS contains direct Step children. Script steps form a flat sequence.',
     'filemaker-spec', 'FileMaker XML XMSC XMSS fmxmlsnippet', datetime('now'), datetime('now')),
    ('fm-xmsc-script', 'XMSC Script requirements',
     'A Script element requires includeInMenu, SiriShortcutVisible, runFullAccess, id, and name attributes. Step elements require enable, id, and name attributes. Boolean values use True or False.',
     'filemaker-spec', 'XMSC Script attributes validation', datetime('now'), datetime('now')),
    ('fm-safe-change', 'Safe FileMaker change policy',
     'Inspect the current structure before changing it. Preserve existing objects. Validate generated XML and show a diff before sending. Deletion, replacement, security, account, privilege, and bulk changes require explicit user approval.',
     'vertex-rule', 'safety approval destructive change', datetime('now'), datetime('now')),
    ('vertex-layering', 'Vertex Project architecture',
     'Vertex Project keeps external source specifications in the SRC layer, transforms them into internal models in the CORE layer, and keeps the UI layer independent from the data source.',
     'project-rule', 'Vertex SRC CORE UI architecture', datetime('now'), datetime('now')),
    ('fm-clipboard-formats', 'Representative FileMaker formats',
     'XMSC represents scripts, XMSS script steps, XMTB tables, XMFD fields, and XML2 layout objects. Preserve the original format and validate against the expected object wrapper.',
     'filemaker-spec', 'XMSC XMSS XMTB XMFD XML2', datetime('now'), datetime('now')),
    ('vertex-ai-policy', 'Vertex AI execution policy',
     'AI performs requirement analysis, design, review, and XML proposal generation. Vertex FM Engine performs validation, diff, approval, clipboard control, history, and FileMaker delivery. Dry Run is enabled by default.',
     'vertex-rule', 'AI orchestrator dry run validation', datetime('now'), datetime('now'));
