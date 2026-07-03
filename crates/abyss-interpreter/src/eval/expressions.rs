use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::diagnostics::did_you_mean_hint;
use crate::env::{CallArg, Callable, EngravedFunction, RuntimeEnv, Value};
use crate::stdlib::methods;
use abyss_core::ast::{Expr, Span, Type};

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

/// Evaluates an expression in the given environment.
///
/// Returns an [`EvalResult`] rather than a bare [`Value`] because an
/// expression may contain an `oracle` whose arm body triggers control
/// flow (`reveal` / `revolve` / `eject`) that must propagate outward.
pub(crate) fn evaluate_expr(expr: &Expr, env: &mut RuntimeEnv) -> Result<EvalResult, EvalError> {
    let result = match expr {
        Expr::Omen(value, _) => return Ok(EvalResult::data(Value::Omen(*value))),
        Expr::Arcana(value, _) => return Ok(EvalResult::data(Value::Arcana(*value))),
        Expr::Aether(value, _) => return Ok(EvalResult::data(Value::Aether(*value))),
        Expr::Rune(value, _) => {
            return Ok(EvalResult::data(Value::Rune(Rc::new(value.clone()))));
        }
        Expr::Abyss(_) => return Ok(EvalResult::abyss()),
        Expr::ListLiteral {
            elements,
            span: line_info,
        } => {
            let mut evaluated = Vec::with_capacity(elements.len());
            for element in elements {
                evaluated.push(eval_result_to_value_checked(
                    evaluate_expr(element, env)?,
                    *line_info,
                )?);
            }
            EvalResult::data(Value::Scroll(Rc::new(RefCell::new(evaluated))))
        }
        Expr::MapLiteral {
            entries,
            span: line_info,
        } => {
            let mut map = HashMap::new();
            for (key, expr) in entries {
                let value = eval_result_to_value_checked(evaluate_expr(expr, env)?, *line_info)?;
                map.insert(key.clone(), value);
            }
            EvalResult::data(Value::Lexicon(Rc::new(RefCell::new(map))))
        }
        Expr::ArtifactLiteral {
            type_name,
            fields,
            span: line_info,
        } => instantiate_artifact_literal(env, type_name, fields, line_info)?,
        Expr::FieldAccess {
            target,
            field,
            span: line_info,
        } => {
            let value = evaluate_expr(target, env)?;
            let handle = expect_artifact_from_eval(value, line_info)?;
            let field_value = read_artifact_field(env, &handle, field, line_info)?;
            EvalResult::data(field_value)
        }
        Expr::Add(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l + r,
            |l, r| l + r,
            Some(|l: String, r: String| format!("{}{}", l, r)),
        )?,
        Expr::Sub(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l - r,
            |l, r| l - r,
            None,
        )?,
        Expr::Mul(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l * r,
            |l, r| l * r,
            None,
        )?,
        Expr::Div(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l / r,
            |l, r| l / r,
            None,
        )?,
        Expr::Mod(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| l % r,
            |l, r| l % r,
            None,
        )?,
        Expr::PowArcana(left, right, line_info) => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => {
                    if r < 0 {
                        return Err(EvalError::NegativeExponent(*line_info));
                    }
                    EvalResult::data(Value::Arcana(l.pow(r as u32)))
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "PowArcana operation requires two Arcana!".to_string(),
                        *line_info,
                    ));
                }
            }
        }
        Expr::PowAether(left, right, line_info) => {
            match (evaluate_expr(left, env)?, evaluate_expr(right, env)?) {
                (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
                    EvalResult::data(Value::Aether(l.powf(r)))
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "PowAether operation requires two Aether!".to_string(),
                        *line_info,
                    ));
                }
            }
        }
        Expr::Equal(left, right, line_info) => compare_values(env, left, right, line_info, true)?,
        Expr::NotEqual(left, right, line_info) => {
            compare_values(env, left, right, line_info, false)?
        }
        Expr::LessThan(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l < r)?
        }
        Expr::LessThanOrEqual(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l <= r)?
        }
        Expr::GreaterThan(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l > r)?
        }
        Expr::GreaterThanOrEqual(left, right, line_info) => {
            order_values(env, left, right, line_info, |l, r| l >= r)?
        }
        Expr::LogicalAnd(left, right, line_info) => {
            logical_op(env, left, right, line_info, |l, r| l && r)?
        }
        Expr::LogicalOr(left, right, line_info) => {
            logical_op(env, left, right, line_info, |l, r| l || r)?
        }
        Expr::LogicalNot(expr, line_info) => {
            let result = evaluate_expr(expr, env)?;
            match result {
                EvalResult::Data(Value::Omen(value)) => EvalResult::data(Value::Omen(!value)),
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "LogicalNot operation requires Omen!".to_string(),
                        *line_info,
                    ));
                }
            }
        }
        Expr::Var(name, line_info) => match env.get_var(name) {
            Some(var_info) => value_to_eval_result(&var_info.value),
            None => {
                return Err(env.undefined_variable_error(name, *line_info));
            }
        },
        Expr::IndexAccess {
            target,
            index,
            span: line_info,
        } => {
            let collection = evaluate_expr(target, env)?;
            let idx_value = evaluate_expr(index, env)?;
            match collection {
                EvalResult::Data(Value::Scroll(items)) => {
                    let idx = expect_arcana_index(&idx_value, line_info)?;
                    let borrowed = items.borrow();
                    let value = borrowed
                        .get(idx)
                        .cloned()
                        .ok_or(EvalError::ScrollIndexOutOfBounds(idx, *line_info))?;
                    EvalResult::data(value)
                }
                EvalResult::Data(Value::Lexicon(entries)) => {
                    let key = expect_rune_key(&idx_value, line_info)?;
                    let borrowed = entries.borrow();
                    let value = borrowed
                        .get(&key)
                        .cloned()
                        .ok_or_else(|| EvalError::MissingLexiconKey(key.clone(), *line_info))?;
                    EvalResult::data(value)
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "Indexing is only supported for scroll or lexicon".to_string(),
                        *line_info,
                    ));
                }
            }
        }
        Expr::FuncCall {
            name,
            args,
            span: line_info,
        } => evaluate_function_call(env, name, args, line_info)?,
        Expr::MethodCall {
            receiver,
            method,
            args,
            span: line_info,
        } => evaluate_method_call(env, receiver, method, args, line_info)?,
        Expr::Oracle {
            is_match,
            conditionals,
            branches,
            span: line_info,
        } => {
            // Push the oracle's local scope, run the body, and unconditionally
            // pop on the way out — including error paths — so a failing
            // scrutinee, pattern, ward, or body cannot leak a scope back into
            // the REPL. The helper itself uses `?` freely.
            env.push_scope();
            let result =
                super::patterns::evaluate_oracle(*is_match, conditionals, branches, line_info, env);
            env.pop_scope();
            result?
        }
    };

    Ok(result)
}

fn binary_numeric_op<TArc, TAether>(
    env: &mut RuntimeEnv,
    left: &Expr,
    right: &Expr,
    line_info: &Option<Span>,
    arcana_op: TArc,
    aether_op: TAether,
    rune_op: Option<fn(String, String) -> String>,
) -> Result<EvalResult, EvalError>
where
    TArc: FnOnce(i64, i64) -> i64,
    TAether: FnOnce(f64, f64) -> f64,
{
    let left_result = evaluate_expr(left, env)?;
    let right_result = evaluate_expr(right, env)?;

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
            *line_info,
        )),
    }
}

fn instantiate_artifact_literal(
    env: &mut RuntimeEnv,
    type_name: &str,
    fields: &[(String, Expr)],
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let schema = lookup_schema_by_name(env, type_name, line_info)?.clone();
    let mut provided = HashSet::new();
    let mut values = HashMap::new();

    for (field_name, expr) in fields {
        if !provided.insert(field_name.clone()) {
            return Err(EvalError::InvalidOperation(
                format!("Field '{}' is provided multiple times", field_name),
                *line_info,
            ));
        }

        let field_schema = ensure_field_exists(&schema, field_name, line_info)?;
        let evaluated = evaluate_expr(expr, env)?;
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
            *line_info,
        ));
    }

    Ok(EvalResult::artifact(instantiate_artifact_handle(
        type_name,
        field_order,
        values,
    )))
}

fn compare_values(
    env: &mut RuntimeEnv,
    left: &Expr,
    right: &Expr,
    line_info: &Option<Span>,
    equality: bool,
) -> Result<EvalResult, EvalError> {
    let left_result = evaluate_expr(left, env)?;
    let right_result = evaluate_expr(right, env)?;

    let comparison = match (left_result, right_result) {
        (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => l == r,
        (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
            (l - r).abs() < f64::EPSILON
        }
        (EvalResult::Data(Value::Rune(l)), EvalResult::Data(Value::Rune(r))) => l == r,
        (EvalResult::Data(Value::Artifact(left)), EvalResult::Data(Value::Artifact(right))) => {
            compare_artifacts(env, &left, &right, line_info)?
        }
        _ => {
            return Err(EvalError::InvalidOperation(
                "Comparison requires compatible types!".to_string(),
                *line_info,
            ));
        }
    };

    let result = if equality { comparison } else { !comparison };
    Ok(EvalResult::data(Value::Omen(result)))
}

fn order_values<F>(
    env: &mut RuntimeEnv,
    left: &Expr,
    right: &Expr,
    line_info: &Option<Span>,
    comparator: F,
) -> Result<EvalResult, EvalError>
where
    F: FnOnce(f64, f64) -> bool,
{
    let left_result = evaluate_expr(left, env)?;
    let right_result = evaluate_expr(right, env)?;

    match (left_result, right_result) {
        (EvalResult::Data(Value::Arcana(l)), EvalResult::Data(Value::Arcana(r))) => Ok(
            EvalResult::data(Value::Omen(comparator(l as f64, r as f64))),
        ),
        (EvalResult::Data(Value::Aether(l)), EvalResult::Data(Value::Aether(r))) => {
            Ok(EvalResult::data(Value::Omen(comparator(l, r))))
        }
        _ => Err(EvalError::InvalidOperation(
            "Comparison requires numeric types!".to_string(),
            *line_info,
        )),
    }
}

fn logical_op<F>(
    env: &mut RuntimeEnv,
    left: &Expr,
    right: &Expr,
    line_info: &Option<Span>,
    op: F,
) -> Result<EvalResult, EvalError>
where
    F: FnOnce(bool, bool) -> bool,
{
    let left_result = evaluate_expr(left, env)?;
    let right_result = evaluate_expr(right, env)?;

    match (left_result, right_result) {
        (EvalResult::Data(Value::Omen(l)), EvalResult::Data(Value::Omen(r))) => {
            Ok(EvalResult::data(Value::Omen(op(l, r))))
        }
        _ => Err(EvalError::InvalidOperation(
            "Logical operation requires two Omen!".to_string(),
            *line_info,
        )),
    }
}

fn evaluate_method_call(
    env: &mut RuntimeEnv,
    receiver: &Expr,
    method_name: &str,
    args: &[Expr],
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let receiver_result = evaluate_expr(receiver, env)?;
    let receiver_var_name = if let Expr::Var(var_name, _) = receiver {
        Some(var_name.clone())
    } else {
        None
    };

    let mut evaluated_args = Vec::with_capacity(args.len());
    for arg in args {
        let evaluated_arg = evaluate_expr(arg, env)?;
        let var_name = if let Expr::Var(var_name, _) = arg {
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
            *line_info,
        )),
    }
}

fn ensure_method_receiver_mutability(
    env: &RuntimeEnv,
    receiver: &Expr,
    artifact_name: &str,
    method_name: &str,
    line_info: &Option<Span>,
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
                *line_info,
            ));
        } else {
            return Err(env.undefined_variable_error(&base_name, *line_info));
        }
    }

    Err(EvalError::InvalidOperation(
        format!(
            "Method {}::{} requires a morph receiver, but the expression is not tied to a mutable variable",
            artifact_name, method_name
        ),
        *line_info,
    ))
}

fn evaluate_artifact_method_call(
    env: &mut RuntimeEnv,
    receiver_ast: &Expr,
    receiver_var_name: Option<String>,
    receiver_handle: crate::env::ArtifactHandle,
    method_name: &str,
    arg_values: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let schema = lookup_schema_from_handle(env, &receiver_handle, line_info)?;
    let artifact_name = schema.name.clone();
    let artifact_method = env
        .get_artifact_method(&artifact_name, method_name)
        .ok_or_else(|| {
            // Build candidate names lazily on the error path so the happy
            // path does not iterate the schema's method table.
            let hint = did_you_mean_hint(method_name, schema.methods.keys().map(String::as_str), 3)
                .map(|h| format!(" {}", h))
                .unwrap_or_default();
            EvalError::InvalidOperation(
                format!(
                    "Method {}::{} is not defined{}",
                    artifact_name, method_name, hint
                ),
                *line_info,
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
        value: EvalResult::artifact(receiver_handle),
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
    args: &[Expr],
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let callable = match env.get_function(name) {
        Some(func) => func.clone(),
        None => {
            return Err(env.undefined_function_error(name, *line_info));
        }
    };

    let mut evaluated_args = Vec::new();
    for arg in args {
        let evaluated_arg = evaluate_expr(arg, env)?;
        let var_name = if let Expr::Var(var_name, _) = arg {
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
        Callable::Builtin(function) => (function.func)(env, evaluated_args, *line_info),
    }
}

fn evaluate_engraved_function(
    env: &mut RuntimeEnv,
    evaluated_args: Vec<CallArg>,
    function: EngravedFunction,
    line_info: &Option<Span>,
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
            *line_info,
        ));
    }

    for (evaluated_arg, param) in eval_args.into_iter().zip(params.iter()) {
        let (param_name, param_type, is_morph_param) =
            (&param.name, &param.param_type, param.is_morph);
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
            *line_info,
        );
    }

    let result = {
        let evaluated = statements::evaluate(&function.body, env);
        env.pop_scope();
        evaluated?
    };

    let value = eval_result_to_value_checked(result, function.line_info)?;

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
                Ok(EvalResult::artifact(handle))
            } else {
                Err(EvalError::TypeError(
                    format!(
                        "Type mismatch for return value of function {} (expected artifact {}, got {})",
                        function.name, expected, type_name
                    ),
                    function.line_info,
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
            function.line_info,
        )),
    }
}

fn validate_artifact_type(
    handle: Rc<RefCell<crate::env::ArtifactValue>>,
    expected: &str,
    line_info: &Option<Span>,
) -> Result<Value, EvalError> {
    let actual = handle.borrow().type_name.clone();
    if actual == expected {
        Ok(Value::Artifact(handle))
    } else {
        Err(EvalError::ArtifactTypeMismatch {
            expected: expected.to_string(),
            found: actual,
            line_info: *line_info,
        })
    }
}

fn convert_morph_param_value(
    evaluated_arg: EvalResult,
    param_type: &Type,
    line_info: &Option<Span>,
) -> Result<Value, EvalError> {
    match param_type {
        Type::Artifact(expected) => match evaluated_arg {
            EvalResult::Data(Value::Artifact(handle)) => {
                validate_artifact_type(handle, expected, line_info)
            }
            other => convert_to_typed_value(other, param_type, line_info),
        },
        _ => Err(EvalError::InvalidOperation(
            "`morph` parameters are only supported for artifact receivers".to_string(),
            *line_info,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_line_info() -> Option<Span> {
        None
    }

    #[test]
    fn test_pow_arcana_negative_exponent() {
        let mut env = RuntimeEnv::new();
        let left = Expr::Arcana(2, dummy_line_info());
        let right = Expr::Arcana(-1, dummy_line_info());
        let expr = Expr::PowArcana(Box::new(left), Box::new(right), dummy_line_info());

        let result = evaluate_expr(&expr, &mut env);
        assert!(matches!(result, Err(EvalError::NegativeExponent(_))));
    }

    #[test]
    fn test_pow_aether_invalid_types() {
        let mut env = RuntimeEnv::new();
        let left = Expr::Arcana(2, dummy_line_info()); // Should be Aether
        let right = Expr::Aether(2.0, dummy_line_info());
        let expr = Expr::PowAether(Box::new(left), Box::new(right), dummy_line_info());

        let result = evaluate_expr(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("requires two Aether")
        ));
    }

    #[test]
    fn test_logical_not_invalid_type() {
        let mut env = RuntimeEnv::new();
        let operand = Expr::Arcana(1, dummy_line_info()); // Should be Omen
        let expr = Expr::LogicalNot(Box::new(operand), dummy_line_info());

        let result = evaluate_expr(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("requires Omen")
        ));
    }

    #[test]
    fn test_index_access_out_of_bounds() {
        let mut env = RuntimeEnv::new();
        let scroll = Expr::ListLiteral {
            elements: vec![],
            span: dummy_line_info(),
        };
        let index = Expr::Arcana(0, dummy_line_info());
        let expr = Expr::IndexAccess {
            target: Box::new(scroll),
            index: Box::new(index),
            span: dummy_line_info(),
        };

        let result = evaluate_expr(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::ScrollIndexOutOfBounds(_, _))
        ));
    }

    #[test]
    fn test_index_access_missing_lexicon_key() {
        let mut env = RuntimeEnv::new();
        let mut entries = HashMap::new();
        entries.insert("known".to_string(), Value::Arcana(1));
        env.set_var(
            "lex".to_string(),
            Value::Lexicon(Rc::new(RefCell::new(entries))),
            Type::Lexicon,
            false,
            None,
        );

        let expr = Expr::IndexAccess {
            target: Box::new(Expr::Var("lex".to_string(), dummy_line_info())),
            index: Box::new(Expr::Rune("missing".to_string(), dummy_line_info())),
            span: dummy_line_info(),
        };

        let result = evaluate_expr(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::MissingLexiconKey(key, _)) if key == "missing"
        ));
    }

    #[test]
    fn test_index_access_invalid_target() {
        let mut env = RuntimeEnv::new();
        let target = Expr::Arcana(1, dummy_line_info()); // Not a collection
        let index = Expr::Arcana(0, dummy_line_info());
        let expr = Expr::IndexAccess {
            target: Box::new(target),
            index: Box::new(index),
            span: dummy_line_info(),
        };

        let result = evaluate_expr(&expr, &mut env);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("only supported for scroll or lexicon")
        ));
    }
}
