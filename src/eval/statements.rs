use crate::ast::{AST, AssignmentOp, ConditionalAssignment, Type};
use crate::env::{Callable, EngravedFunction, Environment, Value};

use super::collections::{
    collect_index_chain, expect_arcana_index, expect_rune_key, resolve_nested_value_mut,
};
use super::expressions::try_evaluate_expression;
use super::result::{EvalError, EvalResult};
use super::values::{
    convert_to_typed_value, eval_result_to_value_checked, extract_aether, extract_arcana,
    extract_omen, extract_rune,
};

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
    if let Some(result) = try_evaluate_expression(ast, env)? {
        return Ok(result);
    }

    match ast {
        AST::Statement(node, _line_info) => evaluate(node, env),
        AST::Omen(..)
        | AST::Arcana(..)
        | AST::Aether(..)
        | AST::Rune(..)
        | AST::Abyss(..)
        | AST::ListLiteral { .. }
        | AST::MapLiteral { .. }
        | AST::Add(..)
        | AST::Sub(..)
        | AST::Mul(..)
        | AST::Div(..)
        | AST::Mod(..)
        | AST::PowArcana(..)
        | AST::PowAether(..)
        | AST::Equal(..)
        | AST::NotEqual(..)
        | AST::LessThan(..)
        | AST::LessThanOrEqual(..)
        | AST::GreaterThan(..)
        | AST::GreaterThanOrEqual(..)
        | AST::LogicalAnd(..)
        | AST::LogicalOr(..)
        | AST::LogicalNot(..)
        | AST::Var(..)
        | AST::Trans(..)
        | AST::IndexAccess { .. }
        | AST::FuncCall { .. } => unreachable!("expression nodes handled earlier"),
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
        AST::Comment(_, _) => Ok(EvalResult::Abyss),
        _ => Err(EvalError::InvalidOperation(
            format!("Unsupported operation: {:?}", ast),
            None,
        )),
    }
}
