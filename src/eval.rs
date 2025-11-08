use crate::ast::{AST, AssignmentOp, ConditionalAssignment, LineInfo, Type};
use crate::env::{Environment, Function, Value};
use colored::*;
use std::{fmt, io::Write};

/// Represents the result of an evaluation in the interpreter.
#[derive(Debug)]
pub enum EvalResult {
    Omen(bool),
    Arcana(i64),
    Aether(f64),
    Rune(String),
    Abyss,
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
            let value = match evaluate(value, env)? {
                EvalResult::Omen(b) if *var_type == Type::Omen => Value::Omen(b),
                EvalResult::Arcana(n) if *var_type == Type::Arcana => Value::Arcana(n),
                EvalResult::Aether(n) if *var_type == Type::Aether => Value::Aether(n),
                EvalResult::Rune(s) if *var_type == Type::Rune => Value::Rune(s),
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "VarAssign operation requires a valid type!".to_string(),
                        line_info.clone(),
                    ));
                }
            };
            env.set_var(
                name.clone(),
                value,
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

            if let Some(var_info) = env.get_var(name) {
                if !var_info.is_morph {
                    return Err(EvalError::InvalidOperation(
                        format!("Cannot reassign to immutable variable {}", name),
                        line_info.clone(),
                    ));
                }

                let result = match (evaluated_value, &var_info.value, op) {
                    (EvalResult::Arcana(v), Value::Arcana(current), op) => {
                        let new_value = match op {
                            AssignmentOp::AddAssign => current + v,
                            AssignmentOp::SubAssign => current - v,
                            AssignmentOp::MulAssign => current * v,
                            AssignmentOp::DivAssign => current / v,
                            AssignmentOp::ModAssign => current % v,
                            AssignmentOp::PowArcanaAssign => {
                                if v < 0 {
                                    return Err(EvalError::NegativeExponent(line_info.clone()));
                                }
                                current.pow(v as u32)
                            }
                            AssignmentOp::Assign => v,
                            _ => {
                                return Err(EvalError::InvalidOperation(
                                    format!("Unsupported operation for variable {}", name),
                                    line_info.clone(),
                                ));
                            }
                        };
                        env.update_var(
                            name,
                            Value::Arcana(new_value),
                            Type::Arcana,
                            line_info.clone(),
                        )
                    }
                    (EvalResult::Aether(v), Value::Aether(current), op) => {
                        let new_value = match op {
                            AssignmentOp::AddAssign => current + v,
                            AssignmentOp::SubAssign => current - v,
                            AssignmentOp::MulAssign => current * v,
                            AssignmentOp::DivAssign => current / v,
                            AssignmentOp::ModAssign => current % v,
                            AssignmentOp::PowAetherAssign => current.powf(v),
                            AssignmentOp::Assign => v,
                            _ => {
                                return Err(EvalError::InvalidOperation(
                                    format!("Unsupported operation for variable {}", name),
                                    line_info.clone(),
                                ));
                            }
                        };
                        env.update_var(
                            name,
                            Value::Aether(new_value),
                            Type::Aether,
                            line_info.clone(),
                        )
                    }
                    (EvalResult::Rune(v), Value::Rune(current), op) => {
                        let new_value = match op {
                            AssignmentOp::AddAssign => format!("{}{}", current, v),
                            AssignmentOp::Assign => v,
                            _ => {
                                return Err(EvalError::InvalidOperation(
                                    format!("Unsupported operation for variable {}", name),
                                    line_info.clone(),
                                ));
                            }
                        };
                        env.update_var(name, Value::Rune(new_value), Type::Rune, line_info.clone())
                    }
                    (EvalResult::Omen(v), _, AssignmentOp::Assign) => {
                        env.update_var(name, Value::Omen(v), Type::Omen, line_info.clone())
                    }
                    _ => Err(EvalError::InvalidOperation(
                        format!(
                            "Type mismatch or unsupported operation for variable {}",
                            name
                        ),
                        line_info.clone(),
                    )),
                };

                result.map(|_| EvalResult::Abyss)
            } else {
                Err(EvalError::UndefinedVariable(
                    name.clone(),
                    line_info.clone(),
                ))
            }
        }
        AST::Var(name, line_info) => match env.get_var(name) {
            Some(var_info) => match &var_info.value {
                Value::Omen(b) => Ok(EvalResult::Omen(*b)),
                Value::Arcana(n) => Ok(EvalResult::Arcana(*n)),
                Value::Aether(n) => Ok(EvalResult::Aether(*n)),
                Value::Rune(s) => Ok(EvalResult::Rune(s.clone())),
            },
            None => Err(EvalError::UndefinedVariable(
                name.clone(),
                line_info.clone(),
            )),
        },
        AST::Unveil(args, _line_info) => {
            let outputs: Result<Vec<String>, EvalError> = args
                .iter()
                .map(|arg| evaluate(arg, env))
                .collect::<Result<Vec<EvalResult>, EvalError>>()?
                .iter()
                .map(|result| match result {
                    EvalResult::Omen(b) => match b {
                        true => Ok("boon".to_string()),
                        false => Ok("hex".to_string()),
                    },
                    EvalResult::Arcana(n) => Ok(n.to_string()),
                    EvalResult::Aether(n) => Ok(n.to_string()),
                    EvalResult::Rune(s) => Ok(s.replace("\\n", "\n")),
                    EvalResult::Abyss => Ok("".to_string()),
                    _ => Err(EvalError::InvalidOperation(
                        "Unsupported type in unveil statement".to_string(),
                        None,
                    )),
                })
                .collect();
            let output_str = outputs?.join("");
            println!("{}", output_str);
            Ok(EvalResult::Abyss)
        }
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
            let function = Function {
                name: name.clone(),
                params: params.clone(),
                return_type: return_type.clone(),
                body: body.clone(),
                line_info: line_info.clone(),
            };
            env.set_function(name.clone(), function);
            Ok(EvalResult::Abyss)
        }
        AST::FuncCall {
            name,
            args,
            line_info,
        } => {
            let function = {
                env.get_function(name)
                    .ok_or_else(|| EvalError::UndefinedVariable(name.clone(), line_info.clone()))?
            }
            .clone();

            let params = function.params.clone();

            let mut evaluated_args = Vec::new();
            for arg in args {
                let evaluated_arg = evaluate(arg, env)?;
                evaluated_args.push(evaluated_arg);
            }

            env.push_scope();

            for (evaluated_arg, param) in evaluated_args.into_iter().zip(params.iter()) {
                let (name, param_type) = match param {
                    AST::EngraveParam {
                        name, param_type, ..
                    } => (name, param_type),
                    _ => {
                        return Err(EvalError::InvalidOperation(
                            format!("Expected EngraveParam in function definition: {}", name),
                            line_info.clone(),
                        ));
                    }
                };
                let value = match (evaluated_arg, param_type) {
                    (EvalResult::Arcana(n), Type::Arcana) => Value::Arcana(n),
                    (EvalResult::Aether(n), Type::Aether) => Value::Aether(n),
                    (EvalResult::Rune(s), Type::Rune) => Value::Rune(s),
                    (EvalResult::Omen(b), Type::Omen) => Value::Omen(b),
                    _ => {
                        return Err(EvalError::TypeError(
                            format!("Type mismatch for parameter {}", name),
                            line_info.clone(),
                        ));
                    }
                };
                env.set_var(
                    name.to_string(),
                    value,
                    param_type.clone(),
                    false,
                    line_info.clone(),
                );
            }

            let result = evaluate(&function.body, env)?;

            env.pop_scope();

            match (result, function.return_type) {
                (EvalResult::Arcana(n), Type::Arcana) => Ok(EvalResult::Arcana(n)),
                (EvalResult::Aether(n), Type::Aether) => Ok(EvalResult::Aether(n)),
                (EvalResult::Rune(s), Type::Rune) => Ok(EvalResult::Rune(s)),
                (EvalResult::Omen(b), Type::Omen) => Ok(EvalResult::Omen(b)),
                (EvalResult::Abyss, Type::Abyss) => Ok(EvalResult::Abyss),
                _ => Err(EvalError::TypeError(
                    format!("Type mismatch for return value of function {}", name),
                    function.line_info.clone(),
                )),
            }
        }
        AST::Summon(prompt, var_type, line_info) => {
            print!("{}", prompt.trim_matches('"'));
            std::io::stdout().flush().map_err(|_| {
                EvalError::InvalidOperation("Failed to flush stdout".to_string(), line_info.clone())
            })?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(|_| {
                EvalError::InvalidOperation("Failed to read input".to_string(), line_info.clone())
            })?;
            match var_type {
                Type::Arcana => input
                    .trim()
                    .parse::<i64>()
                    .map(EvalResult::Arcana)
                    .map_err(|_| {
                        EvalError::InvalidOperation(
                            "Failed to parse input as Arcana".to_string(),
                            line_info.clone(),
                        )
                    }),
                Type::Aether => input
                    .trim()
                    .parse::<f64>()
                    .map(EvalResult::Aether)
                    .map_err(|_| {
                        EvalError::InvalidOperation(
                            "Failed to parse input as Aether".to_string(),
                            line_info.clone(),
                        )
                    }),
                Type::Rune => Ok(EvalResult::Rune(input.trim().to_string())),
                _ => Err(EvalError::InvalidOperation(
                    "Unsupported type for summon".to_string(),
                    line_info.clone(),
                )),
            }
        }
        AST::Comment(_, _) => Ok(EvalResult::Abyss),
        _ => Err(EvalError::InvalidOperation(
            format!("Unsupported operation: {:?}", ast),
            None,
        )),
    }
}
