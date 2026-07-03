use crate::env::{ArtifactMethod, Callable, EngravedFunction, RuntimeEnv, Value};
use abyss_core::ast::{AssignmentOp, Span, Stmt, Type};
use std::rc::Rc;

use super::artifacts::{
    build_artifact_schema, collect_field_chain, ensure_field_exists, ensure_type_known,
    expect_artifact_handle, lookup_schema_from_handle, missing_field_error,
};
use super::collections::{collect_index_chain, expect_arcana_index, expect_rune_key};
use super::expressions::evaluate_expr;
use super::result::{EvalError, EvalResult};
use super::values::{
    convert_to_typed_value, describe_value, eval_result_to_value_checked, extract_aether,
    extract_arcana, extract_omen, extract_rune,
};

/// Evaluates a statement in the given environment.
///
/// Bare expressions in statement position arrive as [`Stmt::Expr`] and
/// delegate to [`evaluate_expr`]; there is no longer a shared node type
/// between the two, so each evaluator states its input in its signature.
pub fn evaluate(stmt: &Stmt, env: &mut RuntimeEnv) -> Result<EvalResult, EvalError> {
    match stmt {
        Stmt::Expr(expr, _span) => evaluate_expr(expr, env),
        Stmt::VarAssign {
            name,
            value,
            var_type,
            is_morph,
            span: line_info,
        } => {
            ensure_type_known(var_type, env, line_info)?;
            let evaluated_value = evaluate_expr(value, env)?;
            let stored_value = convert_to_typed_value(evaluated_value, var_type, line_info)?;
            env.set_var(
                name.clone(),
                stored_value,
                var_type.clone(),
                *is_morph,
                *line_info,
            );
            Ok(EvalResult::abyss())
        }
        Stmt::Assignment {
            name,
            value,
            op,
            span: line_info,
        } => {
            let evaluated_value = evaluate_expr(value, env)?;
            if let Some(var_info) = env.get_var_mut(name) {
                if !var_info.is_morph {
                    return Err(EvalError::ImmutableAssignment(name.clone(), *line_info));
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
                                    return Err(EvalError::NegativeExponent(*line_info));
                                }
                                current.pow(exponent as u32)
                            }
                            AssignmentOp::Assign => extract_arcana(&evaluated_value, line_info)?,
                            _ => {
                                return Err(EvalError::InvalidOperation(
                                    format!("Unsupported operation for variable {}", name),
                                    *line_info,
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
                                    *line_info,
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
                                *line_info,
                            ));
                        }
                    },
                    (Value::Omen(current), Type::Omen) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported operation for variable {}", name),
                                *line_info,
                            ));
                        }
                        *current = extract_omen(&evaluated_value, line_info)?;
                    }
                    (Value::Abyss, Type::Abyss) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                format!("Unsupported operation for variable {}", name),
                                *line_info,
                            ));
                        }
                        if !matches!(evaluated_value, EvalResult::Data(Value::Abyss)) {
                            return Err(EvalError::ExpectedType(Type::Abyss, *line_info));
                        }
                    }
                    (value_slot, Type::Scroll) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                "Scroll reassignment only supports =".to_string(),
                                *line_info,
                            ));
                        }
                        *value_slot =
                            convert_to_typed_value(evaluated_value, &Type::Scroll, line_info)?;
                    }
                    (value_slot, Type::Lexicon) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                "Lexicon reassignment only supports =".to_string(),
                                *line_info,
                            ));
                        }
                        *value_slot =
                            convert_to_typed_value(evaluated_value, &Type::Lexicon, line_info)?;
                    }
                    (value_slot, Type::Materia) => {
                        if !matches!(op, AssignmentOp::Assign) {
                            return Err(EvalError::InvalidOperation(
                                "Materia variables only support =".to_string(),
                                *line_info,
                            ));
                        }
                        *value_slot = eval_result_to_value_checked(evaluated_value, *line_info)?;
                    }
                    _ => {
                        return Err(EvalError::InvalidOperation(
                            format!(
                                "Type mismatch or unsupported operation for variable {}",
                                name
                            ),
                            *line_info,
                        ));
                    }
                }

                Ok(EvalResult::abyss())
            } else {
                Err(env.undefined_variable_error(name, *line_info))
            }
        }
        Stmt::IndexAssignment {
            target,
            index,
            value,
            span: line_info,
        } => {
            let (base_name, nested_indices) = collect_index_chain(target).ok_or_else(|| {
                EvalError::InvalidOperation(
                    "Indexed assignment requires a mutable variable target".to_string(),
                    *line_info,
                )
            })?;

            let mut evaluated_indices = Vec::new();
            for idx_ast in nested_indices {
                evaluated_indices.push(evaluate_expr(idx_ast, env)?);
            }

            let final_index_value = evaluate_expr(index, env)?;
            let new_value = eval_result_to_value_checked(evaluate_expr(value, env)?, *line_info)?;

            let var_info = match env.get_var_mut(&base_name) {
                Some(var_info) => var_info,
                None => {
                    return Err(env.undefined_variable_error(&base_name, *line_info));
                }
            };

            if !var_info.is_morph {
                return Err(EvalError::ImmutableAssignment(
                    base_name.clone(),
                    *line_info,
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
                        return Err(EvalError::ScrollIndexOutOfBounds(idx, *line_info));
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
                        *line_info,
                    ));
                }
            }

            Ok(EvalResult::abyss())
        }
        Stmt::FieldAssignment {
            target,
            field,
            value,
            span: line_info,
        } => {
            let (base_name, access_chain) = collect_field_chain(target).ok_or_else(|| {
                EvalError::InvalidOperation(
                    "Field assignment requires an artifact variable".to_string(),
                    *line_info,
                )
            })?;

            let evaluated_value = evaluate_expr(value, env)?;

            let var_info = match env.get_var_mut(&base_name) {
                Some(var_info) => var_info,
                None => {
                    return Err(env.undefined_variable_error(&base_name, *line_info));
                }
            };

            if !var_info.is_morph {
                return Err(EvalError::ImmutableAssignment(
                    base_name.clone(),
                    *line_info,
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
                            *line_info,
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
        Stmt::Reveal(expr, _span) => {
            let result = evaluate_expr(expr, env)?;
            Ok(EvalResult::Revealed(Box::new(result)))
        }
        Stmt::Block(statements, _span) => {
            let mut last_result = EvalResult::abyss();
            for statement in statements {
                let result = evaluate(statement, env)?;

                match result {
                    EvalResult::Revealed(revealed) => return Ok(*revealed),
                    EvalResult::Revolve(_) | EvalResult::Eject(_) => return Ok(result),
                    _ => {}
                }

                last_result = result;
            }
            Ok(last_result)
        }
        Stmt::Orbit {
            params,
            body,
            span: line_info,
        } => {
            if params.is_empty() {
                loop {
                    env.push_scope();

                    let result = evaluate(body, env)?;

                    match result {
                        EvalResult::Revolve(_) => continue,
                        EvalResult::Eject(_) => break,
                        _ => {}
                    }

                    env.pop_scope();
                }

                Ok(EvalResult::abyss())
            } else {
                let param = &params[0];
                let (name, start, end, op) = (&param.name, &param.start, &param.end, &param.op);
                let start_value = evaluate_expr(start, env)?;
                let end_value = evaluate_expr(end, env)?;

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
                        *line_info,
                    );

                    let remaining_params = params[1..].to_vec();
                    let result = if remaining_params.is_empty() {
                        evaluate(body.as_ref(), env)?
                    } else {
                        evaluate(
                            &Stmt::Orbit {
                                params: remaining_params,
                                body: body.clone(),
                                span: *line_info,
                            },
                            env,
                        )?
                    };

                    match result {
                        EvalResult::Revolve(identifier) => {
                            if let Some(id) = identifier {
                                if id == *name {
                                    continue;
                                } else {
                                    env.pop_scope();
                                    return Ok(EvalResult::Revolve(Some(id)));
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
            }
        }
        Stmt::Revolve(identifier, _span) => Ok(EvalResult::Revolve(identifier.clone())),
        Stmt::Eject(identifier, _span) => Ok(EvalResult::Eject(identifier.clone())),
        Stmt::Engrave {
            name,
            params,
            return_type,
            body,
            method_target,
            span: line_info,
        } => {
            ensure_type_known(return_type, env, line_info)?;
            for param in params {
                ensure_type_known(&param.param_type, env, &param.span)?;
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
                line_info: *line_info,
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
        Stmt::ArtifactDef {
            name,
            fields,
            span: line_info,
        } => {
            if env.artifact_defined_in_current_scope(name) {
                return Err(EvalError::InvalidOperation(
                    format!("Artifact {} is already defined", name),
                    *line_info,
                ));
            }
            let schema = build_artifact_schema(name, fields, env, line_info)?;
            env.define_artifact(schema)?;
            env.set_var(
                name.clone(),
                Value::Glyph(Type::Artifact(name.clone())),
                Type::Glyph,
                false,
                *line_info,
            );
            Ok(EvalResult::abyss())
        }
        Stmt::Comment(_, _) => Ok(EvalResult::abyss()),
    }
}

fn clone_indexed_child(
    value: &Value,
    index: &EvalResult,
    line_info: &Option<Span>,
) -> Result<Value, EvalError> {
    match value {
        Value::Scroll(handle) => {
            let idx = expect_arcana_index(index, line_info)?;
            let borrowed = handle.borrow();
            borrowed
                .get(idx)
                .cloned()
                .ok_or(EvalError::ScrollIndexOutOfBounds(idx, *line_info))
        }
        Value::Lexicon(handle) => {
            let key = expect_rune_key(index, line_info)?;
            let borrowed = handle.borrow();
            borrowed
                .get(&key)
                .cloned()
                .ok_or_else(|| EvalError::MissingLexiconKey(key.clone(), *line_info))
        }
        _ => Err(EvalError::InvalidOperation(
            "Cannot index into non-collection value".to_string(),
            *line_info,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_support::{artifact_handle, lexicon, register_artifact, rune, scroll};
    use abyss_core::ast::Expr;

    fn line() -> Option<Span> {
        Some(Span::new(1, 1))
    }

    #[test]
    fn arcana_assignment_supports_compound_ops() {
        let mut env = RuntimeEnv::new();
        env.set_var("sigil".into(), Value::Arcana(2), Type::Arcana, true, line());

        let assignment = Stmt::Assignment {
            name: "sigil".into(),
            value: Expr::Arcana(5, line()),
            op: AssignmentOp::AddAssign,
            span: line(),
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

        let assignment = Stmt::Assignment {
            name: "sigil".into(),
            value: Expr::Arcana(5, line()),
            op: AssignmentOp::Assign,
            span: line(),
        };

        let err = evaluate(&assignment, &mut env).expect_err("immutable reassign should fail");
        match err {
            EvalError::ImmutableAssignment(name, _) => assert_eq!(name, "sigil"),
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn index_assignment_rejects_immutable_variables() {
        let mut env = RuntimeEnv::new();
        env.set_var(
            "scroll".into(),
            scroll(vec![Value::Arcana(0)]),
            Type::Scroll,
            false,
            line(),
        );

        let index_assignment = Stmt::IndexAssignment {
            target: Expr::Var("scroll".into(), line()),
            index: Expr::Arcana(0, line()),
            value: Expr::Arcana(99, line()),
            span: line(),
        };

        let err = evaluate(&index_assignment, &mut env).expect_err("immutable index assignment");
        assert!(matches!(err, EvalError::ImmutableAssignment(name, _) if name == "scroll"));
    }

    #[test]
    fn field_assignment_rejects_immutable_variables() {
        let mut env = RuntimeEnv::new();
        register_artifact(&mut env, "Relic", vec![("power", Type::Arcana)]);
        env.set_var(
            "relic".into(),
            Value::Artifact(artifact_handle("Relic", vec![("power", Value::Arcana(1))])),
            Type::Artifact("Relic".into()),
            false,
            line(),
        );

        let field_assignment = Stmt::FieldAssignment {
            target: Expr::Var("relic".into(), line()),
            field: "power".into(),
            value: Expr::Arcana(2, line()),
            span: line(),
        };

        let err = evaluate(&field_assignment, &mut env).expect_err("immutable field assignment");
        assert!(matches!(err, EvalError::ImmutableAssignment(name, _) if name == "relic"));
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

        let index_assignment = Stmt::IndexAssignment {
            target: Expr::Var("scroll".into(), line()),
            index: Expr::Arcana(1, line()),
            value: Expr::Arcana(99, line()),
            span: line(),
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
        let assignment = Stmt::FieldAssignment {
            target: Expr::Arcana(1, line()),
            field: "power".into(),
            value: Expr::Arcana(2, line()),
            span: line(),
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
        let assignment = Stmt::FieldAssignment {
            target: Expr::Var("missing".into(), line()),
            field: "power".into(),
            value: Expr::Arcana(1, line()),
            span: line(),
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

        let target = Expr::FieldAccess {
            target: Box::new(Expr::Var("sigil".into(), line())),
            field: "core".into(),
            span: line(),
        };
        let assignment = Stmt::FieldAssignment {
            target,
            field: "power".into(),
            value: Expr::Arcana(10, line()),
            span: line(),
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

        let target = Expr::FieldAccess {
            target: Box::new(Expr::Var("sigil".into(), line())),
            field: "core".into(),
            span: line(),
        };
        let assignment = Stmt::FieldAssignment {
            target,
            field: "power".into(),
            value: Expr::Arcana(10, line()),
            span: line(),
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
            EvalError::ScrollIndexOutOfBounds(index, _) => assert_eq!(index, 5),
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn clone_indexed_child_reports_missing_lexicon_entries() {
        let value = lexicon(vec![("known", Value::Arcana(1))]);
        let err = clone_indexed_child(&value, &EvalResult::data(rune("missing")), &line())
            .expect_err("missing key should fail");
        match err {
            EvalError::MissingLexiconKey(key, _) => assert_eq!(key, "missing"),
            other => panic!("unexpected error variant {:?}", other),
        }
    }

    #[test]
    fn artifact_definition_creates_glyph_variable() {
        let mut env = RuntimeEnv::new();
        let artifact = Stmt::ArtifactDef {
            name: "Relic".into(),
            fields: Vec::new(),
            span: line(),
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
