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
        Type::Glyph => match value {
            Value::Glyph(_) => Ok(value),
            _ => Err(EvalError::TypeError(
                "Expected glyph value".to_string(),
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
        Value::Glyph(_) => "glyph",
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
        Value::Glyph(ty) => Value::Glyph(ty.clone()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    fn artifact_handle(name: &str, fields: Vec<(&str, Value)>) -> ArtifactHandle {
        let mut map = HashMap::new();
        let mut order = Vec::new();
        for (key, value) in fields {
            let key_string = key.to_string();
            order.push(key_string.clone());
            map.insert(key_string, value);
        }
        Rc::new(RefCell::new(ArtifactValue {
            type_name: name.to_string(),
            fields: map,
            field_order: order,
        }))
    }

    fn line() -> Option<LineInfo> {
        Some(LineInfo::new(2, 3))
    }

    #[test]
    fn value_to_eval_result_handles_artifacts_and_scalars() {
        let rune = Value::Rune(Rc::new("sigil".to_string()));
        match value_to_eval_result(&rune) {
            EvalResult::Data(Value::Rune(text)) => assert_eq!(text.as_ref(), "sigil"),
            other => panic!("expected rune data, got {:?}", other),
        }

        let handle = artifact_handle("Glyph", vec![("power", Value::Arcana(3))]);
        match value_to_eval_result(&Value::Artifact(handle.clone())) {
            EvalResult::Artifact(result_handle) => assert!(Rc::ptr_eq(&handle, &result_handle)),
            other => panic!("expected artifact handle, got {:?}", other),
        }
    }

    #[test]
    fn eval_result_to_value_checked_overrides_line_info() {
        let info = line();
        let err = eval_result_to_value_checked(EvalResult::Resume(None), info.clone())
            .expect_err("control flow should error");
        match err {
            EvalError::InvalidOperation(_, returned) => {
                let returned = returned.expect("line info should propagate");
                assert_eq!((returned.line, returned.column), (2, 3));
            }
            other => panic!("unexpected error {:?}", other),
        }
    }

    #[test]
    fn convert_to_typed_value_validates_types_and_clones() {
        let info = line();
        let arcana =
            convert_to_typed_value(EvalResult::data(Value::Arcana(5)), &Type::Arcana, &info)
                .expect("arcana conversion should pass");
        assert!(matches!(arcana, Value::Arcana(5)));

        let rune_err = convert_to_typed_value(
            EvalResult::data(Value::Rune(Rc::new("sigil".into()))),
            &Type::Arcana,
            &info,
        )
        .expect_err("type mismatch should error");
        match rune_err {
            EvalError::TypeError(_, info) => assert!(info.is_some()),
            other => panic!("expected type error, got {:?}", other),
        }

        let handle = artifact_handle("Sigil", vec![("power", Value::Arcana(9))]);
        let info = line();
        let materia =
            convert_to_typed_value(EvalResult::artifact(handle.clone()), &Type::Materia, &info)
                .expect("materia should accept artifact");
        let cloned_handle = match materia {
            Value::Artifact(h) => h,
            other => panic!("expected artifact value, got {:?}", other),
        };
        assert!(!Rc::ptr_eq(&handle, &cloned_handle));
        assert_eq!(handle.borrow().type_name, cloned_handle.borrow().type_name);

        let info = line();
        let artifact = convert_to_typed_value(
            EvalResult::artifact(handle.clone()),
            &Type::Artifact("Sigil".into()),
            &info,
        )
        .expect("matching artifact type should pass");
        assert!(matches!(artifact, Value::Artifact(_)));

        let info = line();
        let wrong_artifact = convert_to_typed_value(
            EvalResult::artifact(handle.clone()),
            &Type::Artifact("Glyph".into()),
            &info,
        )
        .expect_err("mismatched artifact type should fail");
        match wrong_artifact {
            EvalError::TypeError(msg, _) => assert!(msg.contains("Glyph")),
            other => panic!("expected type error, got {:?}", other),
        }

        let info = line();
        let control_err = convert_to_typed_value(EvalResult::Resume(None), &Type::Arcana, &info)
            .expect_err("control flow should not convert");
        match control_err {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn extractors_return_expected_types() {
        let info = line();
        let arcana = extract_arcana(&EvalResult::data(Value::Arcana(7)), &info)
            .expect("arcana extraction should pass");
        assert_eq!(arcana, 7);

        let info = line();
        let aether = extract_aether(&EvalResult::data(Value::Aether(3.5)), &info)
            .expect("aether extraction should pass");
        assert!((aether - 3.5).abs() < f64::EPSILON);

        let info = line();
        let rune = extract_rune(
            &EvalResult::data(Value::Rune(Rc::new("sigil".into()))),
            &info,
        )
        .expect("rune extraction should pass");
        assert_eq!(rune, "sigil");

        let info = line();
        let omen = extract_omen(&EvalResult::data(Value::Omen(true)), &info)
            .expect("omen extraction should pass");
        assert!(omen);
    }

    #[test]
    fn extractors_error_on_wrong_type() {
        let info = line();
        let err = extract_rune(&EvalResult::data(Value::Arcana(1)), &info).unwrap_err();
        match err {
            EvalError::TypeError(_, info) => assert!(info.is_some()),
            other => panic!("expected type error, got {:?}", other),
        }

        let resume = EvalResult::Resume(None);
        let info = line();
        let err = extract_aether(&resume, &info).unwrap_err();
        match err {
            EvalError::TypeError(_, info) => assert!(info.is_some()),
            other => panic!("expected type error from control value, got {:?}", other),
        }
    }

    #[test]
    fn describe_value_matches_variants() {
        assert_eq!(describe_value(&Value::Omen(true)), "omen");
        assert_eq!(describe_value(&Value::Arcana(1)), "arcana");
        assert_eq!(describe_value(&Value::Aether(2.0)), "aether");
        assert_eq!(describe_value(&Value::Rune(Rc::new(String::new()))), "rune");
        assert_eq!(describe_value(&Value::Abyss), "abyss");
        assert_eq!(
            describe_value(&Value::Scroll(Rc::new(RefCell::new(Vec::new())))),
            "scroll"
        );
        assert_eq!(
            describe_value(&Value::Lexicon(Rc::new(RefCell::new(HashMap::new())))),
            "lexicon"
        );
        assert_eq!(
            describe_value(&Value::Artifact(artifact_handle("Sigil", vec![]))),
            "artifact"
        );
    }

    #[test]
    fn clone_value_performs_deep_copies() {
        let scroll = Value::Scroll(Rc::new(RefCell::new(vec![Value::Arcana(1)])));
        let cloned_scroll = clone_value(&scroll);
        if let (Value::Scroll(orig), Value::Scroll(cloned)) = (&scroll, &cloned_scroll) {
            assert!(!Rc::ptr_eq(orig, cloned));
            orig.borrow_mut().push(Value::Arcana(2));
            assert_eq!(cloned.borrow().len(), 1);
        } else {
            panic!("expected scroll clone");
        }

        let lexicon = Value::Lexicon(Rc::new(RefCell::new(HashMap::from([(
            "sigil".into(),
            Value::Arcana(1),
        )]))));
        let cloned_lexicon = clone_value(&lexicon);
        if let (Value::Lexicon(orig), Value::Lexicon(cloned)) = (&lexicon, &cloned_lexicon) {
            assert!(!Rc::ptr_eq(orig, cloned));
            orig.borrow_mut().insert("new".into(), Value::Arcana(2));
            assert_eq!(cloned.borrow().len(), 1);
        } else {
            panic!("expected lexicon clone");
        }

        let artifact = Value::Artifact(artifact_handle("Sigil", vec![("power", Value::Arcana(1))]));
        let cloned_artifact = clone_value(&artifact);
        if let (Value::Artifact(orig), Value::Artifact(cloned)) = (&artifact, &cloned_artifact) {
            assert!(!Rc::ptr_eq(orig, cloned));
            assert_eq!(orig.borrow().fields.len(), cloned.borrow().fields.len());
        } else {
            panic!("expected artifact clone");
        }
    }
}
