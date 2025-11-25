use crate::ast::{AST, LineInfo, Type};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SpectrumSchema {
    pub name: String,
    pub variants: HashMap<String, Vec<Type>>,
    pub line_info: Option<LineInfo>,
}

#[derive(Debug, Clone)]
pub struct ArtifactSchema {
    pub name: String,
    pub fields: Vec<ArtifactFieldSchema>,
    pub methods: HashMap<String, ArtifactMethod>,
    pub line_info: Option<LineInfo>,
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
pub struct ArtifactMethod {
    pub function: EngravedFunction,
    pub requires_mutable_receiver: bool,
}

#[derive(Debug, Clone)]
pub struct EngravedFunction {
    pub name: String,
    pub params: Vec<AST>,
    pub return_type: Type,
    pub body: Box<AST>,
    pub line_info: Option<LineInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
}
