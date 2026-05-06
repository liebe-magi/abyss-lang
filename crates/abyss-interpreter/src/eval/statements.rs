use crate::env::{ArtifactMethod, Callable, EngravedFunction, RuntimeEnv, Value};
use abyss_core::ast::{AST, AssignmentOp, ConditionalAssignment, LineInfo, Type};
use std::cell::RefCell;
use std::rc::Rc;

use super::artifacts::{
    build_artifact_schema, collect_field_chain, ensure_field_exists, ensure_type_known,
    expect_artifact_handle, lookup_schema_from_handle, missing_field_error, read_artifact_field,
    values_equal,
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
pub fn evaluate(ast: &AST, env: &mut RuntimeEnv) -> Result<EvalResult, EvalError> {
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
                Err(env.undefined_variable_error(name, line_info.clone()))
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

            let var_info = match env.get_var_mut(&base_name) {
                Some(var_info) => var_info,
                None => {
                    return Err(env.undefined_variable_error(&base_name, line_info.clone()));
                }
            };

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

            let var_info = match env.get_var_mut(&base_name) {
                Some(var_info) => var_info,
                None => {
                    return Err(env.undefined_variable_error(&base_name, line_info.clone()));
                }
            };

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
            // Push the oracle's local scope, run the body, and unconditionally
            // pop on the way out — including error paths — so a failing
            // scrutinee, pattern, ward, or body cannot leak a scope back into
            // the REPL. The helper itself uses `?` freely.
            env.push_scope();
            let result = evaluate_oracle(*is_match, conditionals, branches, line_info, env);
            env.pop_scope();
            result
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

/// Inner body of `AST::Oracle` evaluation. The caller is responsible for
/// pairing one `env.push_scope()` before this call with one `env.pop_scope()`
/// after, so every `?` inside this helper unwinds through the caller and the
/// outer oracle scope is always popped.
///
/// Each branch additionally gets its own nested scope (push/pop) so that
/// match-mode bindings (`(x) =>`) and any `forge` declarations in the body
/// stay confined to the arm that produced them — without leaking sideways
/// to subsequent arms or upward to the script.
fn evaluate_oracle(
    is_match: bool,
    conditionals: &[ConditionalAssignment],
    branches: &[AST],
    line_info: &Option<LineInfo>,
    env: &mut RuntimeEnv,
) -> Result<EvalResult, EvalError> {
    let mut scrutinee_values = Vec::with_capacity(conditionals.len());
    for conditional in conditionals {
        let result = evaluate(&conditional.expression, env)?;
        let stored = match result {
            EvalResult::Data(Value::Arcana(n)) => Value::Arcana(n),
            EvalResult::Data(Value::Aether(n)) => Value::Aether(n),
            EvalResult::Data(Value::Rune(rune)) => Value::Rune(rune.clone()),
            EvalResult::Data(Value::Omen(b)) => Value::Omen(b),
            // Scrolls flow through as their shared handle so a scroll pattern
            // arm sees the same elements the user passed in. Mutating the
            // scroll inside the arm body therefore visibly mutates the
            // outer value, matching the existing aliasing semantics for the
            // `scroll` type elsewhere in the interpreter.
            EvalResult::Data(Value::Scroll(handle)) => Value::Scroll(handle.clone()),
            // Artifacts (typed records) flow through similarly so an
            // artifact pattern arm sees the same handle the user passed
            // in. The `EvalResult::Artifact` variant is also accepted so
            // the helper can be invoked equally from places that hand
            // through the dedicated artifact result.
            EvalResult::Data(Value::Artifact(handle)) => Value::Artifact(handle.clone()),
            EvalResult::Artifact(handle) => Value::Artifact(handle.clone()),
            // Lexicons flow through as their shared handle so a lexicon
            // pattern arm sees the same entries the user passed in. Mutating
            // the lexicon inside the arm body therefore visibly mutates the
            // outer value, matching the existing aliasing semantics.
            EvalResult::Data(Value::Lexicon(handle)) => Value::Lexicon(handle.clone()),
            other => {
                return Err(EvalError::InvalidOperation(
                    format!("Unsupported type in oracle scrutinee: {:?}", other),
                    line_info.clone(),
                ));
            }
        };
        scrutinee_values.push(stored);
    }

    for branch in branches {
        if let AST::Comment(_, _) = branch {
            continue;
        }

        let AST::OracleBranch {
            pattern,
            guard,
            body,
            line_info,
        } = branch
        else {
            continue;
        };

        env.push_scope();
        let outcome = evaluate_oracle_branch(
            is_match,
            pattern,
            guard.as_deref(),
            body,
            line_info,
            &scrutinee_values,
            env,
        );
        env.pop_scope();

        match outcome? {
            None => continue,
            Some(result) => return Ok(result),
        }
    }

    Ok(EvalResult::abyss())
}

/// Evaluate a single `OracleBranch`. Returns:
/// - `Ok(Some(result))` when the pattern matches, the optional ward holds,
///   and the body has been evaluated to `result`;
/// - `Ok(None)` when the arm does not apply (pattern mismatch or ward
///   yielded `hex`) and the caller should try the next arm;
/// - `Err(e)` on any evaluation error.
///
/// The caller is responsible for pushing and popping the per-branch scope
/// around this call. Match-mode bindings introduced by bare-identifier
/// patterns are written into the current (caller-pushed) scope so they are
/// visible to the ward expression and the body, then unwound when the
/// caller pops.
fn evaluate_oracle_branch(
    is_match: bool,
    pattern: &[AST],
    guard: Option<&AST>,
    body: &AST,
    line_info: &Option<LineInfo>,
    scrutinee_values: &[Value],
    env: &mut RuntimeEnv,
) -> Result<Option<EvalResult>, EvalError> {
    let matched = if pattern.is_empty() {
        true
    } else if is_match {
        if pattern.len() != scrutinee_values.len() {
            return Err(EvalError::InvalidOperation(
                format!(
                    "Oracle branch pattern length {} does not match scrutinee length {}",
                    pattern.len(),
                    scrutinee_values.len()
                ),
                line_info.clone(),
            ));
        }

        let mut matched = true;
        for (idx, pattern_elem) in pattern.iter().enumerate() {
            if let AST::OracleDontCareItem(_) = pattern_elem {
                continue;
            }

            let Some(scrutinee_value) = scrutinee_values.get(idx) else {
                return Err(EvalError::InvalidOperation(
                    "Oracle branch references missing scrutinee".to_string(),
                    line_info.clone(),
                ));
            };

            // A bare identifier in match-mode pattern position introduces a
            // fresh binding to the scrutinee value (rather than looking the
            // identifier up as an expression). The binding lives in the
            // per-branch scope the caller pushed, so it is visible to the
            // ward and body of this arm and disappears when the arm finishes.
            if let AST::Var(name, var_line) = pattern_elem {
                env.set_var(
                    name.clone(),
                    scrutinee_value.clone(),
                    type_of_scrutinee(scrutinee_value),
                    false,
                    var_line.clone(),
                );
                continue;
            }

            // Scroll-shape pattern destructures the scrutinee — which must be
            // a `scroll` — into its elements, with optional trailing rest.
            if let AST::OracleScrollPattern {
                elements,
                line_info: scroll_line,
            } = pattern_elem
            {
                if !match_scroll_pattern(elements, scrutinee_value, scroll_line, env)? {
                    matched = false;
                    break;
                }
                continue;
            }

            // Artifact-shape pattern destructures the scrutinee — which must
            // be an artifact of the named type — by pulling out the listed
            // fields. Fields not listed are not matched against, so partial
            // patterns like `Player { name }` are valid.
            if let AST::OracleArtifactPattern {
                type_name,
                fields,
                line_info: artifact_line,
            } = pattern_elem
            {
                if !match_artifact_pattern(type_name, fields, scrutinee_value, artifact_line, env)?
                {
                    matched = false;
                    break;
                }
                continue;
            }

            // Lexicon-shape pattern destructures the scrutinee — which must
            // be a `lexicon` — by pulling out the listed keys. Keys not
            // listed are not matched against, so partial patterns like
            // `{ "name": n }` are valid; an absent key falls through.
            if let AST::OracleLexiconPattern {
                entries,
                line_info: lexicon_line,
            } = pattern_elem
            {
                if !match_lexicon_pattern(entries, scrutinee_value, lexicon_line, env)? {
                    matched = false;
                    break;
                }
                continue;
            }

            let pattern_result = evaluate(pattern_elem, env)?;

            match (scrutinee_value, pattern_result) {
                (Value::Arcana(cond_n), EvalResult::Data(Value::Arcana(pat_n))) => {
                    if *cond_n != pat_n {
                        matched = false;
                        break;
                    }
                }
                (Value::Aether(cond_n), EvalResult::Data(Value::Aether(pat_n))) => {
                    if (*cond_n - pat_n).abs() >= f64::EPSILON {
                        matched = false;
                        break;
                    }
                }
                (Value::Rune(cond_s), EvalResult::Data(Value::Rune(pat_s))) => {
                    if cond_s.as_ref() != pat_s.as_ref() {
                        matched = false;
                        break;
                    }
                }
                (Value::Omen(cond_b), EvalResult::Data(Value::Omen(pat_b))) => {
                    if *cond_b != pat_b {
                        matched = false;
                        break;
                    }
                }
                _ => {
                    return Err(EvalError::InvalidOperation(
                        "Oracle branch pattern type must match scrutinee type".to_string(),
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
                            "Oracle if-else pattern must evaluate to an omen, found {:?}",
                            other
                        ),
                        line_info.clone(),
                    ));
                }
            }
        }
        all_true
    };

    let matched = if matched {
        match guard {
            None => true,
            Some(guard_expr) => match evaluate(guard_expr, env)? {
                EvalResult::Data(Value::Omen(b)) => b,
                other => {
                    return Err(EvalError::InvalidOperation(
                        format!("Oracle ward must evaluate to an omen, found {:?}", other),
                        line_info.clone(),
                    ));
                }
            },
        }
    } else {
        false
    };

    if matched {
        let result = evaluate(body, env)?;
        let result = match result {
            EvalResult::Revealed(revealed) => *revealed,
            _ => result,
        };
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

/// Maps a `Value` to the runtime `Type` recorded when binding it into a
/// pattern arm. The mapping is exhaustive over the `Value` enum and mirrors
/// `stdlib::methods::receiver_type`, so a binding like `[x, ..]` against a
/// scroll-of-scrolls captures the inner scroll under `Type::Scroll` rather
/// than collapsing to `Materia`. The two helpers should ideally share a
/// single home (e.g. `eval/values.rs`); deferred to a focused refactor so
/// this PR stays scoped to scroll destructuring.
fn type_of_scrutinee(value: &Value) -> Type {
    match value {
        Value::Arcana(_) => Type::Arcana,
        Value::Aether(_) => Type::Aether,
        Value::Rune(_) => Type::Rune,
        Value::Omen(_) => Type::Omen,
        Value::Abyss => Type::Abyss,
        Value::Scroll(_) => Type::Scroll,
        Value::Lexicon(_) => Type::Lexicon,
        Value::Glyph(_) => Type::Glyph,
        Value::Artifact(handle) => Type::Artifact(handle.borrow().type_name.clone()),
    }
}

/// Match a scroll-shape pattern against a scrutinee `Value`, performing any
/// element-level bindings into the caller's scope as side-effects.
///
/// Returns `Ok(true)` when the scrutinee is a scroll whose contents satisfy
/// the pattern (and any bindings have been applied). Returns `Ok(false)`
/// when the scrutinee is a scroll but the lengths or element values do not
/// line up — in which case any bindings written before the mismatch are
/// left in place and the caller is expected to discard them by popping the
/// per-branch scope. Returns `Err` when the scrutinee is not a scroll
/// (type-mismatch error) or the pattern is malformed (multiple rest
/// segments, or a rest that is not at the end).
///
/// PR3 minimum scope: at most one rest segment, and only as the trailing
/// element. Mid-list rests like `[a, .., last]` are rejected here so the
/// matching logic stays linear.
fn match_scroll_pattern(
    elements: &[AST],
    scrutinee_value: &Value,
    line_info: &Option<LineInfo>,
    env: &mut RuntimeEnv,
) -> Result<bool, EvalError> {
    let scroll_handle = match scrutinee_value {
        Value::Scroll(handle) => handle.clone(),
        _ => {
            return Err(EvalError::InvalidOperation(
                format!(
                    "Scroll pattern requires a scroll scrutinee, found {:?}",
                    scrutinee_value
                ),
                line_info.clone(),
            ));
        }
    };

    let scroll_values: Vec<Value> = scroll_handle.borrow().clone();

    let mut rest_index: Option<usize> = None;
    for (idx, element) in elements.iter().enumerate() {
        if matches!(element, AST::OracleScrollRest { .. }) {
            if rest_index.is_some() {
                return Err(EvalError::InvalidOperation(
                    "Scroll pattern may contain at most one rest segment".to_string(),
                    line_info.clone(),
                ));
            }
            rest_index = Some(idx);
        }
    }

    if let Some(idx) = rest_index
        && idx != elements.len() - 1
    {
        return Err(EvalError::InvalidOperation(
            "Scroll rest segment must appear at the end of the pattern".to_string(),
            line_info.clone(),
        ));
    }

    let prefix_len = match rest_index {
        Some(_) => elements.len() - 1,
        None => elements.len(),
    };

    if rest_index.is_some() {
        if scroll_values.len() < prefix_len {
            return Ok(false);
        }
    } else if scroll_values.len() != prefix_len {
        return Ok(false);
    }

    for (idx, element) in elements.iter().take(prefix_len).enumerate() {
        let elem_value = &scroll_values[idx];
        match element {
            AST::OracleDontCareItem(_) => continue,
            AST::Var(name, var_line) => {
                env.set_var(
                    name.clone(),
                    elem_value.clone(),
                    type_of_scrutinee(elem_value),
                    false,
                    var_line.clone(),
                );
            }
            AST::OracleScrollPattern {
                elements: nested_elements,
                line_info: nested_line,
            } => {
                if !match_scroll_pattern(nested_elements, elem_value, nested_line, env)? {
                    return Ok(false);
                }
            }
            AST::OracleArtifactPattern {
                type_name,
                fields,
                line_info: artifact_line,
            } => {
                if !match_artifact_pattern(type_name, fields, elem_value, artifact_line, env)? {
                    return Ok(false);
                }
            }
            AST::OracleLexiconPattern {
                entries,
                line_info: lexicon_line,
            } => {
                if !match_lexicon_pattern(entries, elem_value, lexicon_line, env)? {
                    return Ok(false);
                }
            }
            other => {
                let pattern_result = evaluate(other, env)?;
                if !values_match_for_pattern(elem_value, &pattern_result, line_info)? {
                    return Ok(false);
                }
            }
        }
    }

    if let Some(idx) = rest_index
        && let AST::OracleScrollRest {
            name: Some(name),
            line_info: rest_line,
        } = &elements[idx]
    {
        let tail: Vec<Value> = scroll_values.iter().skip(prefix_len).cloned().collect();
        let tail_handle: Rc<RefCell<Vec<Value>>> = Rc::new(RefCell::new(tail));
        env.set_var(
            name.clone(),
            Value::Scroll(tail_handle),
            Type::Scroll,
            false,
            rest_line.clone(),
        );
    }

    Ok(true)
}

/// Match an artifact-shape pattern against a scrutinee `Value`, performing
/// any field-level bindings into the caller's scope as side-effects.
///
/// Returns `Ok(true)` when the scrutinee is an artifact of the named type
/// whose listed fields satisfy their sub-patterns (and any bindings have
/// been applied). Returns `Ok(false)` in two no-match cases:
///
/// - the scrutinee is an artifact, but of a different type than the
///   pattern names — falling through here lets users dispatch by writing
///   one arm per artifact type;
/// - the scrutinee is the right artifact type but a field-level
///   sub-pattern (a literal compare, a nested scroll pattern, etc.) did
///   not match.
///
/// Returns `Err` when the scrutinee is not an artifact at all, when the
/// pattern's `type_name` is not a defined artifact in scope, or when a
/// listed field name is not declared on that artifact's schema (in which
/// case the existing `did_you_mean` infrastructure from PR4-B surfaces a
/// "did you mean: …" hint via [`super::artifacts::missing_field_error`]).
///
/// Fields not mentioned in the pattern are intentionally unrestricted —
/// the pattern is non-exhaustive, mirroring the per-field "pick what you
/// need" ergonomics that Rust spells with `..` and OCaml requires
/// exhaustively. Adding an explicit rest marker can come later if the
/// distinction proves valuable.
fn match_artifact_pattern(
    type_name: &str,
    fields: &[(String, AST)],
    scrutinee_value: &Value,
    line_info: &Option<LineInfo>,
    env: &mut RuntimeEnv,
) -> Result<bool, EvalError> {
    let handle = match scrutinee_value {
        Value::Artifact(handle) => handle.clone(),
        _ => {
            return Err(EvalError::InvalidOperation(
                format!(
                    "Artifact pattern requires an artifact scrutinee, found {:?}",
                    scrutinee_value
                ),
                line_info.clone(),
            ));
        }
    };

    if env.get_artifact(type_name).is_none() {
        return Err(EvalError::InvalidOperation(
            format!("Artifact pattern references undefined type {}", type_name),
            line_info.clone(),
        ));
    }

    let actual_type = handle.borrow().type_name.clone();
    if actual_type != type_name {
        // Different artifact type — fall through so a sibling arm can
        // dispatch on the actual type (`Player {…} =>` vs `Enemy {…} =>`).
        return Ok(false);
    }

    let schema = lookup_schema_from_handle(env, &handle, line_info)?;
    let schema_field_names: Vec<String> = schema.field_names();
    let schema_name = schema.name.clone();

    for (field_name, sub_pattern) in fields {
        if !schema_field_names.iter().any(|n| n == field_name) {
            return Err(super::artifacts::missing_field_error(
                env.get_artifact(&schema_name).expect("schema present"),
                field_name,
                line_info,
            ));
        }

        // `read_artifact_field` re-validates the field against the schema and
        // returns a recoverable `EvalError` if the runtime value is missing
        // the field for any reason — preferable to the previous
        // `expect("schema-validated field must be present in artifact value")`
        // panic if a future malformed artifact slips through.
        let field_value = read_artifact_field(env, &handle, field_name, line_info)?;

        match sub_pattern {
            AST::OracleDontCareItem(_) => continue,
            AST::Var(name, var_line) => {
                let bound_type = type_of_scrutinee(&field_value);
                env.set_var(
                    name.clone(),
                    field_value,
                    bound_type,
                    false,
                    var_line.clone(),
                );
            }
            AST::OracleScrollPattern {
                elements,
                line_info: scroll_line,
            } => {
                if !match_scroll_pattern(elements, &field_value, scroll_line, env)? {
                    return Ok(false);
                }
            }
            AST::OracleArtifactPattern {
                type_name: nested_type,
                fields: nested_fields,
                line_info: nested_line,
            } => {
                if !match_artifact_pattern(
                    nested_type,
                    nested_fields,
                    &field_value,
                    nested_line,
                    env,
                )? {
                    return Ok(false);
                }
            }
            AST::OracleLexiconPattern {
                entries: nested_entries,
                line_info: nested_line,
            } => {
                if !match_lexicon_pattern(nested_entries, &field_value, nested_line, env)? {
                    return Ok(false);
                }
            }
            other => {
                // For literal-compare on an artifact field we want a deep,
                // type-aware equality so nested scrolls / lexicons / artifacts
                // compare by structure (matching the existing `==` semantics
                // in `eval/artifacts::values_equal`). The scroll-specific
                // `values_match_for_pattern` would only handle scalars and
                // emit a misleading "Scroll pattern element" error for any
                // non-scalar field.
                let pattern_result = evaluate(other, env)?;
                let pattern_value = match pattern_result {
                    EvalResult::Data(value) => value,
                    EvalResult::Artifact(handle) => Value::Artifact(handle),
                    EvalResult::Revealed(_) | EvalResult::Resume(_) | EvalResult::Eject(_) => {
                        return Err(EvalError::InvalidOperation(
                            "Artifact field pattern compare must yield a value".to_string(),
                            line_info.clone(),
                        ));
                    }
                };
                if !values_equal(env, &field_value, &pattern_value, line_info)? {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

/// Match a lexicon-shape pattern against a scrutinee `Value`, performing
/// any entry-level bindings into the caller's scope as side-effects.
///
/// Returns `Ok(true)` when the scrutinee is a lexicon whose listed entries
/// satisfy their sub-patterns (and any bindings have been applied).
/// Returns `Ok(false)` in two no-match cases:
///
/// - the scrutinee is a lexicon that lacks one of the listed keys —
///   missing keys are not an error, they just disqualify this arm so
///   sibling arms with different shapes can match;
/// - the scrutinee is a lexicon with all listed keys present but a
///   sub-pattern (literal compare, nested scroll/artifact/lexicon
///   pattern, etc.) does not match.
///
/// Returns `Err` when the scrutinee is not a lexicon at all. Empty
/// `{}` matches any lexicon (a "match by shape" catch-all), the same way
/// `Tag {}` matches any artifact of type `Tag`. Keys not mentioned in the
/// pattern are intentionally unrestricted.
fn match_lexicon_pattern(
    entries: &[(String, AST)],
    scrutinee_value: &Value,
    line_info: &Option<LineInfo>,
    env: &mut RuntimeEnv,
) -> Result<bool, EvalError> {
    let lexicon_handle = match scrutinee_value {
        Value::Lexicon(handle) => handle.clone(),
        _ => {
            return Err(EvalError::InvalidOperation(
                format!(
                    "Lexicon pattern requires a lexicon scrutinee, found {:?}",
                    scrutinee_value
                ),
                line_info.clone(),
            ));
        }
    };

    for (key, sub_pattern) in entries {
        // Snapshot the value out of the lexicon so we are not holding a
        // borrow across the recursive `match_*` calls (which themselves
        // may borrow the same handle, e.g. for nested lexicon patterns).
        let entry_value = match lexicon_handle.borrow().get(key).cloned() {
            Some(value) => value,
            None => return Ok(false),
        };

        match sub_pattern {
            AST::OracleDontCareItem(_) => continue,
            AST::Var(name, var_line) => {
                let bound_type = type_of_scrutinee(&entry_value);
                env.set_var(
                    name.clone(),
                    entry_value,
                    bound_type,
                    false,
                    var_line.clone(),
                );
            }
            AST::OracleScrollPattern {
                elements,
                line_info: scroll_line,
            } => {
                if !match_scroll_pattern(elements, &entry_value, scroll_line, env)? {
                    return Ok(false);
                }
            }
            AST::OracleArtifactPattern {
                type_name: nested_type,
                fields: nested_fields,
                line_info: nested_line,
            } => {
                if !match_artifact_pattern(
                    nested_type,
                    nested_fields,
                    &entry_value,
                    nested_line,
                    env,
                )? {
                    return Ok(false);
                }
            }
            AST::OracleLexiconPattern {
                entries: nested_entries,
                line_info: nested_line,
            } => {
                if !match_lexicon_pattern(nested_entries, &entry_value, nested_line, env)? {
                    return Ok(false);
                }
            }
            other => {
                // Same deep-equality strategy as `match_artifact_pattern`'s
                // literal-compare path so non-scalar entry values compare
                // structurally and match the runtime `==` semantics.
                let pattern_result = evaluate(other, env)?;
                let pattern_value = match pattern_result {
                    EvalResult::Data(value) => value,
                    EvalResult::Artifact(handle) => Value::Artifact(handle),
                    EvalResult::Revealed(_) | EvalResult::Resume(_) | EvalResult::Eject(_) => {
                        return Err(EvalError::InvalidOperation(
                            "Lexicon entry pattern compare must yield a value".to_string(),
                            line_info.clone(),
                        ));
                    }
                };
                if !values_equal(env, &entry_value, &pattern_value, line_info)? {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

/// Equality check shared by scroll-element matching: returns `true` when
/// `actual` (a scrutinee element value) equals `expected` (a freshly-evaluated
/// pattern expression result). Mismatched types raise an
/// `Invalid operation: Scroll pattern element type must match scrutinee
/// element type` error — analogous to the tuple-pattern path's
/// `Oracle branch pattern type must match scrutinee type` but worded for the
/// scroll-element context, so a heterogeneous scroll pattern fails loudly
/// rather than silently treating a type mismatch as "not equal".
fn values_match_for_pattern(
    actual: &Value,
    expected: &EvalResult,
    line_info: &Option<LineInfo>,
) -> Result<bool, EvalError> {
    match (actual, expected) {
        (Value::Arcana(a), EvalResult::Data(Value::Arcana(b))) => Ok(a == b),
        (Value::Aether(a), EvalResult::Data(Value::Aether(b))) => Ok((*a - b).abs() < f64::EPSILON),
        (Value::Rune(a), EvalResult::Data(Value::Rune(b))) => Ok(a.as_ref() == b.as_ref()),
        (Value::Omen(a), EvalResult::Data(Value::Omen(b))) => Ok(a == b),
        _ => Err(EvalError::InvalidOperation(
            "Scroll pattern element type must match scrutinee element type".to_string(),
            line_info.clone(),
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
    use std::rc::Rc;

    fn line() -> Option<LineInfo> {
        Some(LineInfo::new(1, 1))
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

    fn rune(text: &str) -> Value {
        Value::Rune(Rc::new(text.to_string()))
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

    fn register_artifact(env: &mut RuntimeEnv, name: &str, fields: Vec<(&str, Type)>) {
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
        let mut env = RuntimeEnv::new();
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
        let mut env = RuntimeEnv::new();
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
        let mut env = RuntimeEnv::new();
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
    fn field_assignment_requires_artifact_target() {
        let mut env = RuntimeEnv::new();
        let assignment = AST::FieldAssignment {
            target: Box::new(AST::Arcana(1, line())),
            field: "power".into(),
            value: Box::new(AST::Arcana(2, line())),
            line_info: line(),
        };

        let err = evaluate(&assignment, &mut env).expect_err("non artifact target should fail");
        match err {
            EvalError::InvalidOperation(_, _) => {}
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn field_assignment_reports_missing_variable() {
        let mut env = RuntimeEnv::new();
        let assignment = AST::FieldAssignment {
            target: Box::new(AST::Var("missing".into(), line())),
            field: "power".into(),
            value: Box::new(AST::Arcana(1, line())),
            line_info: line(),
        };

        let err = evaluate(&assignment, &mut env).expect_err("missing variable should error");
        match err {
            EvalError::UndefinedVariable(name, _) => assert_eq!(name, "missing"),
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn field_assignment_rejects_non_artifact_chain_segments() {
        let mut env = RuntimeEnv::new();
        register_artifact(&mut env, "Glyph", vec![("power", Type::Arcana)]);
        register_artifact(
            &mut env,
            "Sigil",
            vec![("core", Type::Artifact("Glyph".into()))],
        );

        let outer = artifact_handle("Sigil", vec![("core", Value::Arcana(7))]);
        env.set_var(
            "sigil".into(),
            Value::Artifact(outer),
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

        let err = evaluate(&assignment, &mut env).expect_err("non artifact segment should error");
        match err {
            EvalError::InvalidOperation(message, _) => {
                assert!(
                    message.contains("Field 'core' is not an artifact"),
                    "{}",
                    message
                )
            }
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn field_assignment_updates_nested_artifact_fields() {
        let mut env = RuntimeEnv::new();
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
        let mut env = RuntimeEnv::new();
        let conditional = ConditionalAssignment {
            variable: "sigil".into(),
            expression: Box::new(AST::Arcana(1, line())),
            line_info: line(),
        };

        let branch = AST::OracleBranch {
            pattern: vec![AST::Arcana(1, line())],
            guard: None,
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
    fn oracle_match_handles_aether_and_rune_patterns() {
        let mut env = RuntimeEnv::new();
        let conditionals = vec![
            ConditionalAssignment {
                variable: "flux".into(),
                expression: Box::new(AST::Aether(1.5, line())),
                line_info: line(),
            },
            ConditionalAssignment {
                variable: "word".into(),
                expression: Box::new(AST::Rune("moon".into(), line())),
                line_info: line(),
            },
        ];

        let branch = AST::OracleBranch {
            pattern: vec![AST::Aether(1.5, line()), AST::Rune("moon".into(), line())],
            guard: None,
            body: Box::new(AST::Arcana(7, line())),
            line_info: line(),
        };

        let oracle = AST::Oracle {
            is_match: true,
            conditionals,
            branches: vec![branch],
            line_info: line(),
        };

        let result = evaluate(&oracle, &mut env).expect("oracle should match scalars");
        match result {
            EvalResult::Data(Value::Arcana(value)) => assert_eq!(value, 7),
            other => panic!("unexpected oracle result {:?}", other),
        }
    }

    #[test]
    fn oracle_if_else_pattern_requires_omen_values() {
        let mut env = RuntimeEnv::new();
        let pattern_branch = AST::OracleBranch {
            pattern: vec![AST::Arcana(1, line())],
            guard: None,
            body: Box::new(AST::Arcana(0, line())),
            line_info: line(),
        };

        let oracle = AST::Oracle {
            is_match: false,
            conditionals: vec![],
            branches: vec![pattern_branch],
            line_info: line(),
        };

        let err = evaluate(&oracle, &mut env).expect_err("if-else mode patterns must yield omens");
        match err {
            EvalError::InvalidOperation(message, _) => {
                assert!(
                    message.contains("Oracle if-else pattern must evaluate to an omen"),
                    "{}",
                    message
                );
            }
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn oracle_pattern_length_mismatch_errors() {
        let mut env = RuntimeEnv::new();
        let conditional = ConditionalAssignment {
            variable: "sigil".into(),
            expression: Box::new(AST::Arcana(1, line())),
            line_info: line(),
        };

        let branch = AST::OracleBranch {
            pattern: vec![AST::Arcana(1, line()), AST::Arcana(2, line())],
            guard: None,
            body: Box::new(AST::Arcana(0, line())),
            line_info: line(),
        };

        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![conditional],
            branches: vec![branch],
            line_info: line(),
        };

        let err = evaluate(&oracle, &mut env).expect_err("pattern length mismatch should fail");
        match err {
            EvalError::InvalidOperation(message, _) => {
                assert!(message.contains("pattern length"), "{}", message)
            }
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn oracle_skips_comments_and_supports_dont_care_items() {
        let mut env = RuntimeEnv::new();
        let conditional = ConditionalAssignment {
            variable: "arc".into(),
            expression: Box::new(AST::Arcana(99, line())),
            line_info: line(),
        };

        let branch = AST::OracleBranch {
            pattern: vec![AST::OracleDontCareItem(line())],
            guard: None,
            body: Box::new(AST::Arcana(5, line())),
            line_info: line(),
        };

        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![conditional],
            branches: vec![AST::Comment("ignored".into(), line()), branch],
            line_info: line(),
        };

        let result = evaluate(&oracle, &mut env).expect("dont care branch should match");
        match result {
            EvalResult::Data(Value::Arcana(value)) => assert_eq!(value, 5),
            other => panic!("unexpected oracle result {:?}", other),
        }
    }

    #[test]
    fn oracle_error_paths_do_not_leak_scope() {
        // Regression for the v0.5.0 PR1 review: every error path inside the
        // oracle evaluator must pop the scope it pushed on entry, otherwise
        // the REPL leaks a scope each time. We exercise each error site in
        // turn against a fresh `RuntimeEnv` (which starts at depth 1), and
        // assert the depth returns to 1 after the evaluator yields `Err`.

        // 1. Scrutinee evaluates to an unsupported type (Abyss is not in the
        //    accepted Arcana / Aether / Rune / Omen list).
        let mut env = RuntimeEnv::new();
        assert_eq!(env.scope_depth(), 1);
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Abyss(line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line())],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("scrutinee type error");
        assert_eq!(env.scope_depth(), 1, "scrutinee error leaked a scope");

        // 2. Pattern length mismatch (1 scrutinee vs 2-element pattern).
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Arcana(1, line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line()), AST::Arcana(2, line())],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("pattern length error");
        assert_eq!(env.scope_depth(), 1, "pattern length error leaked a scope");

        // 3. Pattern type mismatch (Arcana scrutinee vs Rune pattern).
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Arcana(1, line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Rune("x".into(), line())],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("pattern type error");
        assert_eq!(env.scope_depth(), 1, "pattern type error leaked a scope");

        // 4. If-else mode pattern that does not yield an omen.
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: false,
            conditionals: vec![],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line())],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("if-else mode pattern type error");
        assert_eq!(
            env.scope_depth(),
            1,
            "if-else pattern type error leaked a scope"
        );

        // 5. Ward expression evaluates to a non-omen.
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Arcana(1, line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line())],
                guard: Some(Box::new(AST::Arcana(42, line()))),
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("ward type error");
        assert_eq!(env.scope_depth(), 1, "ward type error leaked a scope");

        // 6. Body raises an error after the pattern matched (undefined var).
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Arcana(1, line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line())],
                guard: None,
                body: Box::new(AST::Var("missing".into(), line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("body undefined-variable error");
        assert_eq!(env.scope_depth(), 1, "body error leaked a scope");
    }

    #[test]
    fn oracle_question_mark_propagation_does_not_leak_scope() {
        // The four `?`-propagated error sites that motivated the refactor —
        // scrutinee evaluation, the match-mode pattern loop, the if-else-mode
        // pattern loop, and the new ward expression — must also unwind the
        // pushed scope. The cases above hit the explicit `return Err(...)`
        // arms; these specifically exercise the `?` operator by feeding a
        // sub-expression that fails to evaluate (an undefined variable, which
        // surfaces `EvalError::UndefinedVariable`).

        // A. Scrutinee expression itself fails — exercises
        //    `evaluate(&conditional.expression, env)?`.
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Var("missing".into(), line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line())],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("scrutinee var lookup error");
        assert_eq!(
            env.scope_depth(),
            1,
            "scrutinee `?` propagation leaked a scope"
        );

        // B. Match-mode pattern expression fails — exercises
        //    `evaluate(pattern, env)?` inside the pattern loop. We use
        //    `AST::Add(Var("missing"), Arcana(1))` here rather than a bare
        //    `AST::Var("missing", _)` because, after PR2's binding-pattern
        //    work, a bare identifier in match-mode pattern position is
        //    intercepted as a fresh binding and never reaches `evaluate`.
        //    Wrapping the missing identifier inside an `Add` keeps the
        //    pattern an expression so the inner `evaluate` actually runs and
        //    can raise `UndefinedVariable`.
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Arcana(1, line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Add(
                    Box::new(AST::Var("missing".into(), line())),
                    Box::new(AST::Arcana(1, line())),
                    line(),
                )],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("match-mode pattern var lookup error");
        assert_eq!(
            env.scope_depth(),
            1,
            "match-mode pattern `?` propagation leaked a scope"
        );

        // C. If-else-mode pattern expression fails — exercises
        //    `evaluate(pattern_expr, env)?` inside the all-true loop.
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: false,
            conditionals: vec![],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Var("missing".into(), line())],
                guard: None,
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("if-else-mode pattern var lookup error");
        assert_eq!(
            env.scope_depth(),
            1,
            "if-else-mode pattern `?` propagation leaked a scope"
        );

        // D. Ward expression itself fails — exercises
        //    `evaluate(guard_expr.as_ref(), env)?` (the original bug site
        //    from PR #414, now folded into the central pop_scope).
        let mut env = RuntimeEnv::new();
        let oracle = AST::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(AST::Arcana(1, line())),
                line_info: line(),
            }],
            branches: vec![AST::OracleBranch {
                pattern: vec![AST::Arcana(1, line())],
                guard: Some(Box::new(AST::Var("missing".into(), line()))),
                body: Box::new(AST::Arcana(0, line())),
                line_info: line(),
            }],
            line_info: line(),
        };
        evaluate(&oracle, &mut env).expect_err("ward var lookup error");
        assert_eq!(env.scope_depth(), 1, "ward `?` propagation leaked a scope");
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
    fn clone_indexed_child_reports_scroll_bounds() {
        let value = scroll(vec![Value::Arcana(0)]);
        let err = clone_indexed_child(&value, &EvalResult::data(Value::Arcana(5)), &line())
            .expect_err("out of bounds should fail");
        match err {
            EvalError::InvalidOperation(message, _) => {
                assert!(message.contains("out of bounds"), "{}", message)
            }
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn clone_indexed_child_reports_missing_lexicon_entries() {
        let value = lexicon(vec![("known", Value::Arcana(1))]);
        let err = clone_indexed_child(&value, &EvalResult::data(rune("missing")), &line())
            .expect_err("missing key should fail");
        match err {
            EvalError::InvalidOperation(message, _) => {
                assert!(message.contains("does not exist"), "{}", message)
            }
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn artifact_definition_creates_glyph_variable() {
        let mut env = RuntimeEnv::new();
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
