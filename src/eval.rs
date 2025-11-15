use crate::ast::{AST, AssignmentOp, ConditionalAssignment, LineInfo, Type};
use crate::env::{CallArg, Callable, EngravedFunction, Environment, Value};
use colored::*;
use std::collections::HashMap;
use std::fmt;

/// Represents the result of an evaluation in the interpreter.
#[derive(Debug, Clone)]
pub enum EvalResult {
    Omen(bool),
    Arcana(i64),
    Aether(f64),
    Rune(String),
    Abyss,
    Scroll(Vec<EvalResult>),
    Lexicon(HashMap<String, EvalResult>),
    Revealed(Box<EvalResult>),
    Resume(Option<String>),
    Eject(Option<String>),
}

/// Represents possible errors that can occur during evaluation.
#[derive(Debug)]
pub enum EvalError {
    UndefinedVariable(String, Option<LineInfo>),
    InvalidOperation(String, Option<LineInfo>),
    NegativeExponent(Option<LineInfo>),
    TypeError(String, Option<LineInfo>),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UndefinedVariable(var, _) => write!(f, "Variable {} is not defined!", var),
            EvalError::InvalidOperation(op, _) => write!(f, "Invalid operation: {}", op),
            EvalError::NegativeExponent(_) => {
                write!(f, "PowArcana operation requires a non-negative exponent!")
            }
            EvalError::TypeError(var_type, _) => write!(f, "Type error: {}", var_type),
        }
    }
}
impl std::error::Error for EvalError {}

/// Displays an error message along with the relevant source code and line information, if available.
pub fn display_error_with_source(script: &str, line_info: Option<LineInfo>, error_message: &str) {
    if let Some(info) = line_info {
        let lines: Vec<&str> = script.lines().collect();
        if let Some(source_line) = lines.get(info.line - 1) {
            // Line numbers start from 1, so we subtract 1
            eprintln!(
                "{}",
                format!(
                    "Error at line {}, column {}: {}",
                    info.line, info.column, error_message
                )
                .red()
            );
            eprintln!("  {}", source_line.red());
            eprintln!("  {}{}", " ".repeat(info.column - 1).red(), "^".red());
        } else {
            eprintln!("{}", format!("Error: {}", error_message).red());
        }
    } else {
        eprintln!("{}", format!("Error: {}", error_message).red());
    }
}

fn value_to_eval_result(value: &Value) -> EvalResult {
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

fn eval_result_to_value_any(result: EvalResult) -> Result<Value, EvalError> {
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

fn eval_result_to_value_checked(
    result: EvalResult,
    line_info: Option<LineInfo>,
) -> Result<Value, EvalError> {
    eval_result_to_value_any(result).map_err(|err| match err {
        EvalError::InvalidOperation(msg, _) => EvalError::InvalidOperation(msg, line_info.clone()),
        EvalError::TypeError(msg, _) => EvalError::TypeError(msg, line_info.clone()),
        other => other,
    })
}

fn convert_to_typed_value(
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

/// Extract an Arcana value from an EvalResult for use in compound assignments
fn extract_arcana(result: &EvalResult, line_info: &Option<LineInfo>) -> Result<i64, EvalError> {
    match result {
        EvalResult::Arcana(v) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected arcana value".to_string(),
            line_info.clone(),
        )),
    }
}

/// Extract an Aether value from an EvalResult for use in compound assignments
fn extract_aether(result: &EvalResult, line_info: &Option<LineInfo>) -> Result<f64, EvalError> {
    match result {
        EvalResult::Aether(v) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected aether value".to_string(),
            line_info.clone(),
        )),
    }
}

/// Extract a Rune value from an EvalResult for use in compound assignments
fn extract_rune(result: EvalResult, line_info: &Option<LineInfo>) -> Result<String, EvalError> {
    match result {
        EvalResult::Rune(v) => Ok(v),
        _ => Err(EvalError::TypeError(
            "Expected rune value".to_string(),
            line_info.clone(),
        )),
    }
}

/// Extract an Omen value from an EvalResult for use in compound assignments
fn extract_omen(result: &EvalResult, line_info: &Option<LineInfo>) -> Result<bool, EvalError> {
    match result {
        EvalResult::Omen(v) => Ok(*v),
        _ => Err(EvalError::TypeError(
            "Expected omen value".to_string(),
            line_info.clone(),
        )),
    }
}

fn expect_arcana_index(
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

fn expect_rune_key(index: &EvalResult, line_info: &Option<LineInfo>) -> Result<String, EvalError> {
    if let EvalResult::Rune(value) = index {
        Ok(value.clone())
    } else {
        Err(EvalError::TypeError(
            "Lexicon key must be rune".to_string(),
            line_info.clone(),
        ))
    }
}

fn collect_index_chain(target: &AST) -> Option<(String, Vec<&AST>)> {
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

fn resolve_nested_value_mut<'a>(
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

/// Evaluates an abstract syntax tree (AST) node in the given environment.
///
/// # Arguments
///
/// * `ast` - The AST node to be evaluated.
/// * `env` - The environment containing variable and function bindings.
///
/// # Returns
///
/// The result of the evaluation, or an `EvalError` if an error occurs.
pub fn evaluate(ast: &AST, env: &mut Environment) -> Result<EvalResult, EvalError> {
    match ast {
        AST::Statement(node, _line_info) => evaluate(node, env),
        AST::Omen(b, _line_info) => Ok(EvalResult::Omen(*b)),
        AST::Arcana(n, _line_info) => Ok(EvalResult::Arcana(*n)),
        AST::Aether(n, _line_info) => Ok(EvalResult::Aether(*n)),
        AST::Rune(s, _line_info) => Ok(EvalResult::Rune(s.clone())),
        AST::Abyss(_line_info) => Ok(EvalResult::Abyss),
        AST::ListLiteral { elements, .. } => {
            let mut evaluated = Vec::new();
            for element in elements {
                evaluated.push(evaluate(element, env)?);
            }
            Ok(EvalResult::Scroll(evaluated))
        }
        AST::MapLiteral { entries, .. } => {
            let mut map = HashMap::new();
            for (key, expr) in entries {
                map.insert(key.clone(), evaluate(expr, env)?);
            }
            Ok(EvalResult::Lexicon(map))
        }
        AST::Add(left, right, line_info) => match (evaluate(left, env)?, evaluate(right, env)?) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Arcana(l + r)),
            (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Aether(l + r)),
            (EvalResult::Rune(l), EvalResult::Rune(r)) => {
                Ok(EvalResult::Rune(format!("{}{}", l, r)))
            }
            _ => Err(EvalError::InvalidOperation(
                "Add operation requires either two Arcana, two Aether, or two Rune!".to_string(),
                line_info.clone(),
            )),
        },
        AST::Sub(left, right, line_info) => match (evaluate(left, env)?, evaluate(right, env)?) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Arcana(l - r)),
            (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Aether(l - r)),
            _ => Err(EvalError::InvalidOperation(
                "Subtract operation requires either two Arcana or two Aether!".to_string(),
                line_info.clone(),
            )),
        },
        AST::Mul(left, right, line_info) => match (evaluate(left, env)?, evaluate(right, env)?) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Arcana(l * r)),
            (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Aether(l * r)),
            _ => Err(EvalError::InvalidOperation(
                "Multiply operation requires either two Arcana or two Aether!".to_string(),
                line_info.clone(),
            )),
        },
        AST::Div(left, right, line_info) => match (evaluate(left, env)?, evaluate(right, env)?) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Arcana(l / r)),
            (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Aether(l / r)),
            _ => Err(EvalError::InvalidOperation(
                "Divide operation requires either two Arcana or two Aether!".to_string(),
                line_info.clone(),
            )),
        },
        AST::Mod(left, right, line_info) => match (evaluate(left, env)?, evaluate(right, env)?) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Arcana(l % r)),
            (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Aether(l % r)),
            _ => Err(EvalError::InvalidOperation(
                "Modulo operation requires either two Arcana or two Aether!".to_string(),
                line_info.clone(),
            )),
        },
        AST::PowArcana(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Arcana(l), EvalResult::Arcana(r)) => {
                    if r < 0 {
                        return Err(EvalError::NegativeExponent(line_info.clone()));
                    }
                    Ok(EvalResult::Arcana(l.pow(r as u32)))
                }
                _ => Err(EvalError::InvalidOperation(
                    "PowArcana operation requires two Arcana!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::PowAether(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Aether(l.powf(r))),
                _ => Err(EvalError::InvalidOperation(
                    "PowAether operation requires two Aether!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::Equal(left, right, line_info) => match (evaluate(left, env)?, evaluate(right, env)?) {
            (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Omen(l == r)),
            (EvalResult::Aether(l), EvalResult::Aether(r)) => {
                Ok(EvalResult::Omen((l - r).abs() < f64::EPSILON))
            }
            (EvalResult::Rune(l), EvalResult::Rune(r)) => Ok(EvalResult::Omen(l == r)),
            _ => Err(EvalError::InvalidOperation(
                "Comparison requires compatible types!".to_string(),
                line_info.clone(),
            )),
        },
        AST::NotEqual(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Omen(l != r)),
                (EvalResult::Aether(l), EvalResult::Aether(r)) => {
                    Ok(EvalResult::Omen((l - r).abs() >= f64::EPSILON))
                }
                (EvalResult::Rune(l), EvalResult::Rune(r)) => Ok(EvalResult::Omen(l != r)),
                _ => Err(EvalError::InvalidOperation(
                    "Comparison requires compatible types!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::LessThan(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Omen(l < r)),
                (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Omen(l < r)),
                _ => Err(EvalError::InvalidOperation(
                    "Comparison requires numeric types!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::LessThanOrEqual(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Omen(l <= r)),
                (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Omen(l <= r)),
                _ => Err(EvalError::InvalidOperation(
                    "Comparison requires numeric types!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::GreaterThan(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Omen(l > r)),
                (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Omen(l > r)),
                _ => Err(EvalError::InvalidOperation(
                    "Comparison requires numeric types!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::GreaterThanOrEqual(left, right, line_info) => {
            match (evaluate(left, env)?, evaluate(right, env)?) {
                (EvalResult::Arcana(l), EvalResult::Arcana(r)) => Ok(EvalResult::Omen(l >= r)),
                (EvalResult::Aether(l), EvalResult::Aether(r)) => Ok(EvalResult::Omen(l >= r)),
                _ => Err(EvalError::InvalidOperation(
                    "Comparison requires numeric types!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::LogicalAnd(left, right, line_info) => {
            let left_result = evaluate(left, env)?;
            let right_result = evaluate(right, env)?;
            match (left_result, right_result) {
                (EvalResult::Omen(l), EvalResult::Omen(r)) => Ok(EvalResult::Omen(l && r)),
                _ => Err(EvalError::InvalidOperation(
                    "LogicalAnd operation requires two Omen!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::LogicalOr(left, right, line_info) => {
            let left_result = evaluate(left, env)?;
            let right_result = evaluate(right, env)?;
            match (left_result, right_result) {
                (EvalResult::Omen(l), EvalResult::Omen(r)) => Ok(EvalResult::Omen(l || r)),
                _ => Err(EvalError::InvalidOperation(
                    "LogicalOr operation requires two Omen!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::LogicalNot(expr, line_info) => {
            let result = evaluate(expr, env)?;
            match result {
                EvalResult::Omen(value) => Ok(EvalResult::Omen(!value)),
                _ => Err(EvalError::InvalidOperation(
                    "LogicalNot operation requires Omen!".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::VarAssign {
            name,
            value,
            var_type,
            is_morph,
            line_info,
        } => {
            let evaluated_value = evaluate(value, env)?;
            let stored_value = convert_to_typed_value(evaluated_value, var_type, line_info)?;
            env.set_var(
                name.clone(),
                stored_value,
                var_type.clone(),
                *is_morph,
                line_info.clone(),
            );
            Ok(EvalResult::Abyss)
        }
        AST::Assignment {
            name,
            value,
            op,
            line_info,
        } => {
            let evaluated_value = evaluate(value, env)?;
            if let Some(var_info) = env.get_var_mut(name) {
                if !var_info.is_morph {
                    return Err(EvalError::InvalidOperation(
                        format!("Cannot reassign to immutable variable {}", name),
                        line_info.clone(),
                    ));
                }

                match (&mut var_info.value, &var_info.var_type) {
                    (Value::Arcana(current), Type::Arcana) => {
                        let new_value = match op {
                            AssignmentOp::AddAssign => {
                                *current + extract_arcana(&evaluated_value, line_info)?
                            }
                            AssignmentOp::SubAssign => {
                                *current - extract_arcana(&evaluated_value, line_info)?
                            }
                            AssignmentOp::MulAssign => {
                                *current * extract_arcana(&evaluated_value, line_info)?
                            }
                            AssignmentOp::DivAssign => {
                                *current / extract_arcana(&evaluated_value, line_info)?
                            }
                            AssignmentOp::ModAssign => {
                                *current % extract_arcana(&evaluated_value, line_info)?
                            }
                            AssignmentOp::PowArcanaAssign => {
                                let exponent = extract_arcana(&evaluated_value, line_info)?;
                                if exponent < 0 {
                                    return Err(EvalError::NegativeExponent(line_info.clone()));
                                }
                                current.pow(exponent as u32)
                            }
                            AssignmentOp::Assign => extract_arcana(&evaluated_value, line_info)?,
                            _ => {
                                return Err(EvalError::InvalidOperation(
                                    format!("Unsupported operation for variable {}", name),
                                    line_info.clone(),
                                ));
                            }
                        };
                        *current = new_value;
                    }
                    (Value::Aether(current), Type::Aether) => {
                        let operand = extract_aether(&evaluated_value, line_info)?;
                        let new_value = match op {
                            AssignmentOp::AddAssign => *current + operand,
                            AssignmentOp::SubAssign => *current - operand,
                            AssignmentOp::MulAssign => *current * operand,
                            AssignmentOp::DivAssign => *current / operand,
                            AssignmentOp::ModAssign => *current % operand,
                            AssignmentOp::PowAetherAssign => current.powf(operand),
                            AssignmentOp::Assign => operand,
                            _ => {
                                return Err(EvalError::InvalidOperation(
                                    format!("Unsupported operation for variable {}", name),
                                    line_info.clone(),
                                ));
                            }
                        };
                        *current = new_value;
                    }
                    (Value::Rune(current), Type::Rune) => match op {
                        AssignmentOp::AddAssign => {
                            let rhs = extract_rune(evaluated_value, line_info)?;
                            current.push_str(&rhs);
                        }
                        AssignmentOp::Assign => {
                            *current = extract_rune(evaluated_value, line_info)?;
                        }
                        _ => {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported operation for variable {}", name),
                                line_info.clone(),
                            ));
                        }
                    },
                    (Value::Omen(current), Type::Omen) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported operation for variable {}", name),
                                line_info.clone(),
                            ));
                        }
                        *current = extract_omen(&evaluated_value, line_info)?;
                    }
                    (Value::Abyss, Type::Abyss) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported operation for variable {}", name),
                                line_info.clone(),
                            ));
                        }
                        if !matches!(evaluated_value, EvalResult::Abyss) {
                            return Err(EvalError::TypeError(
                                "Expected abyss value".to_string(),
                                line_info.clone(),
                            ));
                        }
                    }
                    (value_slot, Type::Scroll) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                "Scroll reassignment only supports =".to_string(),
                                line_info.clone(),
                            ));
                        }
                        *value_slot =
                            convert_to_typed_value(evaluated_value, &Type::Scroll, line_info)?;
                    }
                    (value_slot, Type::Lexicon) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                "Lexicon reassignment only supports =".to_string(),
                                line_info.clone(),
                            ));
                        }
                        *value_slot =
                            convert_to_typed_value(evaluated_value, &Type::Lexicon, line_info)?;
                    }
                    (value_slot, Type::Materia) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                "Materia variables only support =".to_string(),
                                line_info.clone(),
                            ));
                        }
                        *value_slot =
                            eval_result_to_value_checked(evaluated_value, line_info.clone())?;
                    }
                    _ => {
                        return Err(EvalError::InvalidOperation(
                            format!(
                                "Type mismatch or unsupported operation for variable {}",
                                name
                            ),
                            line_info.clone(),
                        ));
                    }
                }

                Ok(EvalResult::Abyss)
            } else {
                Err(EvalError::UndefinedVariable(
                    name.clone(),
                    line_info.clone(),
                ))
            }
        }
        AST::IndexAccess {
            target,
            index,
            line_info,
        } => {
            let collection = evaluate(target, env)?;
            let idx_value = evaluate(index, env)?;
            match collection {
                EvalResult::Scroll(items) => {
                    let idx = expect_arcana_index(&idx_value, line_info)?;
                    items.get(idx).cloned().ok_or_else(|| {
                        EvalError::InvalidOperation(
                            format!("Index {} is out of bounds for scroll", idx),
                            line_info.clone(),
                        )
                    })
                }
                EvalResult::Lexicon(entries) => {
                    let key = expect_rune_key(&idx_value, line_info)?;
                    entries.get(&key).cloned().ok_or_else(|| {
                        EvalError::InvalidOperation(
                            format!("Lexicon key '{}' does not exist", key),
                            line_info.clone(),
                        )
                    })
                }
                _ => Err(EvalError::InvalidOperation(
                    "Indexing is only supported for scroll or lexicon".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::IndexAssignment {
            target,
            index,
            value,
            line_info,
        } => {
            let (base_name, nested_indices) = collect_index_chain(target).ok_or_else(|| {
                EvalError::InvalidOperation(
                    "Indexed assignment requires a mutable variable target".to_string(),
                    line_info.clone(),
                )
            })?;

            let mut evaluated_indices = Vec::new();
            for idx_ast in nested_indices {
                evaluated_indices.push(evaluate(idx_ast, env)?);
            }

            let final_index_value = evaluate(index, env)?;
            let new_value = eval_result_to_value_checked(evaluate(value, env)?, line_info.clone())?;

            let var_info = env.get_var_mut(&base_name).ok_or_else(|| {
                EvalError::UndefinedVariable(base_name.clone(), line_info.clone())
            })?;

            if !var_info.is_morph {
                return Err(EvalError::InvalidOperation(
                    format!("Cannot reassign to immutable variable {}", base_name),
                    line_info.clone(),
                ));
            }

            let mut current_value = &mut var_info.value;
            for idx in &evaluated_indices {
                current_value = resolve_nested_value_mut(current_value, idx, line_info)?;
            }

            match current_value {
                Value::Scroll(items) => {
                    let idx = expect_arcana_index(&final_index_value, line_info)?;
                    if idx >= items.len() {
                        return Err(EvalError::InvalidOperation(
                            format!("Index {} is out of bounds for scroll", idx),
                            line_info.clone(),
                        ));
                    }
                    items[idx] = new_value;
                }
                Value::Lexicon(entries) => {
                    let key = expect_rune_key(&final_index_value, line_info)?;
                    entries.insert(key, new_value);
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "Indexed assignment requires a scroll or lexicon".to_string(),
                        line_info.clone(),
                    ));
                }
            }

            Ok(EvalResult::Abyss)
        }
        AST::Var(name, line_info) => match env.get_var(name) {
            Some(var_info) => Ok(value_to_eval_result(&var_info.value)),
            None => Err(EvalError::UndefinedVariable(
                name.clone(),
                line_info.clone(),
            )),
        },
        AST::Trans(expr, target_type, line_info) => {
            let value = evaluate(expr, env)?;
            match target_type {
                Type::Arcana => match value {
                    EvalResult::Aether(n) => Ok(EvalResult::Arcana(n as i64)),
                    EvalResult::Rune(s) => s.parse::<i64>().map(EvalResult::Arcana).map_err(|_| {
                        EvalError::InvalidOperation(
                            "Failed to convert Rune to Arcana".to_string(),
                            line_info.clone(),
                        )
                    }),
                    _ => Err(EvalError::InvalidOperation(
                        "Invalid cast to Arcana".to_string(),
                        line_info.clone(),
                    )),
                },
                Type::Aether => match value {
                    EvalResult::Arcana(n) => Ok(EvalResult::Aether(n as f64)),
                    EvalResult::Rune(s) => s.parse::<f64>().map(EvalResult::Aether).map_err(|_| {
                        EvalError::InvalidOperation(
                            "Failed to convert Rune to Aether".to_string(),
                            line_info.clone(),
                        )
                    }),
                    _ => Err(EvalError::InvalidOperation(
                        "Invalid cast to Aether".to_string(),
                        line_info.clone(),
                    )),
                },
                Type::Rune => match value {
                    EvalResult::Arcana(n) => Ok(EvalResult::Rune(n.to_string())),
                    EvalResult::Aether(n) => Ok(EvalResult::Rune(n.to_string())),
                    _ => Err(EvalError::InvalidOperation(
                        "Invalid cast to Rune".to_string(),
                        line_info.clone(),
                    )),
                },
                Type::Omen => Err(EvalError::InvalidOperation(
                    "Casting to Omen is not supported".to_string(),
                    line_info.clone(),
                )),
                _ => Err(EvalError::InvalidOperation(
                    format!("Unsupported cast to type {:?}", target_type),
                    line_info.clone(),
                )),
            }
        }
        AST::Oracle {
            is_match,
            conditionals,
            branches,
            line_info,
        } => {
            env.push_scope();

            let mut evaluate_and_set_var =
                |conditional: &ConditionalAssignment| -> Result<(), EvalError> {
                    let result = evaluate(&conditional.expression, env)?;
                    match result {
                        EvalResult::Arcana(n) => env.set_var(
                            conditional.variable.clone(),
                            Value::Arcana(n),
                            Type::Arcana,
                            false,
                            line_info.clone(),
                        ),
                        EvalResult::Aether(n) => env.set_var(
                            conditional.variable.clone(),
                            Value::Aether(n),
                            Type::Aether,
                            false,
                            line_info.clone(),
                        ),
                        EvalResult::Rune(ref s) => env.set_var(
                            conditional.variable.clone(),
                            Value::Rune(s.clone()),
                            Type::Rune,
                            false,
                            line_info.clone(),
                        ),
                        EvalResult::Omen(b) => env.set_var(
                            conditional.variable.clone(),
                            Value::Omen(b),
                            Type::Omen,
                            false,
                            line_info.clone(),
                        ),
                        _ => {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported type in oracle conditional: {:?}", result),
                                line_info.clone(),
                            ));
                        }
                    }
                    Ok(())
                };

            for conditional in conditionals {
                evaluate_and_set_var(conditional)?;
            }

            for branch in branches {
                if let AST::Comment(_, _) = branch {
                    continue;
                }

                if let AST::OracleBranch {
                    pattern,
                    body,
                    line_info,
                } = branch
                {
                    let matched = if pattern.is_empty() {
                        true
                    } else if *is_match {
                        let mut matched = true;
                        for (idx, pattern) in pattern.iter().enumerate() {
                            if let AST::OracleDontCareItem(_) = pattern {
                                continue;
                            }
                            let pattern_result = evaluate(pattern, env)?;
                            let conditional_result = evaluate(&conditionals[idx].expression, env)?;

                            match (conditional_result, pattern_result) {
                                (EvalResult::Arcana(cond_n), EvalResult::Arcana(pat_n)) => {
                                    if cond_n != pat_n {
                                        matched = false;
                                        break;
                                    }
                                }
                                (EvalResult::Aether(cond_n), EvalResult::Aether(pat_n)) => {
                                    if (cond_n - pat_n).abs() >= f64::EPSILON {
                                        matched = false;
                                        break;
                                    }
                                }
                                (EvalResult::Rune(cond_s), EvalResult::Rune(pat_s)) => {
                                    if cond_s != pat_s {
                                        matched = false;
                                        break;
                                    }
                                }
                                (EvalResult::Omen(cond_b), EvalResult::Omen(pat_b)) => {
                                    if cond_b != pat_b {
                                        matched = false;
                                        break;
                                    }
                                }
                                _ => {
                                    return Err(EvalError::InvalidOperation(
                                        "Oracle branch pattern type must match conditional type"
                                            .to_string(),
                                        line_info.clone(),
                                    ));
                                }
                            }
                        }
                        matched
                    } else {
                        pattern.iter().all(|pattern| {
                            matches!(evaluate(pattern, env), Ok(EvalResult::Omen(true)))
                        })
                    };

                    if matched {
                        let result = match evaluate(body.as_ref(), env) {
                            Ok(result) => match result {
                                EvalResult::Revealed(revealed) => *revealed,
                                _ => result,
                            },
                            Err(e) => return Err(e),
                        };
                        env.pop_scope();
                        return Ok(result);
                    }
                }
            }

            env.pop_scope();
            Ok(EvalResult::Abyss)
        }
        AST::Reveal(expr, _line_info) => {
            let result = evaluate(expr, env)?;
            Ok(EvalResult::Revealed(Box::new(result)))
        }
        AST::Block(statements, _line_info) => {
            let mut last_result = EvalResult::Abyss;
            for statement in statements {
                let result = evaluate(statement, env)?;

                match result {
                    EvalResult::Revealed(revealed) => return Ok(*revealed),
                    EvalResult::Resume(_) | EvalResult::Eject(_) => return Ok(result),
                    _ => {}
                }

                last_result = result;
            }
            Ok(last_result)
        }
        AST::OracleDontCareItem(_line_info) => Ok(EvalResult::Omen(true)),
        AST::Orbit {
            params,
            body,
            line_info,
        } => {
            if params.is_empty() {
                loop {
                    env.push_scope();

                    let result = evaluate(body, env)?;

                    match result {
                        EvalResult::Resume(_) => continue,
                        EvalResult::Eject(_) => break,
                        _ => {}
                    }

                    env.pop_scope();
                }

                Ok(EvalResult::Abyss)
            } else if let AST::OrbitParam {
                name,
                start,
                end,
                op,
                ..
            } = &params[0]
            {
                let start_value = evaluate(start, env)?;
                let end_value = evaluate(end, env)?;

                if let (EvalResult::Arcana(start_num), EvalResult::Arcana(end_num)) =
                    (start_value, end_value)
                {
                    let range = start_num..end_num + if op == ".." { 0 } else { 1 };

                    for value in range {
                        env.push_scope();

                        env.set_var(
                            name.clone(),
                            Value::Arcana(value),
                            Type::Arcana,
                            true,
                            line_info.clone(),
                        );

                        let remaining_params = params[1..].to_vec();
                        let result = if remaining_params.is_empty() {
                            evaluate(body.as_ref(), env)?
                        } else {
                            evaluate(
                                &AST::Orbit {
                                    params: remaining_params,
                                    body: body.clone(),
                                    line_info: line_info.clone(),
                                },
                                env,
                            )?
                        };

                        match result {
                            EvalResult::Resume(identifier) => {
                                if let Some(id) = identifier {
                                    if id == *name {
                                        continue;
                                    } else {
                                        env.pop_scope();
                                        return Ok(EvalResult::Resume(Some(id)));
                                    }
                                }
                                continue;
                            }
                            EvalResult::Eject(identifier) => {
                                if let Some(id) = identifier {
                                    if id == *name {
                                        break;
                                    } else {
                                        env.pop_scope();
                                        return Ok(EvalResult::Eject(Some(id)));
                                    }
                                }
                                break;
                            }
                            _ => {}
                        }

                        env.pop_scope();
                    }
                    Ok(EvalResult::Abyss)
                } else {
                    Err(EvalError::TypeError(
                        format!("Orbit parameter must be of type Arcana: {}", name),
                        line_info.clone(),
                    ))
                }
            } else {
                Err(EvalError::InvalidOperation(
                    "Expected OrbitParam in Orbit".to_string(),
                    line_info.clone(),
                ))
            }
        }
        AST::Resume(identifier, _line_info) => Ok(EvalResult::Resume(identifier.clone())),
        AST::Eject(identifier, _line_info) => Ok(EvalResult::Eject(identifier.clone())),
        AST::Engrave {
            name,
            params,
            return_type,
            body,
            line_info,
        } => {
            let function = EngravedFunction {
                name: name.clone(),
                params: params.clone(),
                return_type: return_type.clone(),
                body: body.clone(),
                line_info: line_info.clone(),
            };
            env.set_function(name.clone(), Callable::Engraved(function));
            Ok(EvalResult::Abyss)
        }
        AST::FuncCall {
            name,
            args,
            line_info,
        } => {
            let callable = env
                .get_function(name)
                .ok_or_else(|| EvalError::UndefinedVariable(name.clone(), line_info.clone()))?
                .clone();

            let mut evaluated_args = Vec::new();
            for arg in args {
                let evaluated_arg = evaluate(arg, env)?;
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
                    let eval_args: Vec<EvalResult> =
                        evaluated_args.into_iter().map(|arg| arg.value).collect();
                    let params = function.params.clone();
                    env.push_scope();

                    if eval_args.len() != params.len() {
                        return Err(EvalError::InvalidOperation(
                            format!(
                                "Function '{}' expected {} arguments but got {}.",
                                name,
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
                                        name
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

                    let result = evaluate(&function.body, env)?;
                    env.pop_scope();

                    match function.return_type {
                        Type::Arcana => match result {
                            EvalResult::Arcana(n) => Ok(EvalResult::Arcana(n)),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Aether => match result {
                            EvalResult::Aether(n) => Ok(EvalResult::Aether(n)),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Rune => match result {
                            EvalResult::Rune(s) => Ok(EvalResult::Rune(s)),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Omen => match result {
                            EvalResult::Omen(b) => Ok(EvalResult::Omen(b)),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Abyss => match result {
                            EvalResult::Abyss => Ok(EvalResult::Abyss),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Scroll => match result {
                            EvalResult::Scroll(items) => Ok(EvalResult::Scroll(items)),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Lexicon => match result {
                            EvalResult::Lexicon(entries) => Ok(EvalResult::Lexicon(entries)),
                            _ => Err(EvalError::TypeError(
                                format!("Type mismatch for return value of function {}", name),
                                function.line_info.clone(),
                            )),
                        },
                        Type::Materia => Ok(result),
                    }
                }
                Callable::Builtin(function) => {
                    (function.func)(env, evaluated_args, line_info.clone())
                }
            }
        }
        AST::Comment(_, _) => Ok(EvalResult::Abyss),
        _ => Err(EvalError::InvalidOperation(
            format!("Unsupported operation: {:?}", ast),
            None,
        )),
    }
}
