#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardPayload {
    pub format: String,
    pub windows_format: String,
    pub xml: String,
    pub raw_size: usize,
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("FileMaker Clipboard is not available: {0}")]
    NotAvailable(String),
    #[error("Clipboard data is invalid: {0}")]
    InvalidData(String),
    #[error("Windows Clipboard API failed: {0}")]
    WindowsApi(String),
    #[error("Unsupported platform")]
    UnsupportedPlatform,
}

pub trait ClipboardProvider: Send + Sync {
    fn get_filemaker_clipboard(&self) -> Result<ClipboardPayload, ClipboardError>;
    fn set_filemaker_clipboard(&self, format: &str, xml: &str) -> Result<(), ClipboardError>;
}

#[allow(dead_code)] // Used by the native read path introduced in Phase 3.
pub fn internal_format(windows_format: &str) -> String {
    windows_format
        .strip_prefix("Mac-")
        .unwrap_or(windows_format)
        .to_owned()
}

pub fn windows_format(internal_format: &str) -> String {
    if internal_format.starts_with("Mac-") {
        internal_format.to_owned()
    } else {
        format!("Mac-{internal_format}")
    }
}

pub fn encode_filemaker_payload(xml: &str) -> Result<Vec<u8>, ClipboardError> {
    let bytes = xml.as_bytes();
    let length = u32::try_from(bytes.len())
        .map_err(|_| ClipboardError::InvalidData("XML is larger than 4 GiB".to_owned()))?;
    let mut payload = Vec::with_capacity(bytes.len() + 4);
    payload.extend_from_slice(&length.to_le_bytes());
    payload.extend_from_slice(bytes);
    Ok(payload)
}

#[allow(dead_code)] // Used by the native read path introduced in Phase 3.
pub fn decode_filemaker_payload(payload: &[u8]) -> Result<String, ClipboardError> {
    if payload.len() < 4 {
        return Err(ClipboardError::InvalidData(
            "4-byte length header is missing".to_owned(),
        ));
    }
    let length = u32::from_le_bytes(payload[0..4].try_into().expect("four bytes")) as usize;
    if payload.len() < length + 4 {
        return Err(ClipboardError::InvalidData(format!(
            "header declares {length} bytes, but only {} are available",
            payload.len() - 4
        )));
    }
    std::str::from_utf8(&payload[4..length + 4])
        .map(str::to_owned)
        .map_err(|error| ClipboardError::InvalidData(format!("XML is not UTF-8: {error}")))
}

pub fn platform_provider() -> Box<dyn ClipboardProvider> {
    #[cfg(target_os = "windows")]
    {
        return Box::new(windows::WindowsClipboard::new());
    }
    #[cfg(target_os = "macos")]
    {
        return Box::new(macos::MacOsClipboard::new());
    }
    #[allow(unreachable_code)]
    Box::new(UnsupportedClipboard)
}

struct UnsupportedClipboard;

impl ClipboardProvider for UnsupportedClipboard {
    fn get_filemaker_clipboard(&self) -> Result<ClipboardPayload, ClipboardError> {
        Err(ClipboardError::UnsupportedPlatform)
    }

    fn set_filemaker_clipboard(&self, _format: &str, _xml: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_utf8_filemaker_payload() {
        let xml = "<Script name=\"地方競馬\" />";
        let payload = encode_filemaker_payload(xml).unwrap();
        assert_eq!(
            u32::from_le_bytes(payload[0..4].try_into().unwrap()) as usize,
            xml.len()
        );
        assert_eq!(decode_filemaker_payload(&payload).unwrap(), xml);
    }

    #[test]
    fn keeps_unknown_formats_open_ended() {
        assert_eq!(internal_format("Mac-XMZZ"), "XMZZ");
        assert_eq!(windows_format("XMZZ"), "Mac-XMZZ");
    }
}
