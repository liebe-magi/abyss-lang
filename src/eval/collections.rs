use crate::ast::AST;
use crate::ast::LineInfo;
use crate::env::Value;

use super::result::{EvalError, EvalResult};

pub(crate) fn expect_arcana_index(
    index: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<usize, EvalError> {
    if let EvalResult::Arcana(value) = index {
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
    if let EvalResult::Rune(value) = index {
        Ok(value.clone())
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

pub(crate) fn resolve_nested_value_mut<'a>(
    value: &'a mut Value,
    index: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<&'a mut Value, EvalError> {
    match value {
        Value::Scroll(items) => {
            let idx = expect_arcana_index(index, line_info)?;
            items.get_mut(idx).ok_or_else(|| {
                EvalError::InvalidOperation(
                    format!("Index {} is out of bounds for scroll", idx),
                    line_info.clone(),
                )
            })
        }
        Value::Lexicon(entries) => {
            let key = expect_rune_key(index, line_info)?;
            entries.get_mut(key.as_str()).ok_or_else(|| {
                EvalError::InvalidOperation(
                    format!("Lexicon key '{}' does not exist", key),
                    line_info.clone(),
                )
            })
        }
        _ => Err(EvalError::InvalidOperation(
            "Cannot index into non-collection value".to_string(),
            line_info.clone(),
        )),
    }
}
