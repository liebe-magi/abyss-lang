use crate::ast::{LineInfo, Type};
use crate::env::Value;
use std::collections::HashMap;

use super::result::{EvalError, EvalResult};

pub(crate) fn value_to_eval_result(value: &Value) -> EvalResult {
    match value {
        Value::Omen(b) => EvalResult::Omen(*b),
        Value::Arcana(n) => EvalResult::Arcana(*n),
        Value::Aether(n) => EvalResult::Aether(*n),
        Value::Rune(s) => EvalResult::Rune(s.clone()),
        Value::Abyss => EvalResult::Abyss,
        Value::Scroll(items) => {
            EvalResult::Scroll(items.iter().map(value_to_eval_result).collect())
        }
        Value::Lexicon(entries) => EvalResult::Lexicon(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), value_to_eval_result(v)))
                .collect(),
        ),
    }
}

pub(crate) fn eval_result_to_value_any(result: EvalResult) -> Result<Value, EvalError> {
    match result {
        EvalResult::Omen(b) => Ok(Value::Omen(b)),
        EvalResult::Arcana(n) => Ok(Value::Arcana(n)),
        EvalResult::Aether(n) => Ok(Value::Aether(n)),
        EvalResult::Rune(s) => Ok(Value::Rune(s)),
        EvalResult::Abyss => Ok(Value::Abyss),
        EvalResult::Scroll(items) => {
            let converted: Result<Vec<_>, _> =
                items.into_iter().map(eval_result_to_value_any).collect();
            converted.map(Value::Scroll)
        }
        EvalResult::Lexicon(entries) => {
            let converted: Result<HashMap<_, _>, _> = entries
                .into_iter()
                .map(|(k, v)| eval_result_to_value_any(v).map(|v2| (k, v2)))
                .collect();
            converted.map(Value::Lexicon)
        }
        other => Err(EvalError::InvalidOperation(
            format!("Cannot convert {:?} to value", other),
            None,
        )),
    }
}

pub(crate) fn eval_result_to_value_checked(
    result: EvalResult,
    line_info: Option<LineInfo>,
) -> Result<Value, EvalError> {
    eval_result_to_value_any(result).map_err(|err| match err {
        EvalError::InvalidOperation(msg, _) => EvalError::InvalidOperation(msg, line_info.clone()),
        EvalError::TypeError(msg, _) => EvalError::TypeError(msg, line_info.clone()),
        other => other,
    })
}

pub(crate) fn convert_to_typed_value(
    result: EvalResult,
    expected: &Type,
    line_info: &Option<LineInfo>,
) -> Result<Value, EvalError> {
    match expected {
        Type::Materia => eval_result_to_value_checked(result, line_info.clone()),
        Type::Arcana => match result {
            EvalResult::Arcana(n) => Ok(Value::Arcana(n)),
            _ => Err(EvalError::TypeError(
                "Expected arcana value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Aether => match result {
            EvalResult::Aether(n) => Ok(Value::Aether(n)),
            _ => Err(EvalError::TypeError(
                "Expected aether value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Rune => match result {
            EvalResult::Rune(s) => Ok(Value::Rune(s)),
            _ => Err(EvalError::TypeError(
                "Expected rune value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Omen => match result {
            EvalResult::Omen(b) => Ok(Value::Omen(b)),
            _ => Err(EvalError::TypeError(
                "Expected omen value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Abyss => match result {
            EvalResult::Abyss => Ok(Value::Abyss),
            _ => Err(EvalError::TypeError(
                "Expected abyss value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Scroll => match result {
            EvalResult::Scroll(items) => {
                let converted: Vec<_> = items
                    .into_iter()
                    .map(|item| eval_result_to_value_checked(item, line_info.clone()))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Scroll(converted))
            }
            _ => Err(EvalError::TypeError(
                "Expected scroll value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Lexicon => match result {
            EvalResult::Lexicon(entries) => {
                let converted: HashMap<_, _> = entries
                    .into_iter()
                    .map(|(k, v)| {
                        eval_result_to_value_checked(v, line_info.clone())
                            .map(|converted| (k, converted))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(Value::Lexicon(converted))
            }
            _ => Err(EvalError::TypeError(
                "Expected lexicon value".to_string(),
                line_info.clone(),
            )),
        },
    }
}

pub(crate) fn extract_arcana(
    result: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<i64, EvalError> {
    match result {
        EvalResult::Arcana(v) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected arcana value".to_string(),
            line_info.clone(),
        )),
    }
}

pub(crate) fn extract_aether(
    result: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<f64, EvalError> {
    match result {
        EvalResult::Aether(v) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected aether value".to_string(),
            line_info.clone(),
        )),
    }
}

pub(crate) fn extract_rune(
    result: EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<String, EvalError> {
    match result {
        EvalResult::Rune(v) => Ok(v),
        _ => Err(EvalError::TypeError(
            "Expected rune value".to_string(),
            line_info.clone(),
        )),
    }
}

pub(crate) fn extract_omen(
    result: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<bool, EvalError> {
    match result {
        EvalResult::Omen(v) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected omen value".to_string(),
            line_info.clone(),
        )),
    }
}
