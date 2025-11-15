use std::collections::HashMap;

use crate::ast::{AST, LineInfo, Type};
use crate::env::{CallArg, Callable, EngravedFunction, Environment};

use super::collections::{expect_arcana_index, expect_rune_key};
use super::result::{EvalError, EvalResult};
use super::statements;
use super::values::{convert_to_typed_value, value_to_eval_result};

pub(crate) fn try_evaluate_expression(
    ast: &AST,
    env: &mut Environment,
) -> Result<Option<EvalResult>, EvalError> {
    let result = match ast {
        AST::Omen(value, _) => return Ok(Some(EvalResult::Omen(*value))),
        AST::Arcana(value, _) => return Ok(Some(EvalResult::Arcana(*value))),
        AST::Aether(value, _) => return Ok(Some(EvalResult::Aether(*value))),
        AST::Rune(value, _) => return Ok(Some(EvalResult::Rune(value.clone()))),
        AST::Abyss(_) => return Ok(Some(EvalResult::Abyss)),
        AST::ListLiteral { elements, .. } => {
            let mut evaluated = Vec::new();
            for element in elements {
                evaluated.push(statements::evaluate(element, env)?);
            }
            EvalResult::Scroll(evaluated)
        }
        AST::MapLiteral { entries, .. } => {
            let mut map = HashMap::new();
            for (key, expr) in entries {
                map.insert(key.clone(), statements::evaluate(expr, env)?);
            }
            EvalResult::Lexicon(map)
        }
        AST::Add(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| EvalResult::Arcana(l + r),
            |l, r| EvalResult::Aether(l + r),
            Some(|l: String, r: String| EvalResult::Rune(format!("{}{}", l, r))),
        )?,
        AST::Sub(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| EvalResult::Arcana(l - r),
            |l, r| EvalResult::Aether(l - r),
            None,
        )?,
        AST::Mul(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| EvalResult::Arcana(l * r),
            |l, r| EvalResult::Aether(l * r),
            None,
        )?,
        AST::Div(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| EvalResult::Arcana(l / r),
            |l, r| EvalResult::Aether(l / r),
            None,
        )?,
        AST::Mod(left, right, line_info) => binary_numeric_op(
            env,
            left,
            right,
            line_info,
            |l, r| EvalResult::Arcana(l % r),
            |l, r| EvalResult::Aether(l % r),
            None,
        )?,
        AST::PowArcana(left, right, line_info) => match (
            statements::evaluate(left, env)?,
            statements::evaluate(right, env)?,
        ) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => {
                if r < 0 {
                    return Err(EvalError::NegativeExponent(line_info.clone()));
                } else {
                    EvalResult::Arcana(l.pow(r as u32))
                }
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
            (EvalResult::Aether(l), EvalResult::Aether(r)) => EvalResult::Aether(l.powf(r)),
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
                EvalResult::Omen(value) => Ok(EvalResult::Omen(!value)),
                _ => Err(EvalError::InvalidOperation(
                    "LogicalNot operation requires Omen!".to_string(),
                    line_info.clone(),
                )),
            }?
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
        AST::Trans(expr, target_type, line_info) => {
            let value = statements::evaluate(expr, env)?;
            match target_type {
                Type::Arcana => match value {
                    EvalResult::Aether(n) => EvalResult::Arcana(n as i64),
                    EvalResult::Rune(s) => {
                        s.parse::<i64>().map(EvalResult::Arcana).map_err(|_| {
                            EvalError::InvalidOperation(
                                "Failed to convert Rune to Arcana".to_string(),
                                line_info.clone(),
                            )
                        })?
                    }
                    _ => {
                        return Err(EvalError::InvalidOperation(
                            "Invalid cast to Arcana".to_string(),
                            line_info.clone(),
                        ));
                    }
                },
                Type::Aether => match value {
                    EvalResult::Arcana(n) => EvalResult::Aether(n as f64),
                    EvalResult::Rune(s) => {
                        s.parse::<f64>().map(EvalResult::Aether).map_err(|_| {
                            EvalError::InvalidOperation(
                                "Failed to convert Rune to Aether".to_string(),
                                line_info.clone(),
                            )
                        })?
                    }
                    _ => {
                        return Err(EvalError::InvalidOperation(
                            "Invalid cast to Aether".to_string(),
                            line_info.clone(),
                        ));
                    }
                },
                Type::Rune => match value {
                    EvalResult::Arcana(n) => EvalResult::Rune(n.to_string()),
                    EvalResult::Aether(n) => EvalResult::Rune(n.to_string()),
                    _ => {
                        return Err(EvalError::InvalidOperation(
                            "Invalid cast to Rune".to_string(),
                            line_info.clone(),
                        ));
                    }
                },
                Type::Omen => {
                    return Err(EvalError::InvalidOperation(
                        "Casting to Omen is not supported".to_string(),
                        line_info.clone(),
                    ));
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        format!("Unsupported cast to type {:?}", target_type),
                        line_info.clone(),
                    ));
                }
            }
        }
        AST::IndexAccess {
            target,
            index,
            line_info,
        } => {
            let collection = statements::evaluate(target, env)?;
            let idx_value = statements::evaluate(index, env)?;
            match collection {
                EvalResult::Scroll(items) => {
                    let idx = expect_arcana_index(&idx_value, line_info)?;
                    items.get(idx).cloned().ok_or_else(|| {
                        EvalError::InvalidOperation(
                            format!("Index {} is out of bounds for scroll", idx),
                            line_info.clone(),
                        )
                    })?
                }
                EvalResult::Lexicon(entries) => {
                    let key = expect_rune_key(&idx_value, line_info)?;
                    entries.get(&key).cloned().ok_or_else(|| {
                        EvalError::InvalidOperation(
                            format!("Lexicon key '{}' does not exist", key),
                            line_info.clone(),
                        )
                    })?
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
        AST::OracleDontCareItem(_) => EvalResult::Omen(true),
        _ => return Ok(None),
    };

    Ok(Some(result))
}

fn binary_numeric_op<TArc, TAether>(
    env: &mut Environment,
    left: &AST,
    right: &AST,
    line_info: &Option<LineInfo>,
    arcana_op: TArc,
    aether_op: TAether,
    rune_op: Option<fn(String, String) -> EvalResult>,
) -> Result<EvalResult, EvalError>
where
    TArc: FnOnce(i64, i64) -> EvalResult,
    TAether: FnOnce(f64, f64) -> EvalResult,
{
    let left_result = statements::evaluate(left, env)?;
    let right_result = statements::evaluate(right, env)?;

    match (left_result, right_result) {
        (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(arcana_op(l, r)),
        (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(aether_op(l, r)),
        (EvalResult::Rune(l), EvalResult::Rune(r)) if rune_op.is_some() => {
            Ok(rune_op.unwrap()(l, r))
        }
        _ => Err(EvalError::InvalidOperation(
            "Operation requires compatible types".to_string(),
            line_info.clone(),
        )),
    }
}

fn compare_values(
    env: &mut Environment,
    left: &AST,
    right: &AST,
    line_info: &Option<LineInfo>,
    equality: bool,
) -> Result<EvalResult, EvalError> {
    let left_result = statements::evaluate(left, env)?;
    let right_result = statements::evaluate(right, env)?;

    let comparison = match (left_result, right_result) {
        (EvalResult::Arcana(l), EvalResult::Arcana(r)) => l == r,
        (EvalResult::Aether(l), EvalResult::Aether(r)) => (l - r).abs() < f64::EPSILON,
        (EvalResult::Rune(l), EvalResult::Rune(r)) => l == r,
        _ => {
            return Err(EvalError::InvalidOperation(
                "Comparison requires compatible types!".to_string(),
                line_info.clone(),
            ));
        }
    };

    let result = if equality { comparison } else { !comparison };
    Ok(EvalResult::Omen(result))
}

fn order_values<F>(
    env: &mut Environment,
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
        (EvalResult::Arcana(l), EvalResult::Arcana(r)) => {
            Ok(EvalResult::Omen(comparator(l as f64, r as f64)))
        }
        (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Omen(comparator(l, r))),
        _ => Err(EvalError::InvalidOperation(
            "Comparison requires numeric types!".to_string(),
            line_info.clone(),
        )),
    }
}

fn logical_op<F>(
    env: &mut Environment,
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
        (EvalResult::Omen(l), EvalResult::Omen(r)) => Ok(EvalResult::Omen(op(l, r))),
        _ => Err(EvalError::InvalidOperation(
            "Logical operation requires two Omen!".to_string(),
            line_info.clone(),
        )),
    }
}

fn evaluate_function_call(
    env: &mut Environment,
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
    env: &mut Environment,
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
        let (param_name, param_type) = match param {
            AST::EngraveParam {
                name, param_type, ..
            } => (name, param_type),
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
        let value = convert_to_typed_value(evaluated_arg, param_type, line_info)?;
        env.set_var(
            param_name.to_string(),
            value,
            param_type.clone(),
            false,
            line_info.clone(),
        );
    }

    let result = statements::evaluate(&function.body, env)?;
    env.pop_scope();

    match function.return_type {
        Type::Arcana => match result {
            EvalResult::Arcana(n) => Ok(EvalResult::Arcana(n)),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Aether => match result {
            EvalResult::Aether(n) => Ok(EvalResult::Aether(n)),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Rune => match result {
            EvalResult::Rune(s) => Ok(EvalResult::Rune(s)),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Omen => match result {
            EvalResult::Omen(b) => Ok(EvalResult::Omen(b)),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Abyss => match result {
            EvalResult::Abyss => Ok(EvalResult::Abyss),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Scroll => match result {
            EvalResult::Scroll(items) => Ok(EvalResult::Scroll(items)),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Lexicon => match result {
            EvalResult::Lexicon(entries) => Ok(EvalResult::Lexicon(entries)),
            _ => Err(EvalError::TypeError(
                format!(
                    "Type mismatch for return value of function {}",
                    function.name
                ),
                function.line_info.clone(),
            )),
        },
        Type::Materia => Ok(result),
    }
}
