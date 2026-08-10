ALTER TABLE clipboard_items ADD COLUMN saved_to_library INTEGER NOT NULL DEFAULT 0
    CHECK (saved_to_library IN (0, 1));

CREATE INDEX IF NOT EXISTS idx_clipboard_items_library
    ON clipboard_items(saved_to_library, updated_at DESC);
