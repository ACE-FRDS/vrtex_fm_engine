use roxmltree::Document;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XmlError {
    #[error("XML parse error: {0}")]
    Parse(#[from] roxmltree::Error),
    #[error("fmxmlsnippet root element was not found")]
    MissingRoot,
}

pub fn parse_filemaker_xml(xml: &str) -> Result<Document<'_>, XmlError> {
    let document = Document::parse(xml)?;
    if document.root_element().tag_name().name() != "fmxmlsnippet" {
        return Err(XmlError::MissingRoot);
    }
    Ok(document)
}
