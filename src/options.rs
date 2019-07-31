use super::JsonLdError;
use super::RemoteDocument;

// https://www.w3.org/TR/json-ld-api/#idl-def-JsonLdOptions
pub struct JsonLdOptions {
    pub base: Option<String>,
    pub compact_arrays: bool,
    pub document_loader: fn(String) -> Result<RemoteDocument, JsonLdError>,
    pub expand_context: Option<String>,
    pub processing_mode: Option<String>,
}
