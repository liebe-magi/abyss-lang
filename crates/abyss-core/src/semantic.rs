use crate::ast::Type;
use crate::types::{ArtifactSchema, FunctionSignature, SpectrumSchema};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable { var_type: Type, is_mutable: bool },
    Spectrum(SpectrumSchema),
    Artifact(ArtifactSchema),
    Function(FunctionSignature),
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolKind>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define_variable(
        &mut self,
        name: String,
        var_type: Type,
        is_mutable: bool,
    ) -> Result<(), String> {
        self.define_symbol(
            name,
            SymbolKind::Variable {
                var_type,
                is_mutable,
            },
        )
    }

    pub fn define_spectrum(&mut self, schema: SpectrumSchema) -> Result<(), String> {
        self.define_symbol(schema.name.clone(), SymbolKind::Spectrum(schema))
    }

    pub fn define_artifact(&mut self, schema: ArtifactSchema) -> Result<(), String> {
        self.define_symbol(schema.name.clone(), SymbolKind::Artifact(schema))
    }

    pub fn define_function(&mut self, signature: FunctionSignature) -> Result<(), String> {
        self.define_symbol(signature.name.clone(), SymbolKind::Function(signature))
    }

    fn define_symbol(&mut self, name: String, kind: SymbolKind) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(format!("Symbol '{}' already defined in this scope", name));
            }
            scope.insert(name, kind);
            Ok(())
        } else {
            Err("No scope available".to_string())
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolKind> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn lookup_spectrum(&self, name: &str) -> Option<&SpectrumSchema> {
        match self.lookup(name) {
            Some(SymbolKind::Spectrum(schema)) => Some(schema),
            _ => None,
        }
    }

    pub fn lookup_artifact(&self, name: &str) -> Option<&ArtifactSchema> {
        match self.lookup(name) {
            Some(SymbolKind::Artifact(schema)) => Some(schema),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_scope_management() {
        let mut table = SymbolTable::new();
        assert_eq!(table.scopes.len(), 1);

        table.push_scope();
        assert_eq!(table.scopes.len(), 2);

        table.pop_scope();
        assert_eq!(table.scopes.len(), 1);

        // Should not pop the last scope
        table.pop_scope();
        assert_eq!(table.scopes.len(), 1);
    }

    #[test]
    fn test_define_and_lookup_variable() {
        let mut table = SymbolTable::new();
        table
            .define_variable("x".to_string(), Type::Arcana, false)
            .expect("failed to define symbol");

        let kind = table.lookup("x").expect("symbol not found");
        match kind {
            SymbolKind::Variable {
                var_type,
                is_mutable,
            } => {
                assert_eq!(*var_type, Type::Arcana);
                assert!(!*is_mutable);
            }
            _ => panic!("expected variable"),
        }

        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_define_duplicate_in_same_scope() {
        let mut table = SymbolTable::new();
        table
            .define_variable("x".to_string(), Type::Arcana, false)
            .expect("failed to define symbol");

        let result = table.define_variable("x".to_string(), Type::Aether, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_shadowing() {
        let mut table = SymbolTable::new();
        table
            .define_variable("x".to_string(), Type::Arcana, false)
            .expect("failed to define symbol");

        table.push_scope();
        table
            .define_variable("x".to_string(), Type::Aether, true)
            .expect("failed to shadow symbol");

        let kind = table.lookup("x").expect("symbol not found");
        match kind {
            SymbolKind::Variable {
                var_type,
                is_mutable,
            } => {
                assert_eq!(*var_type, Type::Aether);
                assert!(*is_mutable);
            }
            _ => panic!("expected variable"),
        }

        table.pop_scope();
        let kind = table.lookup("x").expect("symbol not found");
        match kind {
            SymbolKind::Variable {
                var_type,
                is_mutable,
            } => {
                assert_eq!(*var_type, Type::Arcana);
                assert!(!*is_mutable);
            }
            _ => panic!("expected variable"),
        }
    }

    #[test]
    fn test_define_and_lookup_spectrum() {
        let mut table = SymbolTable::new();
        let schema = SpectrumSchema {
            name: "Color".to_string(),
            variants: HashMap::new(),
            line_info: None,
        };
        table
            .define_spectrum(schema.clone())
            .expect("failed to define spectrum");

        let lookup = table.lookup_spectrum("Color").expect("spectrum not found");
        assert_eq!(lookup.name, "Color");
    }
}
