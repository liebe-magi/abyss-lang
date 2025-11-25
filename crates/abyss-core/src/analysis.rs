use crate::ast::{AST, LineInfo, Type};
use crate::semantic::SymbolTable;
use crate::types::{ArtifactSchema, SpectrumSchema};

#[derive(Debug, Clone)]
pub enum AnalysisError {
    UndefinedVariable(String, Option<LineInfo>),
    TypeMismatch {
        expected: Type,
        found: Type,
        line_info: Option<LineInfo>,
    },
    NonExhaustiveMatch(String, Option<LineInfo>),
    DuplicateDefinition(String, Option<LineInfo>),
    InvalidOperation(String, Option<LineInfo>),
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<AnalysisError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, ast: &AST) -> Vec<AnalysisError> {
        self.visit(ast);
        self.errors.clone()
    }

    fn visit(&mut self, ast: &AST) {
        match ast {
            AST::Statement(stmt, _) => self.visit(stmt),
            AST::Block(stmts, _) => {
                self.symbol_table.push_scope();
                for stmt in stmts {
                    self.visit(stmt);
                }
                self.symbol_table.pop_scope();
            }
            AST::SpectrumDef {
                name,
                variants,
                line_info,
            } => {
                let mut variants_map = std::collections::HashMap::new();
                for variant in variants {
                    variants_map.insert(variant.name.clone(), variant.args.clone());
                }
                let schema = SpectrumSchema {
                    name: name.clone(),
                    variants: variants_map,
                    line_info: line_info.clone(),
                };
                if let Err(e) = self.symbol_table.define_spectrum(schema) {
                    self.errors
                        .push(AnalysisError::DuplicateDefinition(e, line_info.clone()));
                }
            }
            AST::ArtifactDef {
                name,
                fields: _,
                line_info,
            } => {
                // TODO: Handle fields properly
                let schema = ArtifactSchema {
                    name: name.clone(),
                    fields: Vec::new(), // Placeholder
                    methods: std::collections::HashMap::new(),
                    line_info: line_info.clone(),
                };
                if let Err(e) = self.symbol_table.define_artifact(schema) {
                    self.errors
                        .push(AnalysisError::DuplicateDefinition(e, line_info.clone()));
                }
            }
            AST::VarAssign {
                name,
                value,
                var_type,
                is_morph,
                line_info,
            } => {
                self.visit(value);
                if let Err(e) =
                    self.symbol_table
                        .define_variable(name.clone(), var_type.clone(), *is_morph)
                {
                    self.errors
                        .push(AnalysisError::DuplicateDefinition(e, line_info.clone()));
                }
            }
            AST::Oracle {
                is_match,
                conditionals,
                branches,
                line_info,
            } => {
                for cond in conditionals {
                    self.visit(&cond.expression);
                }
                for branch in branches {
                    self.visit(branch);
                }

                if *is_match {
                    let scrutinee_types: Vec<Type> = conditionals
                        .iter()
                        .map(|c| self.infer_type(&c.expression))
                        .collect();
                    self.check_exhaustiveness(&scrutinee_types, branches, line_info.clone());
                }
            }
            // ... other AST nodes ...
            _ => {}
        }
    }

    fn infer_type(&self, ast: &AST) -> Type {
        match ast {
            AST::Omen(_, _) => Type::Omen,
            AST::Arcana(_, _) => Type::Arcana,
            AST::Aether(_, _) => Type::Aether,
            AST::Rune(_, _) => Type::Rune,
            AST::Abyss(_) => Type::Abyss,
            AST::Var(name, _) => self
                .symbol_table
                .lookup(name)
                .map(|k| match k {
                    crate::semantic::SymbolKind::Variable { var_type, .. } => var_type.clone(),
                    _ => Type::Abyss,
                })
                .unwrap_or(Type::Abyss),
            AST::SpectrumInstantiation { spectrum, .. } => Type::Spectrum(spectrum.clone()),
            AST::ArtifactLiteral { type_name, .. } => Type::Artifact(type_name.clone()),
            _ => Type::Abyss,
        }
    }

    fn check_exhaustiveness(
        &mut self,
        types: &[Type],
        branches: &[AST],
        line_info: Option<LineInfo>,
    ) {
        if types.is_empty() {
            return;
        }

        // Check if there is a catch-all branch
        let has_catch_all_branch = branches.iter().any(|branch| {
            if let AST::OracleBranch { pattern, .. } = branch {
                pattern.iter().all(|p| self.is_catch_all(p))
            } else {
                false
            }
        });

        if has_catch_all_branch {
            return;
        }

        if types.len() > 1 {
            self.errors.push(AnalysisError::NonExhaustiveMatch(
                "Multi-value match requires a catch-all pattern".to_string(),
                line_info,
            ));
            return;
        }

        let type_ = &types[0];
        let patterns: Vec<&AST> = branches
            .iter()
            .filter_map(|b| {
                if let AST::OracleBranch { pattern, .. } = b {
                    pattern.first()
                } else {
                    None
                }
            })
            .collect();

        match type_ {
            Type::Omen => {
                let has_true = patterns.iter().any(|p| self.is_bool_literal(p, true));
                let has_false = patterns.iter().any(|p| self.is_bool_literal(p, false));
                if !has_true || !has_false {
                    self.errors.push(AnalysisError::NonExhaustiveMatch(
                        "Omen match must cover both boon and hex or use _".to_string(),
                        line_info,
                    ));
                }
            }
            Type::Arcana | Type::Aether | Type::Rune => {
                self.errors.push(AnalysisError::NonExhaustiveMatch(
                    format!("{:?} match must use _ or variable binding", type_),
                    line_info,
                ));
            }
            Type::Spectrum(name) => {
                if let Some(schema) = self.symbol_table.lookup_spectrum(name) {
                    let all_variants: std::collections::HashSet<_> =
                        schema.variants.keys().cloned().collect();
                    let covered_variants: std::collections::HashSet<_> = patterns
                        .iter()
                        .filter_map(|p| self.get_spectrum_variant(p))
                        .collect();

                    let missing: Vec<_> = all_variants.difference(&covered_variants).collect();
                    if !missing.is_empty() {
                        self.errors.push(AnalysisError::NonExhaustiveMatch(
                            format!("Missing variants: {:?}", missing),
                            line_info,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn is_catch_all(&self, ast: &AST) -> bool {
        matches!(ast, AST::OracleDontCareItem(_) | AST::PatternBinding { .. })
    }

    fn is_bool_literal(&self, ast: &AST, expected: bool) -> bool {
        if let AST::Omen(val, _) = ast {
            *val == expected
        } else {
            false
        }
    }

    fn get_spectrum_variant(&self, ast: &AST) -> Option<String> {
        if let AST::SpectrumPattern { variant, .. } = ast {
            Some(variant.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhaustiveness_omen_missing_hex() {
        let mut analyzer = SemanticAnalyzer::new();
        let branches = vec![AST::OracleBranch {
            pattern: vec![AST::Omen(true, None)],
            body: Box::new(AST::Abyss(None)),
            line_info: None,
        }];
        analyzer.check_exhaustiveness(&[Type::Omen], &branches, None);
        assert_eq!(analyzer.errors.len(), 1);
        match &analyzer.errors[0] {
            AnalysisError::NonExhaustiveMatch(msg, _) => {
                assert!(msg.contains("must cover both boon and hex"));
            }
            _ => panic!("Expected NonExhaustiveMatch error"),
        }
    }

    #[test]
    fn test_exhaustiveness_omen_covered() {
        let mut analyzer = SemanticAnalyzer::new();
        let branches = vec![
            AST::OracleBranch {
                pattern: vec![AST::Omen(true, None)],
                body: Box::new(AST::Abyss(None)),
                line_info: None,
            },
            AST::OracleBranch {
                pattern: vec![AST::Omen(false, None)],
                body: Box::new(AST::Abyss(None)),
                line_info: None,
            },
        ];
        analyzer.check_exhaustiveness(&[Type::Omen], &branches, None);
        assert!(analyzer.errors.is_empty());
    }

    #[test]
    fn test_exhaustiveness_catch_all() {
        let mut analyzer = SemanticAnalyzer::new();
        let branches = vec![AST::OracleBranch {
            pattern: vec![AST::OracleDontCareItem(None)],
            body: Box::new(AST::Abyss(None)),
            line_info: None,
        }];
        analyzer.check_exhaustiveness(&[Type::Omen], &branches, None);
        assert!(analyzer.errors.is_empty());
    }

    #[test]
    fn test_exhaustiveness_spectrum_missing_variant() {
        let mut analyzer = SemanticAnalyzer::new();
        let schema = SpectrumSchema {
            name: "Color".to_string(),
            variants: [("Red".to_string(), vec![]), ("Blue".to_string(), vec![])]
                .into_iter()
                .collect(),
            line_info: None,
        };
        analyzer.symbol_table.define_spectrum(schema).unwrap();

        let branches = vec![AST::OracleBranch {
            pattern: vec![AST::SpectrumPattern {
                spectrum: "Color".to_string(),
                variant: "Red".to_string(),
                args: vec![],
                line_info: None,
            }],
            body: Box::new(AST::Abyss(None)),
            line_info: None,
        }];
        analyzer.check_exhaustiveness(&[Type::Spectrum("Color".to_string())], &branches, None);
        assert_eq!(analyzer.errors.len(), 1);
        match &analyzer.errors[0] {
            AnalysisError::NonExhaustiveMatch(msg, _) => {
                assert!(msg.contains("Missing variants"));
                assert!(msg.contains("Blue"));
            }
            _ => panic!("Expected NonExhaustiveMatch error"),
        }
    }

    #[test]
    fn test_exhaustiveness_arcana_missing_catch_all() {
        let mut analyzer = SemanticAnalyzer::new();
        let branches = vec![AST::OracleBranch {
            pattern: vec![AST::Arcana(42, None)],
            body: Box::new(AST::Abyss(None)),
            line_info: None,
        }];
        analyzer.check_exhaustiveness(&[Type::Arcana], &branches, None);
        assert_eq!(analyzer.errors.len(), 1);
        match &analyzer.errors[0] {
            AnalysisError::NonExhaustiveMatch(msg, _) => {
                assert!(msg.contains("must use _ or variable binding"));
            }
            _ => panic!("Expected NonExhaustiveMatch error"),
        }
    }
}
