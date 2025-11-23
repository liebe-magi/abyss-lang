use crate::ast::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub var_type: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolInfo>>,
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

    pub fn define(&mut self, name: String, var_type: Type, is_mutable: bool) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(format!("Symbol '{}' already defined in this scope", name));
            }
            scope.insert(
                name,
                SymbolInfo {
                    var_type,
                    is_mutable,
                },
            );
            Ok(())
        } else {
            Err("No scope available".to_string())
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                return Some(info);
            }
        }
        None
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
    fn test_define_and_lookup() {
        let mut table = SymbolTable::new();
        table
            .define("x".to_string(), Type::Arcana, false)
            .expect("failed to define symbol");

        let info = table.lookup("x").expect("symbol not found");
        assert_eq!(info.var_type, Type::Arcana);
        assert!(!info.is_mutable);

        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_define_duplicate_in_same_scope() {
        let mut table = SymbolTable::new();
        table
            .define("x".to_string(), Type::Arcana, false)
            .expect("failed to define symbol");

        let result = table.define("x".to_string(), Type::Aether, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_shadowing() {
        let mut table = SymbolTable::new();
        table
            .define("x".to_string(), Type::Arcana, false)
            .expect("failed to define symbol");

        table.push_scope();
        table
            .define("x".to_string(), Type::Aether, true)
            .expect("failed to shadow symbol");

        let info = table.lookup("x").expect("symbol not found");
        assert_eq!(info.var_type, Type::Aether);
        assert!(info.is_mutable);

        table.pop_scope();
        let info = table.lookup("x").expect("symbol not found");
        assert_eq!(info.var_type, Type::Arcana);
        assert!(!info.is_mutable);
    }

    #[test]
    fn test_lookup_mut() {
        let mut table = SymbolTable::new();
        table
            .define("x".to_string(), Type::Arcana, true)
            .expect("failed to define symbol");

        if let Some(info) = table.lookup_mut("x") {
            info.is_mutable = false;
        }

        let info = table.lookup("x").expect("symbol not found");
        assert!(!info.is_mutable);
    }
}
