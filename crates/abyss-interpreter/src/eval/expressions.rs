use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::env::{CallArg, Callable, EngravedFunction, RuntimeEnv, Value};
use crate::stdlib::methods;
use abyss_core::ast::{AST, LineInfo, Type};

use super::artifacts::{
    collect_field_chain, compare_artifacts, ensure_field_exists, expect_artifact_from_eval,
    instantiate_artifact_handle, lookup_schema_by_name, lookup_schema_from_handle,
    read_artifact_field,
};
use super::collections::{expect_arcana_index, expect_rune_key};
use super::result::{EvalError, EvalResult};
use super::statements;
use super::values::{
    convert_to_typed_value, describe_value, eval_result_to_value_checked, value_to_eval_result,
};

pub(crate) fn try_evaluate_expression(
    ast: &AST,
    env: &mut RuntimeEnv,
) -> Result<Option<EvalResult>, EvalError> {
    let result = match ast {
        AST::Omen(value, _) => return Ok(Some(EvalResult::data(Value::Omen(*value)))),
        AST::Arcana(value, _) => return Ok(Some(EvalResult::data(Value::Arcana(*value)))),
        AST::Aether(value, _) => return Ok(Some(EvalResult::data(Value::Aether(*value)))),
        AST::Rune(value, _) => {
            return Ok(Some(EvalResult::data(Value::Rune(Rc::new(value.clone())))));
        }
        AST::Abyss(_) => return Ok(Some(EvalResult::abyss())),
        AST::ListLiteral {
            elements,
            line_info,
        } => {
            let mut evaluated = Vec::with_capacity(elements.len());
            for element in elements {
                evaluated.push(eval_result_to_value_checked(
                    statements::evaluate(element, env)?,
                    line_info.clone(),
                )?);
            }
            EvalResult::data(Value::Scroll(Rc::new(RefCell::new(evaluated))))
        }
        AST::MapLiteral { entries, line_info } => {
            let mut map = HashMap::new();
            for (key, expr) in entries {
                let value = eval_result_to_value_checked(
                    statements::evaluate(expr, env)?,
                    line_info.clone(),
                )?;
                map.insert(key.clone(), value);
            }
            EvalResult::data(Value::Lexicon(Rc::new(RefCell::new(map))))
        }
        AST::ArtifactLiteral {
            type_name,
            fields,
            line_info,
        } => instantiate_artifact_literal(env, type_name, fields, line_info)?,
        AST::FieldAccess {
            target,
            field,
            line_info,
        } => {
            let value = statements::evaluate(target, env)?;
            let handle = expect_artifact_from_eval(value, line_info)?;
            let field_value = read_artifact_field(env, &handle, field, line_info)?;
            match field_value {
                Value::Artifact(inner) => EvalResult::Artifact(inner),
                other => EvalResult::data(other),
            }
        }
        AST::Add(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l + r,
            |l, r| l + r,
            Some(|l: String, r: String| format!("{}{}", l, r)),
        )?,
        AST::Sub(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l - r,
            |l, r| l - r,
            None,
        )?,
        AST::Mul(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l * r,
            |l, r| l * r,
            None,
        )?,
        AST::Div(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l / r,
            |l, r| l / r,
            None,
        )?,
        AST::Mod(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l % r,
            |l, r| l % r,
            None,
        )?,
        AST::PowArcana(left, right, line_info) => match (
            statements::evaluate(left, env)?,
            statements::evaluate(right, env)?,
        ) {
            (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => {
                if r < 0 {
                    return Err(EvalError::NegativeExponent(line_info.clone()));
                }
                EvalResult::data(Value::Arcana(l.pow(r as u32)))
            }
            _ => {
                return Err(EvalError::InvalidOperation(
                    "PowArcana operation requires two Arcana!".to_string(),
                    line_info.clone(),
                ));
            }
        },
        AST::PowAether(left, right, line_info) => match (
            statements::evaluate(left, env)?,
            statements::evaluate(right, env)?,
        ) {
            (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
                EvalResult::data(Value::Aether(l.powf(r)))
            }
            _ => {
                return Err(EvalError::InvalidOperation(
                    "PowAether operation requires two Aether!".to_string(),
                    line_info.clone(),
                ));
            }
        },
        AST::Equal(left, right, line_info) => compare_values(env, left, right, line_info, true)?,
        AST::NotEqual(left, right, line_info) => {
            compare_values(env, left, right, line_info, false)?
        }
        AST::LessThan(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l < r)?
        }
        AST::LessThanOrEqual(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l <= r)?
        }
        AST::GreaterThan(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l > r)?
        }
        AST::GreaterThanOrEqual(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l >= r)?
        }
        AST::LogicalAnd(left, right, line_info) => {
            logical_op(env, left, right, line_info, |l, r| l && r)?
        }
        AST::LogicalOr(left, right, line_info) => {
            logical_op(env, left, right, line_info, |l, r| l || r)?
        }
        AST::LogicalNot(expr, line_info) => {
            let result = statements::evaluate(expr, env)?;
            match result {
                EvalResult::Data(Value::Omen(value)) => EvalResult::data(Value::Omen(!value)),
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "LogicalNot operation requires Omen!".to_string(),
                        line_info.clone(),
                    ));
                }
            }
        }
        AST::Var(name, line_info) => match env.get_var(name) {
            Some(var_info) => value_to_eval_result(&var_info.value),
            None => {
                return Err(EvalError::UndefinedVariable(
                    name.clone(),
                    line_info.clone(),
                ));
            }
        },
        AST::IndexAccess {
            target,
            index,
            line_info,
        } => {
            let collection = statements::evaluate(target, env)?;
            let idx_value = statements::evaluate(index, env)?;
            match collection {
                EvalResult::Data(Value::Scroll(items)) => {
                    let idx = expect_arcana_index(&idx_value, line_info)?;
                    let borrowed = items.borrow();
                    let value = borrowed.get(idx).cloned().ok_or_else(|| {
                        EvalError::InvalidOperation(
                            format!("Index {} is out of bounds for scroll", idx),
                            line_info.clone(),
                        )
                    })?;
                    EvalResult::data(value)
                }
                EvalResult::Data(Value::Lexicon(entries)) => {
                    let key = expect_rune_key(&idx_value, line_info)?;
                    let borrowed = entries.borrow();
                    let value = borrowed.get(&key).cloned().ok_or_else(|| {
                        EvalError::InvalidOperation(
                            format!("Lexicon key '{}' does not exist", key),
                            line_info.clone(),
                        )
                    })?;
                    EvalResult::data(value)
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "Indexing is only supported for scroll or lexicon".to_string(),
                        line_info.clone(),
                    ));
                }
            }
        }
        AST::FuncCall {
            name,
            args,
            line_info,
        } => evaluate_function_call(env, name, args, line_info)?,
        AST::MethodCall {
            receiver,
            method,
            args,
            line_info,
        } => evaluate_method_call(env, receiver, method, args, line_info)?,
        AST::OracleDontCareItem(_) => EvalResult::data(Value::Omen(true)),
        _ => return Ok(None),
    };

    Ok(Some(result))
}

fn binary_numeric_op<TArc, TAether>(
    env: &mut RuntimeEnv,
    left: &AST,
    right: &AST,
    line_info: &Option<LineInfo>,
    arcana_op: TArc,
    aether_op: TAether,
    rune_op: Option<fn(String, String) -> String>,
) -> Result<EvalResult, EvalError>
where
    TArc: FnOnce(i64, i64) -> i64,
    TAether: FnOnce(f64, f64) -> f64,
{
    let left_result = statements::evaluate(left, env)?;
    let right_result = statements::evaluate(right, env)?;

    match (left_result, right_result) {
        (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => {
            Ok(EvalResult::data(Value::Arcana(arcana_op(l, r))))
        }
        (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
            Ok(EvalResult::data(Value::Aether(aether_op(l, r))))
        }
        (EvalResult::Data(Value::Rune(l)), EvalResult::Data(Value::Rune(r)))
            if rune_op.is_some() =>
        {
            let op = rune_op.unwrap();
            Ok(EvalResult::data(Value::Rune(Rc::new(op(
                l.as_ref().clone(),
                r.as_ref().clone(),
            )))))
        }
        _ => Err(EvalError::InvalidOperation(
            "Operation requires compatible types".to_string(),
            line_info.clone(),
        )),
    }
}

fn instantiate_artifact_literal(
    env: &mut RuntimeEnv,
    type_name: &str,
    fields: &[(String, AST)],
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let schema = lookup_schema_by_name(env, type_name, line_info)?.clone();
    let mut provided = HashSet::new();
    let mut values = HashMap::new();

    for (field_name, expr) in fields {
        if !provided.insert(field_name.clone()) {
            return Err(EvalError::InvalidOperation(
                format!("Field '{}' is provided multiple times", field_name),
                line_info.clone(),
            ));
        }

        let field_schema = ensure_field_exists(&schema, field_name, line_info)?;
        let evaluated = statements::evaluate(expr, env)?;
        let typed_value = convert_to_typed_value(evaluated, &field_schema.field_type, line_info)?;
        values.insert(field_name.clone(), typed_value);
    }

    let field_order = schema.field_names();
    let missing: Vec<String> = field_order
        .iter()
        .filter(|name| !values.contains_key(*name))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(EvalError::InvalidOperation(
            format!(
                "Artifact {} literal is missing fields: {}",
                type_name,
                missing.join(", ")
            ),
            line_info.clone(),
        ));
    }

    Ok(EvalResult::Artifact(instantiate_artifact_handle(
        type_name,
        field_order,
        values,
    )))
}

fn compare_values(
    env: &mut RuntimeEnv,
    left: &AST,
    right: &AST,
    line_info: &Option<LineInfo>,
    equality: bool,
) -> Result<EvalResult, EvalError> {
    let left_result = statements::evaluate(left, env)?;
    let right_result = statements::evaluate(right, env)?;

    let comparison = match (left_result, right_result) {
        (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => l == r,
        (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
            (l - r).abs() < f64::EPSILON
        }
        (EvalResult::Data(Value::Rune(l)), EvalResult::Data(Value::Rune(r))) => l == r,
        (EvalResult::Artifact(left), EvalResult::Artifact(right))
        | (EvalResult::Artifact(left), EvalResult::Data(Value::Artifact(right)))
        | (EvalResult::Data(Value::Artifact(left)), EvalResult::Artifact(right))
        | (EvalResult::Data(Value::Artifact(left)), EvalResult::Data(Value::Artifact(right))) => {
            compare_artifacts(env, &left, &right, line_info)?
        }
        _ => {
            return Err(EvalError::InvalidOperation(
                "Comparison requires compatible types!".to_string(),
                line_info.clone(),
            ));
        }
    };

    let result = if equality { comparison } else { !comparison };
    Ok(EvalResult::data(Value::Omen(result)))
}

fn order_values<F>(
    env: &mut RuntimeEnv,
    left: &AST,
    right: &AST,
    line_info: &Option<LineInfo>,
    comparator: F,
) -> Result<EvalResult, EvalError>
where
    F: FnOnce(f64, f64) -> bool,
{
    let left_result = statements::evaluate(left, env)?;
    let right_result = statements::evaluate(right, env)?;

    match (left_result, right_result) {
        (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => Ok(
            EvalResult::data(Value::Omen(comparator(l as f64, r as f64))),
        ),
        (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
            Ok(EvalResult::data(Value::Omen(comparator(l, r))))
        }
        _ => Err(EvalError::InvalidOperation(
            "Comparison requires numeric types!".to_string(),
            line_info.clone(),
        )),
    }
}

fn logical_op<F>(
    env: &mut RuntimeEnv,
    left: &AST,
    right: &AST,
    line_info: &Option<LineInfo>,
    op: F,
) -> Result<EvalResult, EvalError>
where
    F: FnOnce(bool, bool) -> bool,
{
    let left_result = statements::evaluate(left, env)?;
    let right_result = statements::evaluate(right, env)?;

    match (left_result, right_result) {
        (EvalResult::Data(Value::Omen(l)), EvalResult::Data(Value::Omen(r))) => {
            Ok(EvalResult::data(Value::Omen(op(l, r))))
        }
        _ => Err(EvalError::InvalidOperation(
            "Logical operation requires two Omen!".to_string(),
            line_info.clone(),
        )),
    }
}

fn evaluate_method_call(
    env: &mut RuntimeEnv,
    receiver: &AST,
    method_name: &str,
    args: &[AST],
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let receiver_result = statements::evaluate(receiver, env)?;
    let receiver_var_name = if let AST::Var(var_name, _) = receiver {
        Some(var_name.clone())
    } else {
        None
    };

    let mut evaluated_args = Vec::with_capacity(args.len());
    for arg in args {
        let evaluated_arg = statements::evaluate(arg, env)?;
        let var_name = if let AST::Var(var_name, _) = arg {
            Some(var_name.clone())
        } else {
            None
        };
        evaluated_args.push(CallArg {
            value: evaluated_arg,
            var_name,
        });
    }

    match receiver_result {
        EvalResult::Artifact(handle) => evaluate_artifact_method_call(
            env,
            receiver,
            receiver_var_name,
            handle,
            method_name,
            evaluated_args,
            line_info,
        ),
        EvalResult::Data(Value::Artifact(handle)) => evaluate_artifact_method_call(
            env,
            receiver,
            receiver_var_name,
            handle,
            method_name,
            evaluated_args,
            line_info,
        ),
        EvalResult::Data(value) => methods::dispatch_builtin_method(
            env,
            receiver,
            receiver_var_name.as_deref(),
            value,
            method_name,
            evaluated_args,
            line_info,
        ),
        control => Err(EvalError::InvalidOperation(
            format!(
                "Cannot invoke method {} on control-flow result {:?}",
                method_name, control
            ),
            line_info.clone(),
        )),
    }
}

fn ensure_method_receiver_mutability(
    env: &RuntimeEnv,
    receiver: &AST,
    artifact_name: &str,
    method_name: &str,
    line_info: &Option<LineInfo>,
) -> Result<(), EvalError> {
    if let Some((base_name, _)) = collect_field_chain(receiver) {
        if let Some(var_info) = env.get_var(&base_name) {
            if var_info.is_morph {
                return Ok(());
            }
            return Err(EvalError::InvalidOperation(
                format!(
                    "Cannot call {}::{} with immutable receiver '{}'",
                    artifact_name, method_name, base_name
                ),
                line_info.clone(),
            ));
        } else {
            return Err(EvalError::UndefinedVariable(base_name, line_info.clone()));
        }
    }

    Err(EvalError::InvalidOperation(
        format!(
            "Method {}::{} requires a morph receiver, but the expression is not tied to a mutable variable",
            artifact_name, method_name
        ),
        line_info.clone(),
    ))
}

fn evaluate_artifact_method_call(
    env: &mut RuntimeEnv,
    receiver_ast: &AST,
    receiver_var_name: Option<String>,
    receiver_handle: crate::env::ArtifactHandle,
    method_name: &str,
    arg_values: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let schema = lookup_schema_from_handle(env, &receiver_handle, line_info)?;
    let artifact_name = schema.name.clone();
    let artifact_method = env
        .get_artifact_method(&artifact_name, method_name)
        .ok_or_else(|| {
            EvalError::InvalidOperation(
                format!("Method {}::{} is not defined", artifact_name, method_name),
                line_info.clone(),
            )
        })?;

    if artifact_method.requires_mutable_receiver {
        ensure_method_receiver_mutability(
            env,
            receiver_ast,
            &artifact_name,
            method_name,
            line_info,
        )?;
    }

    let mut evaluated_args = Vec::with_capacity(arg_values.len() + 1);
    evaluated_args.push(CallArg {
        value: EvalResult::Artifact(receiver_handle),
        var_name: receiver_var_name,
    });
    evaluated_args.extend(arg_values);

    evaluate_engraved_function(
        env,
        evaluated_args,
        artifact_method.function.clone(),
        line_info,
    )
}

fn evaluate_function_call(
    env: &mut RuntimeEnv,
    name: &str,
    args: &[AST],
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let callable = match env.get_function(name) {
        Some(func) => func.clone(),
        None => {
            return Err(EvalError::UndefinedVariable(
                name.to_string(),
                line_info.clone(),
            ));
        }
    };

    let mut evaluated_args = Vec::new();
    for arg in args {
        let evaluated_arg = statements::evaluate(arg, env)?;
        let var_name = if let AST::Var(var_name, _) = arg {
            Some(var_name.clone())
        } else {
            None
        };
        evaluated_args.push(CallArg {
            value: evaluated_arg,
            var_name,
        });
    }

    match callable {
        Callable::Engraved(function) => {
            evaluate_engraved_function(env, evaluated_args, function, line_info)
        }
        Callable::Builtin(function) => (function.func)(env, evaluated_args, line_info.clone()),
    }
}

fn evaluate_engraved_function(
    env: &mut RuntimeEnv,
    evaluated_args: Vec<CallArg>,
    function: EngravedFunction,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let eval_args: Vec<EvalResult> = evaluated_args.into_iter().map(|arg| arg.value).collect();
    let params = function.params.clone();
    env.push_scope();

    if eval_args.len() != params.len() {
        return Err(EvalError::InvalidOperation(
            format!(
                "Function '{}' expected {} arguments but got {}.",
                function.name,
                params.len(),
                eval_args.len()
            ),
            line_info.clone(),
        ));
    }

    for (evaluated_arg, param) in eval_args.into_iter().zip(params.iter()) {
        let (param_name, param_type, is_morph_param) = match param {
            AST::EngraveParam {
                name,
                param_type,
                is_morph,
                ..
            } => (name, param_type, *is_morph),
            _ => {
                return Err(EvalError::InvalidOperation(
                    format!(
                        "Expected EngraveParam in function definition: {}",
                        function.name
                    ),
                    line_info.clone(),
                ));
            }
        };
        let value = if is_morph_param {
            convert_morph_param_value(evaluated_arg, param_type, line_info)?
        } else {
            convert_to_typed_value(evaluated_arg, param_type, line_info)?
        };
        env.set_var(
            param_name.to_string(),
            value,
            param_type.clone(),
            is_morph_param,
            line_info.clone(),
        );
    }

    let result = {
        let evaluated = statements::evaluate(&function.body, env);
        env.pop_scope();
        evaluated?
    };

    let value = eval_result_to_value_checked(result, function.line_info.clone())?;

    match (function.return_type.clone(), value) {
        (Type::Arcana, Value::Arcana(v)) => Ok(EvalResult::data(Value::Arcana(v))),
        (Type::Aether, Value::Aether(v)) => Ok(EvalResult::data(Value::Aether(v))),
        (Type::Rune, Value::Rune(v)) => Ok(EvalResult::data(Value::Rune(v))),
        (Type::Omen, Value::Omen(v)) => Ok(EvalResult::data(Value::Omen(v))),
        (Type::Abyss, Value::Abyss) => Ok(EvalResult::data(Value::Abyss)),
        (Type::Scroll, Value::Scroll(values)) => Ok(EvalResult::data(Value::Scroll(values))),
        (Type::Lexicon, Value::Lexicon(entries)) => Ok(EvalResult::data(Value::Lexicon(entries))),
        (Type::Materia, value) => Ok(EvalResult::data(value)),
        (Type::Artifact(expected), Value::Artifact(handle)) => {
            let type_name = handle.borrow().type_name.clone();
            if type_name == expected {
                Ok(EvalResult::Artifact(handle))
            } else {
                Err(EvalError::TypeError(
                    format!(
                        "Type mismatch for return value of function {} (expected artifact {}, got {})",
                        function.name, expected, type_name
                    ),
                    function.line_info.clone(),
                ))
            }
        }
        (expected, actual) => Err(EvalError::TypeError(
            format!(
                "Type mismatch for return value of function {} (expected {:?}, got {:?})",
                function.name,
                expected,
                describe_value(&actual)
            ),
            function.line_info.clone(),
        )),
    }
}

fn validate_artifact_type(
    handle: Rc<RefCell<crate::env::ArtifactValue>>,
    expected: &str,
    line_info: &Option<LineInfo>,
) -> Result<Value, EvalError> {
    let actual = handle.borrow().type_name.clone();
    if actual == expected {
        Ok(Value::Artifact(handle))
    } else {
        Err(EvalError::TypeError(
            format!(
                "Expected artifact of type {} but received {}",
                expected, actual
            ),
            line_info.clone(),
        ))
    }
}

fn convert_morph_param_value(
    evaluated_arg: EvalResult,
    param_type: &Type,
    line_info: &Option<LineInfo>,
) -> Result<Value, EvalError> {
    match param_type {
        Type::Artifact(expected) => match evaluated_arg {
            EvalResult::Artifact(handle) | EvalResult::Data(Value::Artifact(handle)) => {
                validate_artifact_type(handle, expected, line_info)
            }
            other => convert_to_typed_value(other, param_type, line_info),
        },
        _ => Err(EvalError::InvalidOperation(
            "`morph` parameters are only supported for artifact receivers".to_string(),
            line_info.clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_line_info() -> Option<LineInfo> {
        None
    }

    #[test]
    fn test_pow_arcana_negative_exponent() {
        let mut env = RuntimeEnv::new();
        let left = AST::Arcana(2, dummy_line_info());
        let right = AST::Arcana(-1, dummy_line_info());
        let expr = AST::PowArcana(Box::new(left), Box::new(right), dummy_line_info());

        let result = try_evaluate_expression(&expr, &mut env);
        assert!(matches!(result, Err(EvalError::NegativeExponent(_))));
    }

    #[test]
    fn test_pow_aether_invalid_types() {
        let mut env = RuntimeEnv::new();
        let left = AST::Arcana(2, dummy_line_info()); // Should be Aether
        let right = AST::Aether(2.0, dummy_line_info());
        let expr = AST::PowAether(Box::new(left), Box::new(right), dummy_line_info());

        let result = try_evaluate_expression(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("requires two Aether")
        ));
    }

    #[test]
    fn test_logical_not_invalid_type() {
        let mut env = RuntimeEnv::new();
        let operand = AST::Arcana(1, dummy_line_info()); // Should be Omen
        let expr = AST::LogicalNot(Box::new(operand), dummy_line_info());

        let result = try_evaluate_expression(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("requires Omen")
        ));
    }

    #[test]
    fn test_index_access_out_of_bounds() {
        let mut env = RuntimeEnv::new();
        let scroll = AST::ListLiteral {
            elements: vec![],
            line_info: dummy_line_info(),
        };
        let index = AST::Arcana(0, dummy_line_info());
        let expr = AST::IndexAccess {
            target: Box::new(scroll),
            index: Box::new(index),
            line_info: dummy_line_info(),
        };

        let result = try_evaluate_expression(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("out of bounds")
        ));
    }

    #[test]
    fn test_index_access_invalid_target() {
        let mut env = RuntimeEnv::new();
        let target = AST::Arcana(1, dummy_line_info()); // Not a collection
        let index = AST::Arcana(0, dummy_line_info());
        let expr = AST::IndexAccess {
            target: Box::new(target),
            index: Box::new(index),
            line_info: dummy_line_info(),
        };

        let result = try_evaluate_expression(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("only supported for scroll or lexicon")
        ));
    }
}
