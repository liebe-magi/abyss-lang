use crate::ast::AST;
use crate::ast::LineInfo;
use crate::env::Value;

use super::result::{EvalError, EvalResult};

pub(crate) fn expect_arcana_index(
    index: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<usize, EvalError> {
    if let EvalResult::Data(Value::Arcana(value)) = index {
        if *value < 0 {
            return Err(EvalError::InvalidOperation(
                "Scroll index cannot be negative".to_string(),
                line_info.clone(),
            ));
        }
        Ok(*value as usize)
    } else {
        Err(EvalError::TypeError(
            "Scroll index must be arcana".to_string(),
            line_info.clone(),
        ))
    }
}

pub(crate) fn expect_rune_key(
    index: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<String, EvalError> {
    if let EvalResult::Data(Value::Rune(value)) = index {
        Ok(value.as_ref().clone())
    } else {
        Err(EvalError::TypeError(
            "Lexicon key must be rune".to_string(),
            line_info.clone(),
        ))
    }
}

pub(crate) fn collect_index_chain(target: &AST) -> Option<(String, Vec<&AST>)> {
    let mut indices = Vec::new();
    let mut current = target;

    loop {
        match current {
            AST::Var(name, _) => {
                indices.reverse();
                return Some((name.clone(), indices));
            }
            AST::IndexAccess { target, index, .. } => {
                indices.push(index.as_ref());
                current = target.as_ref();
            }
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn line() -> Option<LineInfo> {
        Some(LineInfo::new(1, 1))
    }

    #[test]
    fn expect_arcana_index_accepts_non_negative_values() {
        let index = EvalResult::Data(Value::Arcana(4));
        assert_eq!(expect_arcana_index(&index, &None).unwrap(), 4);

        let zero_index = EvalResult::Data(Value::Arcana(0));
        assert_eq!(expect_arcana_index(&zero_index, &None).unwrap(), 0);
    }

    #[test]
    fn expect_arcana_index_rejects_invalid_inputs() {
        let negative_index = EvalResult::Data(Value::Arcana(-1));
        let err = expect_arcana_index(&negative_index, &line()).unwrap_err();
        match err {
            EvalError::InvalidOperation(message, info) => {
                assert!(message.contains("cannot be negative"));
                assert!(info.is_some());
            }
            other => panic!("Unexpected error variant: {:?}", other),
        }

        let rune_index = EvalResult::Data(Value::Rune(Rc::new("key".into())));
        let err = expect_arcana_index(&rune_index, &None).unwrap_err();
        match err {
            EvalError::TypeError(message, _) => {
                assert!(message.contains("must be arcana"));
            }
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn expect_rune_key_validates_rune_inputs() {
        let rune = EvalResult::Data(Value::Rune(Rc::new("glyph".into())));
        assert_eq!(expect_rune_key(&rune, &None).unwrap(), "glyph");

        let arcana = EvalResult::Data(Value::Arcana(2));
        let err = expect_rune_key(&arcana, &line()).unwrap_err();
        match err {
            EvalError::TypeError(message, info) => {
                assert!(message.contains("Lexicon key"));
                assert!(info.is_some());
            }
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn collect_index_chain_extracts_indices_in_order() {
        let ast = AST::IndexAccess {
            target: Box::new(AST::IndexAccess {
                target: Box::new(AST::Var("sigils".into(), None)),
                index: Box::new(AST::Arcana(1, None)),
                line_info: None,
            }),
            index: Box::new(AST::Rune("beta".into(), None)),
            line_info: None,
        };

        let (name, indices) = collect_index_chain(&ast).expect("expected chain");
        assert_eq!(name, "sigils");
        assert_eq!(indices.len(), 2);
        match indices[0] {
            AST::Arcana(value, _) => assert_eq!(*value, 1),
            other => panic!("Unexpected first index: {:?}", other),
        }
        match indices[1] {
            AST::Rune(value, _) => assert_eq!(value, "beta"),
            other => panic!("Unexpected second index: {:?}", other),
        }
    }

    #[test]
    fn collect_index_chain_handles_non_chain_inputs() {
        let var = AST::Var("sigils".into(), None);
        let (name, indices) = collect_index_chain(&var).expect("var should be recognized");
        assert_eq!(name, "sigils");
        assert!(indices.is_empty());

        let field_access = AST::FieldAccess {
            target: Box::new(AST::Var("sigils".into(), None)),
            field: "core".into(),
            line_info: None,
        };
        assert!(collect_index_chain(&field_access).is_none());
    }
}
