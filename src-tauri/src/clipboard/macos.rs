use super::{ClipboardError, ClipboardPayload, ClipboardProvider};

pub struct MacOsClipboard;

impl MacOsClipboard {
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardProvider for MacOsClipboard {
    fn get_filemaker_clipboard(&self) -> Result<ClipboardPayload, ClipboardError> {
        Err(ClipboardError::NotAvailable(
            "macOS support is intentionally stubbed for the Windows-first release".to_owned(),
        ))
    }

    fn set_filemaker_clipboard(&self, _format: &str, _xml: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::NotAvailable(
            "macOS support is intentionally stubbed for the Windows-first release".to_owned(),
        ))
    }
}
