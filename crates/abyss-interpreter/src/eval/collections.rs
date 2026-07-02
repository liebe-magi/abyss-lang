use crate::env::Value;
use abyss_core::ast::Expr;
use abyss_core::ast::Span;

use super::result::{EvalError, EvalResult};

pub(crate) fn expect_arcana_index(
    index: &EvalResult,
    line_info: &Option<Span>,
) -> Result<usize, EvalError> {
    if let EvalResult::Data(Value::Arcana(value)) = index {
        if *value < 0 {
            return Err(EvalError::InvalidOperation(
                "Scroll index cannot be negative".to_string(),
                *line_info,
            ));
        }
        Ok(*value as usize)
    } else {
        Err(EvalError::TypeError(
            "Scroll index must be arcana".to_string(),
            *line_info,
        ))
    }
}

pub(crate) fn expect_rune_key(
    index: &EvalResult,
    line_info: &Option<Span>,
) -> Result<String, EvalError> {
    if let EvalResult::Data(Value::Rune(value)) = index {
        Ok(value.as_ref().clone())
    } else {
        Err(EvalError::TypeError(
            "Lexicon key must be rune".to_string(),
            *line_info,
        ))
    }
}

pub(crate) fn collect_index_chain(target: &Expr) -> Option<(String, Vec<&Expr>)> {
    let mut indices = Vec::new();
    let mut current = target;

    loop {
        match current {
            Expr::Var(name, _) => {
                indices.reverse();
                return Some((name.clone(), indices));
            }
            Expr::IndexAccess { target, index, .. } => {
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

    fn line() -> Option<Span> {
        Some(Span::new(1, 1))
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
        let ast = Expr::IndexAccess {
            target: Box::new(Expr::IndexAccess {
                target: Box::new(Expr::Var("sigils".into(), None)),
                index: Box::new(Expr::Arcana(1, None)),
                span: None,
            }),
            index: Box::new(Expr::Rune("beta".into(), None)),
            span: None,
        };

        let (name, indices) = collect_index_chain(&ast).expect("expected chain");
        assert_eq!(name, "sigils");
        assert_eq!(indices.len(), 2);
        match indices[0] {
            Expr::Arcana(value, _) => assert_eq!(*value, 1),
            other => panic!("Unexpected first index: {:?}", other),
        }
        match indices[1] {
            Expr::Rune(value, _) => assert_eq!(value, "beta"),
            other => panic!("Unexpected second index: {:?}", other),
        }
    }

    #[test]
    fn collect_index_chain_handles_non_chain_inputs() {
        let var = Expr::Var("sigils".into(), None);
        let (name, indices) = collect_index_chain(&var).expect("var should be recognized");
        assert_eq!(name, "sigils");
        assert!(indices.is_empty());

        let field_access = Expr::FieldAccess {
            target: Box::new(Expr::Var("sigils".into(), None)),
            field: "core".into(),
            span: None,
        };
        assert!(collect_index_chain(&field_access).is_none());
    }
}
