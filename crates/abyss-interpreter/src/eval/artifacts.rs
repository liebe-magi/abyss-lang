use crate::env::{
    ArtifactFieldSchema, ArtifactHandle, ArtifactSchema, ArtifactValue, RuntimeEnv, Value,
};
use abyss_core::ast::{AST, ArtifactField, LineInfo, Type};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::result::{EvalError, EvalResult};
use super::values::describe_value;
use crate::diagnostics::did_you_mean_hint;

pub(crate) fn ensure_type_known(
    ty: &Type,
    env: &RuntimeEnv,
    line_info: &Option<LineInfo>,
) -> Result<(), EvalError> {
    if let Type::Artifact(name) = ty
        && env.get_artifact(name).is_none()
    {
        return Err(EvalError::TypeError(
            format!("Artifact type {} is not defined", name),
            line_info.clone(),
        ));
    }
    Ok(())
}

pub(crate) fn ensure_field_type_known(
    field: &ArtifactField,
    env: &RuntimeEnv,
    current_artifact: &str,
) -> Result<(), EvalError> {
    match &field.field_type {
        Type::Artifact(name) if name == current_artifact => Ok(()),
        Type::Artifact(name) => {
            if env.get_artifact(name).is_some() {
                Ok(())
            } else {
                Err(EvalError::TypeError(
                    format!(
                        "Artifact field {} references undefined type {}",
                        field.name, name
                    ),
                    field.line_info.clone(),
                ))
            }
        }
        _ => Ok(()),
    }
}

pub(crate) fn build_artifact_schema(
    name: &str,
    fields: &[ArtifactField],
    env: &RuntimeEnv,
    line_info: &Option<LineInfo>,
) -> Result<ArtifactSchema, EvalError> {
    let mut seen = HashSet::new();
    let mut compiled_fields = Vec::with_capacity(fields.len());

    for field in fields {
        if !seen.insert(field.name.clone()) {
            return Err(EvalError::InvalidOperation(
                format!("Field '{}' is defined multiple times", field.name),
                field.line_info.clone().or_else(|| line_info.clone()),
            ));
        }
        ensure_field_type_known(field, env, name)?;
        compiled_fields.push(ArtifactFieldSchema {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
        });
    }

    Ok(ArtifactSchema {
        name: name.to_string(),
        fields: compiled_fields,
        methods: HashMap::new(),
        line_info: line_info.clone(),
    })
}

pub(crate) fn expect_artifact_handle(
    value: &Value,
    line_info: &Option<LineInfo>,
) -> Result<ArtifactHandle, EvalError> {
    match value {
        Value::Artifact(handle) => Ok(handle.clone()),
        other => Err(EvalError::InvalidOperation(
            format!("Expected artifact value, found {}", describe_value(other)),
            line_info.clone(),
        )),
    }
}

pub(crate) fn expect_artifact_from_eval(
    result: EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<ArtifactHandle, EvalError> {
    match result {
        EvalResult::Artifact(handle) => Ok(handle),
        EvalResult::Data(Value::Artifact(handle)) => Ok(handle),
        EvalResult::Data(other) => Err(EvalError::InvalidOperation(
            format!("Expected artifact value, found {}", describe_value(&other)),
            line_info.clone(),
        )),
        control => Err(EvalError::InvalidOperation(
            format!(
                "Expected artifact value but received control-flow result {:?}",
                control
            ),
            line_info.clone(),
        )),
    }
}

pub(crate) fn lookup_schema_by_name<'a>(
    env: &'a RuntimeEnv,
    type_name: &str,
    line_info: &Option<LineInfo>,
) -> Result<&'a ArtifactSchema, EvalError> {
    env.get_artifact(type_name).ok_or_else(|| {
        EvalError::InvalidOperation(
            format!("Artifact type {} is not defined", type_name),
            line_info.clone(),
        )
    })
}

pub(crate) fn lookup_schema_from_handle<'a>(
    env: &'a RuntimeEnv,
    handle: &ArtifactHandle,
    line_info: &Option<LineInfo>,
) -> Result<&'a ArtifactSchema, EvalError> {
    let type_name = handle.borrow().type_name.clone();
    lookup_schema_by_name(env, &type_name, line_info)
}

pub(crate) fn ensure_field_exists<'a>(
    schema: &'a ArtifactSchema,
    field: &str,
    line_info: &Option<LineInfo>,
) -> Result<&'a ArtifactFieldSchema, EvalError> {
    schema
        .field(field)
        .ok_or_else(|| missing_field_error(schema, field, line_info))
}

pub(crate) fn missing_field_error(
    schema: &ArtifactSchema,
    field: &str,
    line_info: &Option<LineInfo>,
) -> EvalError {
    let available_names = schema.field_names();
    let hint = did_you_mean_hint(field, available_names.iter().map(String::as_str), 3)
        .map(|h| format!(" {}", h))
        .unwrap_or_default();
    let available = available_names.join(", ");
    EvalError::InvalidOperation(
        format!(
            "Field '{}'{} does not exist on artifact {} (available: [{}])",
            field, hint, schema.name, available
        ),
        line_info.clone(),
    )
}

pub(crate) fn read_artifact_field(
    env: &RuntimeEnv,
    handle: &ArtifactHandle,
    field: &str,
    line_info: &Option<LineInfo>,
) -> Result<Value, EvalError> {
    let schema = lookup_schema_from_handle(env, handle, line_info)?;
    ensure_field_exists(schema, field, line_info)?;
    let borrowed = handle.borrow();
    borrowed
        .fields
        .get(field)
        .cloned()
        .ok_or_else(|| missing_field_error(schema, field, line_info))
}

pub(crate) fn instantiate_artifact_handle(
    type_name: &str,
    field_order: Vec<String>,
    fields: HashMap<String, Value>,
) -> ArtifactHandle {
    Rc::new(RefCell::new(ArtifactValue {
        type_name: type_name.to_string(),
        fields,
        field_order,
    }))
}

pub(crate) fn compare_artifacts(
    env: &RuntimeEnv,
    left: &ArtifactHandle,
    right: &ArtifactHandle,
    line_info: &Option<LineInfo>,
) -> Result<bool, EvalError> {
    let left_borrow = left.borrow();
    let right_borrow = right.borrow();

    if left_borrow.type_name != right_borrow.type_name {
        return Ok(false);
    }

    let schema = lookup_schema_by_name(env, &left_borrow.type_name, line_info)?;
    for field in &schema.fields {
        let left_value = left_borrow
            .fields
            .get(&field.name)
            .ok_or_else(|| missing_field_error(schema, &field.name, line_info))?;
        let right_value = right_borrow
            .fields
            .get(&field.name)
            .ok_or_else(|| missing_field_error(schema, &field.name, line_info))?;

        if !values_equal(env, left_value, right_value, line_info)? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Extracts the base variable name and field access chain from an AST expression.
///
/// This function only handles direct variable access (`AST::Var`) and field access chains
/// (`AST::FieldAccess`). It returns `None` for all other AST node types, including:
/// - Method calls (`AST::MethodCall`)
/// - Index access (`AST::IndexAccess`)
/// - Other complex expressions
///
/// This means that mutable method calls are only supported on direct variable/field access
/// patterns. Attempting to call a mutable method on a method call result or indexed expression
/// will produce an error indicating the expression is not tied to a mutable variable.
///
/// Returns `Some((base_var_name, field_chain))` if the expression can be traced to a variable,
/// or `None` if the expression type is not supported for mutability tracking.
pub(crate) fn collect_field_chain(ast: &AST) -> Option<(String, Vec<String>)> {
    match ast {
        AST::Var(name, _) => Some((name.clone(), Vec::new())),
        AST::FieldAccess { target, field, .. } => {
            let (base, mut chain) = collect_field_chain(target)?;
            chain.push(field.clone());
            Some((base, chain))
        }
        _ => None,
    }
}

pub(crate) fn values_equal(
    env: &RuntimeEnv,
    left: &Value,
    right: &Value,
    line_info: &Option<LineInfo>,
) -> Result<bool, EvalError> {
    match (left, right) {
        (Value::Omen(l), Value::Omen(r)) => Ok(l == r),
        (Value::Arcana(l), Value::Arcana(r)) => Ok(l == r),
        (Value::Aether(l), Value::Aether(r)) => Ok((l - r).abs() < f64::EPSILON),
        (Value::Rune(l), Value::Rune(r)) => Ok(l == r),
        (Value::Abyss, Value::Abyss) => Ok(true),
        (Value::Glyph(left), Value::Glyph(right)) => Ok(left == right),
        (Value::Artifact(l), Value::Artifact(r)) => compare_artifacts(env, l, r, line_info),
        (Value::Scroll(left_items), Value::Scroll(right_items)) => {
            let left_borrow = left_items.borrow();
            let right_borrow = right_items.borrow();
            if left_borrow.len() != right_borrow.len() {
                return Ok(false);
            }
            for (l, r) in left_borrow.iter().zip(right_borrow.iter()) {
                if !values_equal(env, l, r, line_info)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Lexicon(left_map), Value::Lexicon(right_map)) => {
            let left_borrow = left_map.borrow();
            let right_borrow = right_map.borrow();
            if left_borrow.len() != right_borrow.len() {
                return Ok(false);
            }
            for (key, left_value) in left_borrow.iter() {
                match right_borrow.get(key) {
                    Some(right_value) => {
                        if !values_equal(env, left_value, right_value, line_info)? {
                            return Ok(false);
                        }
                    }
                    None => return Ok(false),
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with_schema(name: &str, fields: Vec<(&str, Type)>) -> RuntimeEnv {
        let mut env = RuntimeEnv::new();
        env.define_artifact(ArtifactSchema {
            name: name.to_string(),
            fields: fields
                .into_iter()
                .map(|(field, ty)| ArtifactFieldSchema {
                    name: field.to_string(),
                    field_type: ty,
                })
                .collect(),
            methods: HashMap::new(),
            line_info: None,
        })
        .unwrap();
        env
    }

    fn artifact_handle(name: &str, entries: Vec<(&str, Value)>) -> ArtifactHandle {
        let mut map = HashMap::new();
        let mut order = Vec::new();
        for (key, value) in entries {
            let key_string = key.to_string();
            order.push(key_string.clone());
            map.insert(key_string, value);
        }
        instantiate_artifact_handle(name, order, map)
    }

    fn arcana(value: i64) -> Value {
        Value::Arcana(value)
    }

    fn rune(text: &str) -> Value {
        Value::Rune(Rc::new(text.to_string()))
    }

    fn scroll(values: Vec<Value>) -> Value {
        Value::Scroll(Rc::new(RefCell::new(values)))
    }

    fn lexicon(entries: Vec<(&str, Value)>) -> Value {
        let mut map = HashMap::new();
        for (key, value) in entries {
            map.insert(key.to_string(), value);
        }
        Value::Lexicon(Rc::new(RefCell::new(map)))
    }

    #[test]
    fn ensure_type_known_rejects_unknown_artifact_types() {
        let env = RuntimeEnv::new();
        let err = ensure_type_known(&Type::Artifact("Sigil".into()), &env, &None).unwrap_err();
        match err {
            EvalError::TypeError(_, _) => {}
            other => panic!("expected type error, got {:?}", other),
        }
    }

    #[test]
    fn ensure_field_type_known_validates_external_references() {
        let env = env_with_schema("Glyph", vec![("power", Type::Arcana)]);
        let valid = ArtifactField {
            name: "ally".into(),
            field_type: Type::Artifact("Glyph".into()),
            line_info: None,
        };
        ensure_field_type_known(&valid, &env, "Sigil").unwrap();

        let invalid = ArtifactField {
            name: "unknown".into(),
            field_type: Type::Artifact("Missing".into()),
            line_info: None,
        };
        let err = ensure_field_type_known(&invalid, &env, "Sigil").unwrap_err();
        match err {
            EvalError::TypeError(_, _) => {}
            other => panic!("expected type error, got {:?}", other),
        }
    }

    #[test]
    fn build_artifact_schema_detects_duplicate_fields() {
        let env = env_with_schema("Glyph", vec![]);
        let duplicate_fields = vec![
            ArtifactField {
                name: "power".into(),
                field_type: Type::Arcana,
                line_info: None,
            },
            ArtifactField {
                name: "power".into(),
                field_type: Type::Arcana,
                line_info: None,
            },
        ];

        let err = build_artifact_schema("Sigil", &duplicate_fields, &env, &None).unwrap_err();
        match err {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn expect_artifact_from_eval_accepts_multiple_sources() {
        let handle = artifact_handle("Sigil", vec![]);
        let eval_handle =
            expect_artifact_from_eval(EvalResult::Artifact(handle.clone()), &None).unwrap();
        assert!(Rc::ptr_eq(&handle, &eval_handle));

        let data_handle =
            expect_artifact_from_eval(EvalResult::Data(Value::Artifact(handle.clone())), &None)
                .unwrap();
        assert!(Rc::ptr_eq(&handle, &data_handle));

        let type_err = expect_artifact_from_eval(EvalResult::data(arcana(1)), &None).unwrap_err();
        match type_err {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("expected invalid operation, got {:?}", other),
        }

        let control_err = expect_artifact_from_eval(EvalResult::Resume(None), &None).unwrap_err();
        match control_err {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn lookup_schema_and_field_accessors_resolve_entries() {
        let env = env_with_schema("Sigil", vec![("power", Type::Arcana)]);
        let schema = lookup_schema_by_name(&env, "Sigil", &None).unwrap();
        assert_eq!(schema.name, "Sigil");

        let handle = artifact_handle("Sigil", vec![("power", arcana(3))]);
        let schema_from_handle = lookup_schema_from_handle(&env, &handle, &None).unwrap();
        assert_eq!(schema_from_handle.name, "Sigil");

        let field = ensure_field_exists(schema, "power", &None).unwrap();
        assert_eq!(field.name, "power");

        let missing = ensure_field_exists(schema, "missing", &None).unwrap_err();
        match missing {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("expected invalid operation, got {:?}", other),
        }

        let value = read_artifact_field(&env, &handle, "power", &None).unwrap();
        assert!(matches!(value, Value::Arcana(3)));

        let missing_err = read_artifact_field(&env, &handle, "missing", &None).unwrap_err();
        match missing_err {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn compare_artifacts_checks_schema_and_values() {
        let env = env_with_schema("Sigil", vec![("power", Type::Arcana), ("text", Type::Rune)]);
        let left = artifact_handle("Sigil", vec![("power", arcana(3)), ("text", rune("alpha"))]);
        let right_same =
            artifact_handle("Sigil", vec![("power", arcana(3)), ("text", rune("alpha"))]);
        assert!(compare_artifacts(&env, &left, &right_same, &None).unwrap());

        let right_diff =
            artifact_handle("Sigil", vec![("power", arcana(4)), ("text", rune("alpha"))]);
        assert!(!compare_artifacts(&env, &left, &right_diff, &None).unwrap());

        let other = artifact_handle("Glyph", vec![("power", arcana(3))]);
        assert!(!compare_artifacts(&env, &left, &other, &None).unwrap());
    }

    #[test]
    fn collect_field_chain_tracks_nested_field_access() {
        let chain = AST::FieldAccess {
            target: Box::new(AST::FieldAccess {
                target: Box::new(AST::Var("sigil".into(), None)),
                field: "inner".into(),
                line_info: None,
            }),
            field: "deep".into(),
            line_info: None,
        };

        let (base, fields) = collect_field_chain(&chain).expect("chain should resolve");
        assert_eq!(base, "sigil");
        assert_eq!(fields, vec!["inner".to_string(), "deep".to_string()]);

        assert!(collect_field_chain(&AST::Abyss(None)).is_none());
    }

    #[test]
    fn values_equal_handles_nested_structures() {
        let env = env_with_schema("Sigil", vec![]);
        let scroll_a = scroll(vec![arcana(1), arcana(2)]);
        let scroll_b = scroll(vec![arcana(1), arcana(2)]);
        assert!(values_equal(&env, &scroll_a, &scroll_b, &None).unwrap());

        let lex_a = lexicon(vec![("key", rune("alpha"))]);
        let lex_b = lexicon(vec![("key", rune("alpha"))]);
        assert!(values_equal(&env, &lex_a, &lex_b, &None).unwrap());

        let lex_c = lexicon(vec![("key", rune("beta"))]);
        assert!(!values_equal(&env, &lex_a, &lex_c, &None).unwrap());
    }

    #[test]
    fn missing_field_error_appends_did_you_mean_for_close_match() {
        let env = env_with_schema(
            "Player",
            vec![
                ("name", Type::Rune),
                ("hp", Type::Arcana),
                ("mp", Type::Arcana),
            ],
        );
        let schema = lookup_schema_by_name(&env, "Player", &None).unwrap();
        let err = missing_field_error(schema, "nmae", &None);
        match err {
            EvalError::InvalidOperation(msg, _) => {
                assert!(
                    msg.contains(
                        "Field 'nmae' (did you mean: name?) does not exist on artifact Player"
                    ),
                    "missing did-you-mean hint in: {msg}"
                );
                // The "available" listing should still be present so users can
                // see the full schema even when a close match is suggested.
                assert!(msg.contains("(available: [name, hp, mp])"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn missing_field_error_omits_hint_when_no_close_match() {
        let env = env_with_schema("Player", vec![("name", Type::Rune), ("hp", Type::Arcana)]);
        let schema = lookup_schema_by_name(&env, "Player", &None).unwrap();
        let err = missing_field_error(schema, "completely_unrelated_field", &None);
        match err {
            EvalError::InvalidOperation(msg, _) => {
                assert!(!msg.contains("did you mean"), "msg: {msg}");
                assert!(msg.contains("(available: [name, hp])"));
            }
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }
}
