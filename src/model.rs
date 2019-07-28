pub struct JsonLdOptions {
    pub base: Option<String>,
    pub compact_arrays: bool,
    pub compact_to_relative: bool,
    pub expand_context: Option<String>,
    pub extract_all_scripts: bool,
    pub frame_expansion: bool,
    pub ordered: bool,
    pub processing_mode: Option<String>,
    pub produce_generalized_rdf: bool,
    pub use_native_types: bool,
    pub use_rdf_types: bool,
}
