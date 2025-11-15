use crate::ast::{LineInfo, Type};
use crate::env::Value;
use std::rc::Rc;

use super::result::{EvalError, EvalResult};

pub(crate) fn value_to_eval_result(value: &Value) -> EvalResult {
    EvalResult::data(value.clone())
}

pub(crate) fn eval_result_to_value_any(result: EvalResult) -> Result<Value, EvalError> {
    match result {
        EvalResult::Data(value) => Ok(value),
        EvalResult::Revealed(_) | EvalResult::Resume(_) | EvalResult::Eject(_) => {
            Err(EvalError::InvalidOperation(
                "Control-flow result cannot be treated as data".to_string(),
                None,
            ))
        }
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
    let value = match result {
        EvalResult::Data(value) => value,
        control => {
            return Err(EvalError::InvalidOperation(
                format!("Expected data value but received {:?}", control),
                line_info.clone(),
            ));
        }
    };

    match expected {
        Type::Materia => Ok(value),
        Type::Arcana => match value {
            Value::Arcana(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected arcana value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Aether => match value {
            Value::Aether(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected aether value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Rune => match value {
            Value::Rune(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected rune value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Omen => match value {
            Value::Omen(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected omen value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Abyss => match value {
            Value::Abyss => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected abyss value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Scroll => match value {
            Value::Scroll(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected scroll value".to_string(),
                line_info.clone(),
            )),
        },
        Type::Lexicon => match value {
            Value::Lexicon(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected lexicon value".to_string(),
                line_info.clone(),
            )),
        },
    }
}

fn rune_to_string(rc: &Rc<String>) -> String {
    rc.as_ref().clone()
}

pub(crate) fn extract_arcana(
    result: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<i64, EvalError> {
    match result {
        EvalResult::Data(Value::Arcana(v)) => Ok(*v),
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
        EvalResult::Data(Value::Aether(v)) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected aether value".to_string(),
            line_info.clone(),
        )),
    }
}

pub(crate) fn extract_rune(
    result: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<String, EvalError> {
    match result {
        EvalResult::Data(Value::Rune(rc)) => Ok(rune_to_string(rc)),
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
        EvalResult::Data(Value::Omen(v)) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected omen value".to_string(),
            line_info.clone(),
        )),
    }
}
