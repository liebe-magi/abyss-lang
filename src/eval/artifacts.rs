use crate::ast::{AST, ArtifactField, LineInfo, Type};
use crate::env::{
    ArtifactFieldSchema, ArtifactHandle, ArtifactSchema, ArtifactValue, Environment, Value,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::result::{EvalError, EvalResult};
use super::values::describe_value;

pub(crate) fn ensure_type_known(
    ty: &Type,
    env: &Environment,
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
    env: &Environment,
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
    env: &Environment,
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
    env: &'a Environment,
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
    env: &'a Environment,
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
    let available = schema.field_names().join(", ");
    EvalError::InvalidOperation(
        format!(
            "Field '{}' does not exist on artifact {} (available: [{}])",
            field, schema.name, available
        ),
        line_info.clone(),
    )
}

pub(crate) fn read_artifact_field(
    env: &Environment,
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
    env: &Environment,
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

fn values_equal(
    env: &Environment,
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
