use serde_json::Value;

// https://www.w3.org/TR/json-ld-api/#idl-def-RemoteDocument
pub struct RemoteDocument {
    pub document: Value,
    pub document_url: String,
    pub context_url: String,
}
