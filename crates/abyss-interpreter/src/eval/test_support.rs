//! Shared constructors for the `eval` unit tests.
//!
//! Only compiled for `cfg(test)`; keeps the per-module `#[cfg(test)]`
//! blocks free of copy-pasted `Value` / artifact scaffolding.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use abyss_core::ast::Type;

use crate::env::{
    ArtifactFieldSchema, ArtifactHandle, ArtifactSchema, ArtifactValue, RuntimeEnv, Value,
};

pub(crate) fn rune(text: &str) -> Value {
    Value::Rune(Rc::new(text.to_string()))
}

pub(crate) fn scroll(values: Vec<Value>) -> Value {
    Value::Scroll(Rc::new(RefCell::new(values)))
}

pub(crate) fn lexicon(entries: Vec<(&str, Value)>) -> Value {
    let mut map = HashMap::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    Value::Lexicon(Rc::new(RefCell::new(map)))
}

pub(crate) fn artifact_handle(name: &str, fields: Vec<(&str, Value)>) -> ArtifactHandle {
    let mut map = HashMap::new();
    let mut order = Vec::new();
    for (field, value) in fields {
        let key = field.to_string();
        order.push(key.clone());
        map.insert(key, value);
    }
    Rc::new(RefCell::new(ArtifactValue {
        type_name: name.to_string(),
        fields: map,
        field_order: order,
    }))
}

pub(crate) fn register_artifact(env: &mut RuntimeEnv, name: &str, fields: Vec<(&str, Type)>) {
    let schema = ArtifactSchema {
        name: name.to_string(),
        fields: fields
            .into_iter()
            .map(|(field, field_type)| ArtifactFieldSchema {
                name: field.to_string(),
                field_type,
            })
            .collect(),
        methods: HashMap::new(),
        line_info: None,
    };
    env.define_artifact(schema).expect("schema registration");
}
