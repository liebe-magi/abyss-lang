//! Artifact (user-defined struct) metadata and instances.
//!
//! An artifact definition registers an [`ArtifactSchema`] (field names,
//! types, and methods) in the [`RuntimeEnv`](crate::env::RuntimeEnv);
//! instantiation produces an [`ArtifactValue`] shared behind an
//! [`ArtifactHandle`] so multiple bindings alias the same instance.
//! Re-exported from [`crate::env`] for backwards compatibility with the
//! pre-split module layout.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use abyss_core::ast::{Span, Type};

use crate::env::EngravedFunction;
use crate::value::Value;

/// A method engraved on an artifact schema. `requires_mutable_receiver`
/// records whether the receiver was declared `morph self`, so calls on
/// immutable bindings can be rejected.
#[derive(Debug, Clone)]
pub struct ArtifactMethod {
    pub function: EngravedFunction,
    pub requires_mutable_receiver: bool,
}

#[derive(Debug, Clone)]
pub struct ArtifactSchema {
    pub name: String,
    pub fields: Vec<ArtifactFieldSchema>,
    pub methods: HashMap<String, ArtifactMethod>,
    pub line_info: Option<Span>,
}

impl ArtifactSchema {
    pub fn field(&self, name: &str) -> Option<&ArtifactFieldSchema> {
        self.fields.iter().find(|field| field.name == name)
    }

    pub fn field_names(&self) -> Vec<String> {
        self.fields.iter().map(|field| field.name.clone()).collect()
    }

    pub fn method(&self, name: &str) -> Option<&ArtifactMethod> {
        self.methods.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactFieldSchema {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone)]
pub struct ArtifactValue {
    pub type_name: String,
    pub fields: HashMap<String, Value>,
    pub field_order: Vec<String>,
}

pub type ArtifactHandle = Rc<RefCell<ArtifactValue>>;
