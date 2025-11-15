use crate::ast::{LineInfo, Type};
use crate::env::{ArtifactHandle, ArtifactValue, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::result::{EvalError, EvalResult};

pub(crate) fn value_to_eval_result(value: &Value) -> EvalResult {
    match value {
        Value::Artifact(handle) => EvalResult::Artifact(handle.clone()),
        _ => EvalResult::data(value.clone()),
    }
}

pub(crate) fn eval_result_to_value_any(result: EvalResult) -> Result<Value, EvalError> {
    match result {
        EvalResult::Data(value) => Ok(value),
        EvalResult::Artifact(handle) => Ok(Value::Artifact(handle)),
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
        EvalResult::Artifact(handle) => Value::Artifact(handle),
        control => {
            return Err(EvalError::InvalidOperation(
                format!("Expected data value but received {:?}", control),
                line_info.clone(),
            ));
        }
    };

    match expected {
        Type::Materia => Ok(match value {
            Value::Artifact(handle) => Value::Artifact(clone_artifact_handle(&handle)),
            other => other,
        }),
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
        Type::Artifact(expected) => match value {
            Value::Artifact(handle) => {
                let borrowed = handle.borrow();
                if &borrowed.type_name == expected {
                    Ok(Value::Artifact(clone_artifact_handle(&handle)))
                } else {
                    Err(EvalError::TypeError(
                        format!(
                            "Expected artifact of type {} but received {}",
                            expected, borrowed.type_name
                        ),
                        line_info.clone(),
                    ))
                }
            }
            _ => Err(EvalError::TypeError(
                format!("Expected artifact of type {}", expected),
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
        EvalResult::Data(Value::Rune(rc)) => Ok(rc.as_ref().clone()),
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

pub(crate) fn describe_value(value: &Value) -> &'static str {
    match value {
        Value::Omen(_) => "omen",
        Value::Arcana(_) => "arcana",
        Value::Aether(_) => "aether",
        Value::Rune(_) => "rune",
        Value::Abyss => "abyss",
        Value::Scroll(_) => "scroll",
        Value::Lexicon(_) => "lexicon",
        Value::Artifact(_) => "artifact",
    }
}

fn clone_artifact_handle(handle: &ArtifactHandle) -> ArtifactHandle {
    let borrowed = handle.borrow();
    let mut cloned_fields = HashMap::new();
    for (key, value) in borrowed.fields.iter() {
        cloned_fields.insert(key.clone(), clone_value(value));
    }
    Rc::new(RefCell::new(ArtifactValue {
        type_name: borrowed.type_name.clone(),
        fields: cloned_fields,
        field_order: borrowed.field_order.clone(),
    }))
}

fn clone_value(value: &Value) -> Value {
    match value {
        Value::Omen(v) => Value::Omen(*v),
        Value::Arcana(v) => Value::Arcana(*v),
        Value::Aether(v) => Value::Aether(*v),
        Value::Rune(r) => Value::Rune(r.clone()),
        Value::Abyss => Value::Abyss,
        Value::Scroll(values) => Value::Scroll(clone_scroll(values)),
        Value::Lexicon(entries) => Value::Lexicon(clone_lexicon(entries)),
        Value::Artifact(handle) => Value::Artifact(clone_artifact_handle(handle)),
    }
}

fn clone_scroll(values: &Rc<RefCell<Vec<Value>>>) -> Rc<RefCell<Vec<Value>>> {
    let borrowed = values.borrow();
    let mut cloned = Vec::with_capacity(borrowed.len());
    for value in borrowed.iter() {
        cloned.push(clone_value(value));
    }
    Rc::new(RefCell::new(cloned))
}

fn clone_lexicon(
    entries: &Rc<RefCell<HashMap<String, Value>>>,
) -> Rc<RefCell<HashMap<String, Value>>> {
    let borrowed = entries.borrow();
    let mut cloned = HashMap::with_capacity(borrowed.len());
    for (key, value) in borrowed.iter() {
        cloned.insert(key.clone(), clone_value(value));
    }
    Rc::new(RefCell::new(cloned))
}
