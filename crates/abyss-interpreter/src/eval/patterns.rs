//! Oracle evaluation and pattern matching.
//!
//! Hosts the `Expr::Oracle` machinery: scrutinee evaluation, per-branch
//! scope handling, and the three destructuring shapes (scroll, artifact,
//! lexicon) with their shared literal-compare fallback.
//! [`evaluate_oracle`] is the only entry point; the caller (the
//! `Expr::Oracle` arm in [`super::expressions::evaluate_expr`]) pushes
//! the outer oracle scope before calling and pops it after.

use std::cell::RefCell;
use std::rc::Rc;

use crate::env::{RuntimeEnv, Value};
use abyss_core::ast::{ConditionalAssignment, Expr, OracleBranch, Pattern, Span, Stmt, Type};

use super::artifacts::{lookup_schema_from_handle, read_artifact_field, values_equal};
use super::expressions::evaluate_expr;
use super::result::{EvalError, EvalResult};
use super::statements::evaluate;

/// Inner body of `Expr::Oracle` evaluation. The caller is responsible for
/// pairing one `env.push_scope()` before this call with one `env.pop_scope()`
/// after, so every `?` inside this helper unwinds through the caller and the
/// outer oracle scope is always popped.
///
/// Each branch additionally gets its own nested scope (push/pop) so that
/// match-mode bindings (`(x) =>`) and any `forge` declarations in the body
/// stay confined to the arm that produced them — without leaking sideways
/// to subsequent arms or upward to the script.
pub(super) fn evaluate_oracle(
    is_match: bool,
    conditionals: &[ConditionalAssignment],
    branches: &[OracleBranch],
    line_info: &Option<Span>,
    env: &mut RuntimeEnv,
) -> Result<EvalResult, EvalError> {
    let mut scrutinee_values = Vec::with_capacity(conditionals.len());
    for conditional in conditionals {
        let result = evaluate_expr(&conditional.expression, env)?;
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
            // in.
            EvalResult::Data(Value::Artifact(handle)) => Value::Artifact(handle.clone()),
            // Lexicons flow through as their shared handle so a lexicon
            // pattern arm sees the same entries the user passed in. Mutating
            // the lexicon inside the arm body therefore visibly mutates the
            // outer value, matching the existing aliasing semantics.
            EvalResult::Data(Value::Lexicon(handle)) => Value::Lexicon(handle.clone()),
            other => {
                return Err(EvalError::InvalidOperation(
                    format!("Unsupported type in oracle scrutinee: {:?}", other),
                    *line_info,
                ));
            }
        };
        scrutinee_values.push(stored);
    }

    for branch in branches {
        env.push_scope();
        let outcome = evaluate_oracle_branch(
            is_match,
            &branch.pattern,
            branch.guard.as_ref(),
            &branch.body,
            &branch.span,
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
    pattern: &[Pattern],
    guard: Option<&Expr>,
    body: &Stmt,
    line_info: &Option<Span>,
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
                *line_info,
            ));
        }

        let mut matched = true;
        for (idx, pattern_elem) in pattern.iter().enumerate() {
            if let Pattern::DontCare(_) = pattern_elem {
                continue;
            }

            let Some(scrutinee_value) = scrutinee_values.get(idx) else {
                return Err(EvalError::InvalidOperation(
                    "Oracle branch references missing scrutinee".to_string(),
                    *line_info,
                ));
            };

            // A bare identifier in match-mode pattern position introduces a
            // fresh binding to the scrutinee value (rather than looking the
            // identifier up as an expression). The binding lives in the
            // per-branch scope the caller pushed, so it is visible to the
            // ward and body of this arm and disappears when the arm finishes.
            if let Pattern::Expr(Expr::Var(name, var_line)) = pattern_elem {
                env.set_var(
                    name.clone(),
                    scrutinee_value.clone(),
                    type_of_scrutinee(scrutinee_value),
                    false,
                    *var_line,
                );
                continue;
            }

            // Scroll-shape pattern destructures the scrutinee — which must be
            // a `scroll` — into its elements, with optional trailing rest.
            if let Pattern::Scroll {
                elements,
                span: scroll_line,
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
            if let Pattern::Artifact {
                type_name,
                fields,
                span: artifact_line,
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
            if let Pattern::Lexicon {
                entries,
                span: lexicon_line,
            } = pattern_elem
            {
                if !match_lexicon_pattern(entries, scrutinee_value, lexicon_line, env)? {
                    matched = false;
                    break;
                }
                continue;
            }

            let Pattern::Expr(pattern_expr) = pattern_elem else {
                // Only a trailing rest segment reaches here, and it is
                // meaningless outside a scroll pattern.
                return Err(EvalError::InvalidOperation(
                    "Rest segment is only valid inside a scroll pattern".to_string(),
                    *line_info,
                ));
            };
            let pattern_result = evaluate_expr(pattern_expr, env)?;

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
                        *line_info,
                    ));
                }
            }
        }
        matched
    } else {
        let mut all_true = true;
        for pattern_elem in pattern {
            let Pattern::Expr(pattern_expr) = pattern_elem else {
                return Err(EvalError::InvalidOperation(
                    "Oracle if-else pattern must be an expression".to_string(),
                    *line_info,
                ));
            };
            match evaluate_expr(pattern_expr, env)? {
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
                        *line_info,
                    ));
                }
            }
        }
        all_true
    };

    let matched = if matched {
        match guard {
            None => true,
            Some(guard_expr) => match evaluate_expr(guard_expr, env)? {
                EvalResult::Data(Value::Omen(b)) => b,
                other => {
                    return Err(EvalError::InvalidOperation(
                        format!("Oracle ward must evaluate to an omen, found {:?}", other),
                        *line_info,
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
    elements: &[Pattern],
    scrutinee_value: &Value,
    line_info: &Option<Span>,
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
                *line_info,
            ));
        }
    };

    let scroll_values: Vec<Value> = scroll_handle.borrow().clone();

    let mut rest_index: Option<usize> = None;
    for (idx, element) in elements.iter().enumerate() {
        if matches!(element, Pattern::Rest { .. }) {
            if rest_index.is_some() {
                return Err(EvalError::InvalidOperation(
                    "Scroll pattern may contain at most one rest segment".to_string(),
                    *line_info,
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
            *line_info,
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
            Pattern::DontCare(_) => continue,
            Pattern::Expr(Expr::Var(name, var_line)) => {
                env.set_var(
                    name.clone(),
                    elem_value.clone(),
                    type_of_scrutinee(elem_value),
                    false,
                    *var_line,
                );
            }
            Pattern::Scroll {
                elements: nested_elements,
                span: nested_line,
            } => {
                if !match_scroll_pattern(nested_elements, elem_value, nested_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Artifact {
                type_name,
                fields,
                span: artifact_line,
            } => {
                if !match_artifact_pattern(type_name, fields, elem_value, artifact_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Lexicon {
                entries,
                span: lexicon_line,
            } => {
                if !match_lexicon_pattern(entries, elem_value, lexicon_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Rest { .. } => {
                // Validated above: at most one rest, trailing only. A rest
                // inside the prefix loop is unreachable.
                unreachable!("rest segment handled after the prefix loop")
            }
            Pattern::Expr(other) => {
                let pattern_result = evaluate_expr(other, env)?;
                if !values_match_for_pattern(elem_value, &pattern_result, line_info)? {
                    return Ok(false);
                }
            }
        }
    }

    if let Some(idx) = rest_index
        && let Pattern::Rest {
            name: Some(name),
            span: rest_line,
        } = &elements[idx]
    {
        let tail: Vec<Value> = scroll_values.iter().skip(prefix_len).cloned().collect();
        let tail_handle: Rc<RefCell<Vec<Value>>> = Rc::new(RefCell::new(tail));
        env.set_var(
            name.clone(),
            Value::Scroll(tail_handle),
            Type::Scroll,
            false,
            *rest_line,
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
    fields: &[(String, Pattern)],
    scrutinee_value: &Value,
    line_info: &Option<Span>,
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
                *line_info,
            ));
        }
    };

    if env.get_artifact(type_name).is_none() {
        return Err(EvalError::InvalidOperation(
            format!("Artifact pattern references undefined type {}", type_name),
            *line_info,
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
            Pattern::DontCare(_) => continue,
            Pattern::Rest { .. } => {
                return Err(EvalError::InvalidOperation(
                    "Rest segment is only valid inside a scroll pattern".to_string(),
                    *line_info,
                ));
            }
            Pattern::Expr(Expr::Var(name, var_line)) => {
                let bound_type = type_of_scrutinee(&field_value);
                env.set_var(name.clone(), field_value, bound_type, false, *var_line);
            }
            Pattern::Scroll {
                elements,
                span: scroll_line,
            } => {
                if !match_scroll_pattern(elements, &field_value, scroll_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Artifact {
                type_name: nested_type,
                fields: nested_fields,
                span: nested_line,
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
            Pattern::Lexicon {
                entries: nested_entries,
                span: nested_line,
            } => {
                if !match_lexicon_pattern(nested_entries, &field_value, nested_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Expr(other) => {
                // For literal-compare on an artifact field we want a deep,
                // type-aware equality so nested scrolls / lexicons / artifacts
                // compare by structure (matching the existing `==` semantics
                // in `eval/artifacts::values_equal`). The scroll-specific
                // `values_match_for_pattern` would only handle scalars and
                // emit a misleading "Scroll pattern element" error for any
                // non-scalar field.
                let pattern_result = evaluate_expr(other, env)?;
                let pattern_value = match pattern_result {
                    EvalResult::Data(value) => value,
                    EvalResult::Revealed(_) | EvalResult::Revolve(_) | EvalResult::Eject(_) => {
                        return Err(EvalError::InvalidOperation(
                            "Artifact field pattern compare must yield a value".to_string(),
                            *line_info,
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
    entries: &[(String, Pattern)],
    scrutinee_value: &Value,
    line_info: &Option<Span>,
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
                *line_info,
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
            Pattern::DontCare(_) => continue,
            Pattern::Rest { .. } => {
                return Err(EvalError::InvalidOperation(
                    "Rest segment is only valid inside a scroll pattern".to_string(),
                    *line_info,
                ));
            }
            Pattern::Expr(Expr::Var(name, var_line)) => {
                let bound_type = type_of_scrutinee(&entry_value);
                env.set_var(name.clone(), entry_value, bound_type, false, *var_line);
            }
            Pattern::Scroll {
                elements,
                span: scroll_line,
            } => {
                if !match_scroll_pattern(elements, &entry_value, scroll_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Artifact {
                type_name: nested_type,
                fields: nested_fields,
                span: nested_line,
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
            Pattern::Lexicon {
                entries: nested_entries,
                span: nested_line,
            } => {
                if !match_lexicon_pattern(nested_entries, &entry_value, nested_line, env)? {
                    return Ok(false);
                }
            }
            Pattern::Expr(other) => {
                // Same deep-equality strategy as `match_artifact_pattern`'s
                // literal-compare path so non-scalar entry values compare
                // structurally and match the runtime `==` semantics.
                let pattern_result = evaluate_expr(other, env)?;
                let pattern_value = match pattern_result {
                    EvalResult::Data(value) => value,
                    EvalResult::Revealed(_) | EvalResult::Revolve(_) | EvalResult::Eject(_) => {
                        return Err(EvalError::InvalidOperation(
                            "Lexicon entry pattern compare must yield a value".to_string(),
                            *line_info,
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
    line_info: &Option<Span>,
) -> Result<bool, EvalError> {
    match (actual, expected) {
        (Value::Arcana(a), EvalResult::Data(Value::Arcana(b))) => Ok(a == b),
        (Value::Aether(a), EvalResult::Data(Value::Aether(b))) => Ok((*a - b).abs() < f64::EPSILON),
        (Value::Rune(a), EvalResult::Data(Value::Rune(b))) => Ok(a.as_ref() == b.as_ref()),
        (Value::Omen(a), EvalResult::Data(Value::Omen(b))) => Ok(a == b),
        _ => Err(EvalError::InvalidOperation(
            "Scroll pattern element type must match scrutinee element type".to_string(),
            *line_info,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Option<Span> {
        Some(Span::new(1, 1))
    }
    #[test]
    fn oracle_match_branch_returns_revealed_value() {
        let mut env = RuntimeEnv::new();
        let conditional = ConditionalAssignment {
            variable: "sigil".into(),
            expression: Box::new(Expr::Arcana(1, line())),
            span: line(),
        };

        let branch = OracleBranch {
            pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
            guard: None,
            body: Stmt::Reveal(Expr::Arcana(42, line()), line()),
            span: line(),
        };

        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![conditional],
            branches: vec![branch],
            span: line(),
        };

        let result = evaluate_expr(&oracle, &mut env).expect("oracle should succeed");
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
                expression: Box::new(Expr::Aether(1.5, line())),
                span: line(),
            },
            ConditionalAssignment {
                variable: "word".into(),
                expression: Box::new(Expr::Rune("moon".into(), line())),
                span: line(),
            },
        ];

        let branch = OracleBranch {
            pattern: vec![
                Pattern::Expr(Expr::Aether(1.5, line())),
                Pattern::Expr(Expr::Rune("moon".into(), line())),
            ],
            guard: None,
            body: Stmt::Expr(Expr::Arcana(7, line()), line()),
            span: line(),
        };

        let oracle = Expr::Oracle {
            is_match: true,
            conditionals,
            branches: vec![branch],
            span: line(),
        };

        let result = evaluate_expr(&oracle, &mut env).expect("oracle should match scalars");
        match result {
            EvalResult::Data(Value::Arcana(value)) => assert_eq!(value, 7),
            other => panic!("unexpected oracle result {:?}", other),
        }
    }

    #[test]
    fn oracle_if_else_pattern_requires_omen_values() {
        let mut env = RuntimeEnv::new();
        let pattern_branch = OracleBranch {
            pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
            guard: None,
            body: Stmt::Expr(Expr::Arcana(0, line()), line()),
            span: line(),
        };

        let oracle = Expr::Oracle {
            is_match: false,
            conditionals: vec![],
            branches: vec![pattern_branch],
            span: line(),
        };

        let err =
            evaluate_expr(&oracle, &mut env).expect_err("if-else mode patterns must yield omens");
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
            expression: Box::new(Expr::Arcana(1, line())),
            span: line(),
        };

        let branch = OracleBranch {
            pattern: vec![
                Pattern::Expr(Expr::Arcana(1, line())),
                Pattern::Expr(Expr::Arcana(2, line())),
            ],
            guard: None,
            body: Stmt::Expr(Expr::Arcana(0, line()), line()),
            span: line(),
        };

        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![conditional],
            branches: vec![branch],
            span: line(),
        };

        let err =
            evaluate_expr(&oracle, &mut env).expect_err("pattern length mismatch should fail");
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
            expression: Box::new(Expr::Arcana(99, line())),
            span: line(),
        };

        let branch = OracleBranch {
            pattern: vec![Pattern::DontCare(line())],
            guard: None,
            body: Stmt::Expr(Expr::Arcana(5, line()), line()),
            span: line(),
        };

        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![conditional],
            branches: vec![branch],
            span: line(),
        };

        let result = evaluate_expr(&oracle, &mut env).expect("dont care branch should match");
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
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Abyss(line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("scrutinee type error");
        assert_eq!(env.scope_depth(), 1, "scrutinee error leaked a scope");

        // 2. Pattern length mismatch (1 scrutinee vs 2-element pattern).
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Arcana(1, line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![
                    Pattern::Expr(Expr::Arcana(1, line())),
                    Pattern::Expr(Expr::Arcana(2, line())),
                ],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("pattern length error");
        assert_eq!(env.scope_depth(), 1, "pattern length error leaked a scope");

        // 3. Pattern type mismatch (Arcana scrutinee vs Rune pattern).
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Arcana(1, line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Rune("x".into(), line()))],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("pattern type error");
        assert_eq!(env.scope_depth(), 1, "pattern type error leaked a scope");

        // 4. If-else mode pattern that does not yield an omen.
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: false,
            conditionals: vec![],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("if-else mode pattern type error");
        assert_eq!(
            env.scope_depth(),
            1,
            "if-else pattern type error leaked a scope"
        );

        // 5. Ward expression evaluates to a non-omen.
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Arcana(1, line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
                guard: Some(Expr::Arcana(42, line())),
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("ward type error");
        assert_eq!(env.scope_depth(), 1, "ward type error leaked a scope");

        // 6. Body raises an error after the pattern matched (undefined var).
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Arcana(1, line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
                guard: None,
                body: Stmt::Expr(Expr::Var("missing".into(), line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("body undefined-variable error");
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
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Var("missing".into(), line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("scrutinee var lookup error");
        assert_eq!(
            env.scope_depth(),
            1,
            "scrutinee `?` propagation leaked a scope"
        );

        // B. Match-mode pattern expression fails — exercises
        //    `evaluate(pattern, env)?` inside the pattern loop. We use
        //    `Expr::Add(Var("missing"), Arcana(1))` here rather than a bare
        //    `Expr::Var("missing", _)` because, after PR2's binding-pattern
        //    work, a bare identifier in match-mode pattern position is
        //    intercepted as a fresh binding and never reaches `evaluate`.
        //    Wrapping the missing identifier inside an `Add` keeps the
        //    pattern an expression so the inner `evaluate` actually runs and
        //    can raise `UndefinedVariable`.
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Arcana(1, line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Add(
                    Box::new(Expr::Var("missing".into(), line())),
                    Box::new(Expr::Arcana(1, line())),
                    line(),
                ))],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("match-mode pattern var lookup error");
        assert_eq!(
            env.scope_depth(),
            1,
            "match-mode pattern `?` propagation leaked a scope"
        );

        // C. If-else-mode pattern expression fails — exercises
        //    `evaluate(pattern_expr, env)?` inside the all-true loop.
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: false,
            conditionals: vec![],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Var("missing".into(), line()))],
                guard: None,
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("if-else-mode pattern var lookup error");
        assert_eq!(
            env.scope_depth(),
            1,
            "if-else-mode pattern `?` propagation leaked a scope"
        );

        // D. Ward expression itself fails — exercises
        //    `evaluate(guard_expr.as_ref(), env)?` (the original bug site
        //    from PR #414, now folded into the central pop_scope).
        let mut env = RuntimeEnv::new();
        let oracle = Expr::Oracle {
            is_match: true,
            conditionals: vec![ConditionalAssignment {
                variable: "v".into(),
                expression: Box::new(Expr::Arcana(1, line())),
                span: line(),
            }],
            branches: vec![OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, line()))],
                guard: Some(Expr::Var("missing".into(), line())),
                body: Stmt::Expr(Expr::Arcana(0, line()), line()),
                span: line(),
            }],
            span: line(),
        };
        evaluate_expr(&oracle, &mut env).expect_err("ward var lookup error");
        assert_eq!(env.scope_depth(), 1, "ward `?` propagation leaked a scope");
    }
}
