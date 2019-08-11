use std::collections::{BTreeMap, HashMap};

use serde_json::{Map, Value};
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
                    let mut defined: HashMap<String, bool> = HashMap::new();

                    // 5.12
                    for term in map.keys() {
                        if term == "@base" || term == "@vocab" || term == "@language" {
                            continue;
                        };

                        self.create_term_definition(&map, term, &mut defined)?;
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
        term: &str,
        defined: &mut HashMap<String, bool>,
    ) -> Result<(), JsonLdError> {
        // 1
        if let Some(v) = defined.get(term) {
            match v {
                true => return Ok(()),
                false => return Err(JsonLdError::CyclicIRIMapping),
            }
        }

        // 2
        defined.insert(term.to_owned(), false);

        // 3
        let mut value = local_context.get(term).unwrap();

        // 5
        if is_keyword(term) {
            return Err(JsonLdError::KeywordRedefinition);
        }

        // 7
        self.terms.remove(term);

        // 9
        let mut tmp_val; // TODO: find out how to give reference to variable with limited scope
        if let Value::String(s) = value {
            let mut map: Map<String, Value> = Map::new();
            map.insert("@id".to_string(), Value::String(s.clone()));

            tmp_val = Value::Object(map);
            value = &tmp_val;
        }

        // 10
        match value {
            Value::Object(value) => {
                // 11
                let mut definition = Map::new();

                // 13
                if let Some(t) = value.get("@type") {
                    // 13.1
                    match t {
                        Value::String(t) => {
                            // 13.2
                            if let Some(t) =
                                self.expand_iri(t, false, true, local_context, defined)?
                            {
                                if t == "@id" || t == "@vocab" || is_absolute_iri(&t) {
                                    // 13.3
                                    definition.insert("@type".to_string(), Value::String(t));
                                } else {
                                    return Err(JsonLdError::InvalidTypeMapping);
                                }
                            } else {
                                return Err(JsonLdError::InvalidTypeMapping);
                            }
                        }
                        _ => return Err(JsonLdError::InvalidTypeMapping),
                    }
                }

                // 14
                if let Some(reverse) = value.get("@reverse") {
                    // 14.1
                    if value.contains_key("@id") || value.contains_key("@nest") {
                        return Err(JsonLdError::InvalidReverseProperty);
                    }

                    // 14.2
                    if let Value::String(reverse) = reverse {
                        // 14.3
                        if let Some(id) =
                            self.expand_iri(reverse, false, true, local_context, defined)?
                        {
                            if !is_absolute_iri(&id) {
                                return Err(JsonLdError::InvalidIRIMapping);
                            }

                            definition.insert("@id".to_string(), Value::String(id));

                            // 14.4
                            if let Some(container) = value.get("@container") {
                                let container = match container {
                                    Value::Null => Value::Null,
                                    Value::String(s) => {
                                        if s == "@set" || s == "@index" {
                                            Value::String(s.to_string())
                                        } else {
                                            return Err(JsonLdError::InvalidReverseProperty);
                                        }
                                    }
                                    _ => return Err(JsonLdError::InvalidReverseProperty),
                                };

                                definition.insert("@container".to_string(), container);
                            }

                            // 14.5
                            definition.insert("@reverse".to_string(), Value::Bool(true));

                            // 14.6
                            self.terms
                                .insert(term.to_string(), Value::Object(definition));
                            defined.insert(term.to_string(), true);
                            return Ok(());
                        } else {
                            return Err(JsonLdError::InvalidIRIMapping);
                        }
                    } else {
                        return Err(JsonLdError::InvalidIRIMapping);
                    }
                }

                // 15
                definition.insert("@reverse".to_string(), Value::Bool(false));

                // 16
                if let Some(id) = value.get("id") {
                    match id {
                        Value::String(s) => {
                            if s != term {
                                // 16.3
                                let id = self.expand_iri(s, false, true, local_context, defined)?;
                                if let Some(id) = id {
                                    if !is_absolute_iri(&id) && !is_keyword(&id) {
                                        return Err(JsonLdError::InvalidIRIMapping);
                                    }

                                    definition.insert("@id".to_string(), Value::String(id));
                                } else {
                                    return Err(JsonLdError::InvalidIRIMapping);
                                }
                            }
                        },
                        // 16.2
                        _ => return Err(JsonLdError::InvalidIRIMapping)
                    }
                }

            }
            _ => return Err(JsonLdError::InvalidTermDefinition),
        }

        Ok(())
    }

    fn expand_iri(
        &mut self,
        value: &str,
        relative: bool,
        vocab: bool,
        local_context: &Map<String, Value>,
        defined: &mut HashMap<String, bool>,
    ) -> Result<Option<String>, JsonLdError> {
        // 1
        if is_keyword(value) {
            return Ok(Some(value.to_string()));
        }

        // 2
        if local_context.contains_key(value)
            && defined.contains_key(value)
            && !*defined.get(value).unwrap()
        {
            self.create_term_definition(local_context, value, defined)?
        }

        // 4
        if vocab && self.terms.contains_key(value) {
            return Ok(match self.terms.get(value).unwrap() {
                Value::Null => None,
                Value::Object(m) => Some(m.get("@id").unwrap().to_string()),
                _ => panic!("should not happen"),
            });
        }

        // 5
        if let Some(i) = value.find(":") {
            // 5.1
            let (prefix, suffix) = value.split_at(i);

            // 5.2
            if prefix == "_" || suffix.starts_with("//") {
                return Ok(Some(value.to_string()));
            }

            // 5.3
            if local_context.contains_key(prefix)
                && (!defined.contains_key(prefix) || !defined.get(prefix).unwrap())
            {
                self.create_term_definition(local_context, prefix, defined)?
            }

            // 5.4
            if let Some(v) = self.terms.get(prefix) {
                let iri_mapping = match v {
                    Value::Object(m) => m.get("@id").unwrap().to_string(),
                    _ => panic!("should not happen"),
                };

                return Ok(Some(iri_mapping + suffix));
            }

            // 5.5
            return Ok(Some(value.to_string()));
        }

        // 6
        if vocab && self.vocab.is_some() {
            return Ok(Some(self.vocab.as_ref().unwrap().to_string() + value));
        } else if relative && self.base.is_some() {
            return Ok(Some(
                self.base.as_ref().unwrap().join(value).unwrap().to_string(),
            ));
        }

        // 7
        Ok(Some(value.to_string()))
    }
}

fn is_keyword(val: &str) -> bool {
    return match val {
        "@container" | "@context" | "@graph" | "@id" | "@index" | "@language" | "@list"
        | "@reverse" | "@set" | "@type" | "@value" | "@vocab" => true,
        _ => false,
    };
}

fn is_absolute_iri(iri: &str) -> bool {
    unimplemented!();
}

fn is_relative_iri(iri: &str) -> bool {
    unimplemented!();
}
