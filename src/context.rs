use std::collections::{BTreeMap, HashMap};

use serde_json::{ Map, Value };
use url::Url;

use super::{JsonLdError, JsonLdOptions};

#[derive(Clone)]
pub struct Context {
    pub base: Option<Url>,
    pub vocab: Option<String>,
    pub terms: BTreeMap<String, Value>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            base: None,
            vocab: None,
            terms: BTreeMap::new(),
        }
    }

    pub fn from_base(base: Option<Url>) -> Context {
        Context {
            base,
            vocab: None,
            terms: BTreeMap::new(),
        }
    }

    pub fn from_options(options: JsonLdOptions) -> Result<Context, JsonLdError> {
        let mut base = None;
        if let Some(b) = options.base {
            base = Url::parse(&b)
                .map(Some)
                .or(Err(JsonLdError::InvalidBaseIRI))?
        }

        Ok(Context {
            base,
            vocab: None,
            terms: BTreeMap::new(),
        })
    }

    // https://w3c.github.io/json-ld-api/#context-processing-algorithm
    pub fn process(
        mut self,
        local_context: Value,
        remote_contexts: Vec<String>,
    ) -> Result<Context, JsonLdError> {
        // 4
        let local_context = match local_context {
            Value::Array(a) => a,
            _ => vec![local_context],
        };

        // 5
        for context in local_context {
            match context {
                // 5.1
                Value::Null => {
                    self = Context::from_base(self.base.clone());
                }

                // 5.2
                Value::String(s) => {
                    // TODO: dereference context
                    unimplemented!();
                }

                // 5.4
                Value::Object(map) => {
                    // 5.7
                    let base = map.get("@base");
                    if base.is_some() && remote_contexts.is_empty() {
                        // 5.7.1
                        let value = base.unwrap();

                        match value {
                            // 5.7.2
                            Value::Null => self.base = None,

                            Value::String(s) => {
                                // 5.7.3
                                if is_absolute_iri(&s) {
                                    self.base =
                                        Some(Url::parse(&s).or(Err(JsonLdError::InvalidBaseIRI))?);

                                // 5.7.4
                                } else if let Some(base) = &self.base {
                                    if is_relative_iri(&s) {
                                        self.base = Some(
                                            base.join(&s).or(Err(JsonLdError::InvalidBaseIRI))?,
                                        );
                                    }
                                }
                            }
                            _ => return Err(JsonLdError::InvalidBaseIRI),
                        }
                    }

                    // 5.8
                    if let Some(value) = map.get("@vocab") {
                        // 5.8.1
                        match value {
                            // 5.8.2
                            Value::Null => self.vocab = None,

                            // 5.8.3
                            Value::String(s) => {
                                if is_absolute_iri(&s) {
                                    self.vocab = Some(s.clone());
                                } else {
                                    return Err(JsonLdError::InvalidVocabMapping);
                                }
                            }

                            _ => return Err(JsonLdError::InvalidVocabMapping),
                        }
                    }

                    // 5.9
                    // 5.9.1
                    if let Some(value) = map.get("@language") {
                        match value {
                            // 5.9.2
                            Value::Null => self.vocab = None,

                            // 5.9.3
                            Value::String(s) => self.vocab = Some(s.to_lowercase()),

                            _ => return Err(JsonLdError::InvalidDefaultLanguage),
                        }
                    }

                    // 5.11
                    let defined: HashMap<String, bool> = HashMap::new();

                    // 5.12
                    for term in map.keys() {
                        if term == "@base" || term == "@vocab" || term == "@language" {
                            continue;
                        };

                        self.create_term_definition(&map, term, &defined);
                    }
                }

                // 5.3
                _ => return Err(JsonLdError::InvalidLocalContext),
            }
        }

        Ok(self)
    }

    fn create_term_definition(
        &mut self,
        local_context: &Map<String, Value>,
        term: &String,
        defined: &HashMap<String, bool>,
    ) {
        
    }
}

fn expand_iri() {}

fn is_absolute_iri(iri: &String) -> bool {
    unimplemented!();
}

fn is_relative_iri(iri: &String) -> bool {
    unimplemented!();
}
