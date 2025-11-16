use crate::ast::{AST, AssignmentOp, ConditionalAssignment, LineInfo, Type};
use crate::env::{ArtifactMethod, Callable, EngravedFunction, Environment, Value};
use std::rc::Rc;

use super::artifacts::{
    build_artifact_schema, collect_field_chain, ensure_field_exists, ensure_type_known,
    expect_artifact_handle, lookup_schema_from_handle, missing_field_error,
};
use super::collections::{collect_index_chain, expect_arcana_index, expect_rune_key};
use super::expressions::try_evaluate_expression;
use super::result::{EvalError, EvalResult};
use super::values::{
    convert_to_typed_value, describe_value, eval_result_to_value_checked, extract_aether,
    extract_arcana, extract_omen, extract_rune,
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
        | AST::IndexAccess { .. }
        | AST::FuncCall { .. }
        | AST::MethodCall { .. } => unreachable!("expression nodes handled earlier"),
        AST::VarAssign {
            name,
            value,
            var_type,
            is_morph,
            line_info,
        } => {
            ensure_type_known(var_type, env, line_info)?;
            let evaluated_value = evaluate(value, env)?;
            let stored_value = convert_to_typed_value(evaluated_value, var_type, line_info)?;
            env.set_var(
                name.clone(),
                stored_value,
                var_type.clone(),
                *is_morph,
                line_info.clone(),
            );
            Ok(EvalResult::abyss())
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
                            let rhs = extract_rune(&evaluated_value, line_info)?;
                            let mut new_value = current.as_ref().clone();
                            new_value.push_str(&rhs);
                            *current = Rc::new(new_value);
                        }
                        AssignmentOp::Assign => {
                            let rhs = extract_rune(&evaluated_value, line_info)?;
                            *current = Rc::new(rhs);
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
                        if !matches!(evaluated_value, EvalResult::Data(Value::Abyss)) {
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

                Ok(EvalResult::abyss())
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

            let mut resolved_target = var_info.value.clone();
            for idx in &evaluated_indices {
                resolved_target = clone_indexed_child(&resolved_target, idx, line_info)?;
            }

            match resolved_target {
                Value::Scroll(handle) => {
                    let idx = expect_arcana_index(&final_index_value, line_info)?;
                    let mut items = handle.borrow_mut();
                    if idx >= items.len() {
                        return Err(EvalError::InvalidOperation(
                            format!("Index {} is out of bounds for scroll", idx),
                            line_info.clone(),
                        ));
                    }
                    items[idx] = new_value;
                }
                Value::Lexicon(handle) => {
                    let key = expect_rune_key(&final_index_value, line_info)?;
                    let mut entries = handle.borrow_mut();
                    entries.insert(key, new_value);
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "Indexed assignment requires a scroll or lexicon".to_string(),
                        line_info.clone(),
                    ));
                }
            }

            Ok(EvalResult::abyss())
        }
        AST::FieldAssignment {
            target,
            field,
            value,
            line_info,
        } => {
            let (base_name, access_chain) = collect_field_chain(target).ok_or_else(|| {
                EvalError::InvalidOperation(
                    "Field assignment requires an artifact variable".to_string(),
                    line_info.clone(),
                )
            })?;

            let evaluated_value = evaluate(value, env)?;

            let var_info = env.get_var_mut(&base_name).ok_or_else(|| {
                EvalError::UndefinedVariable(base_name.clone(), line_info.clone())
            })?;

            if !var_info.is_morph {
                return Err(EvalError::InvalidOperation(
                    format!("Cannot reassign to immutable variable {}", base_name),
                    line_info.clone(),
                ));
            }

            let mut current_handle = expect_artifact_handle(&var_info.value, line_info)?;
            for segment in &access_chain {
                let schema = lookup_schema_from_handle(env, &current_handle, line_info)?;
                ensure_field_exists(schema, segment, line_info)?;
                let next_value = {
                    let borrowed = current_handle.borrow();
                    borrowed
                        .fields
                        .get(segment)
                        .cloned()
                        .ok_or_else(|| missing_field_error(schema, segment, line_info))?
                };
                current_handle = match next_value {
                    Value::Artifact(handle) => handle,
                    other => {
                        return Err(EvalError::InvalidOperation(
                            format!(
                                "Field '{}' is not an artifact (found {})",
                                segment,
                                describe_value(&other)
                            ),
                            line_info.clone(),
                        ));
                    }
                };
            }

            let schema = lookup_schema_from_handle(env, &current_handle, line_info)?;
            let field_schema = ensure_field_exists(schema, field, line_info)?;
            let typed_value =
                convert_to_typed_value(evaluated_value, &field_schema.field_type, line_info)?;

            let mut borrowed = current_handle.borrow_mut();
            borrowed.fields.insert(field.clone(), typed_value);

            Ok(EvalResult::abyss())
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
                        EvalResult::Data(Value::Arcana(n)) => env.set_var(
                            conditional.variable.clone(),
                            Value::Arcana(n),
                            Type::Arcana,
                            false,
                            line_info.clone(),
                        ),
                        EvalResult::Data(Value::Aether(n)) => env.set_var(
                            conditional.variable.clone(),
                            Value::Aether(n),
                            Type::Aether,
                            false,
                            line_info.clone(),
                        ),
                        EvalResult::Data(Value::Rune(rune)) => env.set_var(
                            conditional.variable.clone(),
                            Value::Rune(rune.clone()),
                            Type::Rune,
                            false,
                            line_info.clone(),
                        ),
                        EvalResult::Data(Value::Omen(b)) => env.set_var(
                            conditional.variable.clone(),
                            Value::Omen(b),
                            Type::Omen,
                            false,
                            line_info.clone(),
                        ),
                        other => {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported type in oracle conditional: {:?}", other),
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
                                (
                                    EvalResult::Data(Value::Arcana(cond_n)),
                                    EvalResult::Data(Value::Arcana(pat_n)),
                                ) => {
                                    if cond_n != pat_n {
                                        matched = false;
                                        break;
                                    }
                                }
                                (
                                    EvalResult::Data(Value::Aether(cond_n)),
                                    EvalResult::Data(Value::Aether(pat_n)),
                                ) => {
                                    if (cond_n - pat_n).abs() >= f64::EPSILON {
                                        matched = false;
                                        break;
                                    }
                                }
                                (
                                    EvalResult::Data(Value::Rune(cond_s)),
                                    EvalResult::Data(Value::Rune(pat_s)),
                                ) => {
                                    if cond_s != pat_s {
                                        matched = false;
                                        break;
                                    }
                                }
                                (
                                    EvalResult::Data(Value::Omen(cond_b)),
                                    EvalResult::Data(Value::Omen(pat_b)),
                                ) => {
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
                        let mut all_true = true;
                        for pattern_expr in pattern {
                            match evaluate(pattern_expr, env)? {
                                EvalResult::Data(Value::Omen(true)) => continue,
                                EvalResult::Data(Value::Omen(false)) => {
                                    all_true = false;
                                    break;
                                }
                                other => {
                                    return Err(EvalError::InvalidOperation(
                                        format!(
                                            "Oracle guard must evaluate to an omen, found {:?}",
                                            other
                                        ),
                                        line_info.clone(),
                                    ));
                                }
                            }
                        }
                        all_true
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
            Ok(EvalResult::abyss())
        }
        AST::Reveal(expr, _line_info) => {
            let result = evaluate(expr, env)?;
            Ok(EvalResult::Revealed(Box::new(result)))
        }
        AST::Block(statements, _line_info) => {
            let mut last_result = EvalResult::abyss();
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
        AST::OracleDontCareItem(_line_info) => Ok(EvalResult::data(Value::Omen(true))),
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

                Ok(EvalResult::abyss())
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

                let start_num = extract_arcana(&start_value, line_info)?;
                let end_num = extract_arcana(&end_value, line_info)?;

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
                Ok(EvalResult::abyss())
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
            method_target,
            line_info,
        } => {
            ensure_type_known(return_type, env, line_info)?;
            for param in params {
                if let AST::EngraveParam {
                    param_type,
                    line_info: param_info,
                    ..
                } = param
                {
                    ensure_type_known(param_type, env, param_info)?;
                }
            }
            let function_name = if let Some(target) = method_target {
                format!("{}::{}", target.artifact, name)
            } else {
                name.clone()
            };
            let function = EngravedFunction {
                name: function_name,
                params: params.clone(),
                return_type: return_type.clone(),
                body: body.clone(),
                line_info: line_info.clone(),
            };
            if let Some(target) = method_target {
                let artifact_method = ArtifactMethod {
                    function,
                    requires_mutable_receiver: target.requires_morph,
                };
                env.add_artifact_method(&target.artifact, name, artifact_method, line_info)?;
            } else {
                env.set_function(name.clone(), Callable::Engraved(function));
            }
            Ok(EvalResult::abyss())
        }
        AST::ArtifactDef {
            name,
            fields,
            line_info,
        } => {
            if env.artifact_defined_in_current_scope(name) {
                return Err(EvalError::InvalidOperation(
                    format!("Artifact {} is already defined", name),
                    line_info.clone(),
                ));
            }
            let schema = build_artifact_schema(name, fields, env, line_info)?;
            env.define_artifact(schema)?;
            env.set_var(
                name.clone(),
                Value::Glyph(Type::Artifact(name.clone())),
                Type::Glyph,
                false,
                line_info.clone(),
            );
            Ok(EvalResult::abyss())
        }
        AST::Comment(_, _) => Ok(EvalResult::abyss()),
        _ => Err(EvalError::InvalidOperation(
            format!("Unsupported operation: {:?}", ast),
            None,
        )),
    }
}

fn clone_indexed_child(
    value: &Value,
    index: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<Value, EvalError> {
    match value {
        Value::Scroll(handle) => {
            let idx = expect_arcana_index(index, line_info)?;
            let borrowed = handle.borrow();
            borrowed.get(idx).cloned().ok_or_else(|| {
                EvalError::InvalidOperation(
                    format!("Index {} is out of bounds for scroll", idx),
                    line_info.clone(),
                )
            })
        }
        Value::Lexicon(handle) => {
            let key = expect_rune_key(index, line_info)?;
            let borrowed = handle.borrow();
            borrowed.get(&key).cloned().ok_or_else(|| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::{ArtifactFieldSchema, ArtifactSchema, ArtifactValue};
    use std::cell::RefCell;
    use std::collections::HashMap;

    fn line() -> Option<LineInfo> {
        Some(LineInfo::new(1, 1))
    }

    fn scroll(values: Vec<Value>) -> Value {
        Value::Scroll(Rc::new(RefCell::new(values)))
    }

    fn artifact_handle(name: &str, fields: Vec<(&str, Value)>) -> Rc<RefCell<ArtifactValue>> {
        let mut map = HashMap::new();
        let mut order = Vec::new();
        for (field, value) in fields {
            let key = field.to_string();
            order.push(key.clone());
            map.insert(key, value);
        }
        Rc::new(RefCell::new(ArtifactValue {
            type_name: name.to_string(),
            fields: map,
            field_order: order,
        }))
    }

    fn register_artifact(env: &mut Environment, name: &str, fields: Vec<(&str, Type)>) {
        let schema = ArtifactSchema {
            name: name.to_string(),
            fields: fields
                .into_iter()
                .map(|(field, field_type)| ArtifactFieldSchema {
                    name: field.to_string(),
                    field_type,
                })
                .collect(),
            methods: HashMap::new(),
            line_info: None,
        };
        env.define_artifact(schema).expect("schema registration");
    }

    #[test]
    fn arcana_assignment_supports_compound_ops() {
        let mut env = Environment::new();
        env.set_var("sigil".into(), Value::Arcana(2), Type::Arcana, true, line());

        let assignment = AST::Assignment {
            name: "sigil".into(),
            value: Box::new(AST::Arcana(5, line())),
            op: AssignmentOp::AddAssign,
            line_info: line(),
        };

        evaluate(&assignment, &mut env).expect("assignment should succeed");
        let stored = env.get_var("sigil").expect("variable exists");
        match &stored.value {
            Value::Arcana(value) => assert_eq!(*value, 7),
            other => panic!("unexpected value {:?}", other),
        }
    }

    #[test]
    fn assignment_rejects_immutable_variables() {
        let mut env = Environment::new();
        env.set_var(
            "sigil".into(),
            Value::Arcana(2),
            Type::Arcana,
            false,
            line(),
        );

        let assignment = AST::Assignment {
            name: "sigil".into(),
            value: Box::new(AST::Arcana(5, line())),
            op: AssignmentOp::Assign,
            line_info: line(),
        };

        let err = evaluate(&assignment, &mut env).expect_err("immutable reassign should fail");
        match err {
            EvalError::InvalidOperation(..) => {}
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn index_assignment_updates_scroll_entries() {
        let mut env = Environment::new();
        env.set_var(
            "scroll".into(),
            scroll(vec![Value::Arcana(0), Value::Arcana(1)]),
            Type::Scroll,
            true,
            line(),
        );

        let index_assignment = AST::IndexAssignment {
            target: Box::new(AST::Var("scroll".into(), line())),
            index: Box::new(AST::Arcana(1, line())),
            value: Box::new(AST::Arcana(99, line())),
            line_info: line(),
        };

        evaluate(&index_assignment, &mut env).expect("index assignment succeeds");
        let stored = env.get_var("scroll").expect("scroll exists");
        if let Value::Scroll(handle) = &stored.value {
            let borrowed = handle.borrow();
            match &borrowed[1] {
                Value::Arcana(value) => assert_eq!(*value, 99),
                other => panic!("unexpected value {:?}", other),
            }
        } else {
            panic!("expected scroll value");
        }
    }

    #[test]
    fn field_assignment_updates_nested_artifact_fields() {
        let mut env = Environment::new();
        register_artifact(&mut env, "Glyph", vec![("power", Type::Arcana)]);
        register_artifact(
            &mut env,
            "Sigil",
            vec![("core", Type::Artifact("Glyph".into()))],
        );

        let inner = artifact_handle("Glyph", vec![("power", Value::Arcana(3))]);
        let outer = artifact_handle("Sigil", vec![("core", Value::Artifact(inner.clone()))]);
        env.set_var(
            "sigil".into(),
            Value::Artifact(outer.clone()),
            Type::Artifact("Sigil".into()),
            true,
            line(),
        );

        let target = AST::FieldAccess {
            target: Box::new(AST::Var("sigil".into(), line())),
            field: "core".into(),
            line_info: line(),
        };
        let assignment = AST::FieldAssignment {
            target: Box::new(target),
            field: "power".into(),
            value: Box::new(AST::Arcana(10, line())),
            line_info: line(),
        };

        evaluate(&assignment, &mut env).expect("field assignment succeeds");
        let borrowed = inner.borrow();
        match borrowed.fields.get("power") {
            Some(Value::Arcana(value)) => assert_eq!(*value, 10),
            other => panic!("unexpected field value {:?}", other),
        }
        drop(borrowed);
        let outer_borrow = outer.borrow();
        assert!(outer_borrow.fields.contains_key("core"));
    }

    #[test]
    fn oracle_match_branch_returns_revealed_value() {
        let mut env = Environment::new();
        let conditional = ConditionalAssignment {
            variable: "sigil".into(),
            expression: Box::new(AST::Arcana(1, line())),
            line_info: line(),
        };

        let branch = AST::OracleBranch {
            pattern: vec![AST::Arcana(1, line())],
            body: Box::new(AST::Reveal(Box::new(AST::Arcana(42, line())), line())),
            line_info: line(),
        };

        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![conditional],
            branches: vec![branch],
            line_info: line(),
        };

        let result = evaluate(&oracle, &mut env).expect("oracle should succeed");
        match result {
            EvalResult::Data(Value::Arcana(value)) => assert_eq!(value, 42),
            other => panic!("unexpected oracle result {:?}", other),
        }
    }

    #[test]
    fn clone_indexed_child_errors_on_non_collections() {
        let err = clone_indexed_child(
            &Value::Arcana(1),
            &EvalResult::data(Value::Arcana(0)),
            &line(),
        )
        .expect_err("non-collections should fail");
        match err {
            EvalError::InvalidOperation(_, info) => assert!(info.is_some()),
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn artifact_definition_creates_glyph_variable() {
        let mut env = Environment::new();
        let artifact = AST::ArtifactDef {
            name: "Relic".into(),
            fields: Vec::new(),
            line_info: line(),
        };

        evaluate(&artifact, &mut env).expect("artifact definition succeeds");

        let glyph_entry = env.get_var("Relic").expect("glyph variable exists");
        assert_eq!(glyph_entry.var_type, Type::Glyph);
        assert!(!glyph_entry.is_morph);
        match &glyph_entry.value {
            Value::Glyph(Type::Artifact(name)) if name == "Relic" => {}
            other => panic!("unexpected glyph value {:?}", other),
        }
    }
}
