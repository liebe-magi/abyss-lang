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
