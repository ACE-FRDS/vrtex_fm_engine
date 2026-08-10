ALTER TABLE clipboard_items ADD COLUMN in_history INTEGER NOT NULL DEFAULT 1
    CHECK (in_history IN (0, 1));

CREATE INDEX IF NOT EXISTS idx_clipboard_items_history
    ON clipboard_items(in_history, last_used_at DESC);
