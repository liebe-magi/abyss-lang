//! Sandbox-safe math rituals: pure functions with no I/O, so they behave
//! identically on the CLI and in the Wasm playground.

use crate::env::{CallArg, RuntimeEnv, Value};
use crate::eval::values::describe_value;
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::Span;

/// `abs(x)` — magnitude of an `arcana` or `aether`, preserving the type.
pub fn native_abs(
    _env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line_info: Option<Span>,
) -> Result<EvalResult, EvalError> {
    match take_one_numeric(args, "abs", &line_info)? {
        Value::Arcana(n) => Ok(EvalResult::data(Value::Arcana(n.abs()))),
        Value::Aether(n) => Ok(EvalResult::data(Value::Aether(n.abs()))),
        _ => unreachable!("take_one_numeric only returns numeric values"),
    }
}

/// `sqrt(x)` — square root as a `fate`: `bless {{ value }}` for a
/// non-negative `aether`, `curse {{ reason }}` for a negative one, so
/// call sites write `sqrt(x)?` (v0.7.0 fallible-API convention,
/// design: #548). A non-`aether` argument remains a hard error —
/// that's a programming mistake, not data-dependent failure.
pub fn native_sqrt(
    _env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line_info: Option<Span>,
) -> Result<EvalResult, EvalError> {
    use std::rc::Rc;

    let value = take_one_aether(args, "sqrt", &line_info)?;
    if value < 0.0 {
        return Ok(EvalResult::data(crate::stdlib::make_variant(
            "curse",
            Some((
                "reason",
                Value::Rune(Rc::new("sqrt() requires a non-negative aether".to_string())),
            )),
        )));
    }
    Ok(EvalResult::data(crate::stdlib::make_variant(
        "bless",
        Some(("value", Value::Aether(value.sqrt()))),
    )))
}

/// `floor(x)` — largest `arcana` not greater than the `aether` input.
pub fn native_floor(
    _env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line_info: Option<Span>,
) -> Result<EvalResult, EvalError> {
    let value = take_one_aether(args, "floor", &line_info)?;
    Ok(EvalResult::data(Value::Arcana(value.floor() as i64)))
}

/// `ceil(x)` — smallest `arcana` not less than the `aether` input.
pub fn native_ceil(
    _env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line_info: Option<Span>,
) -> Result<EvalResult, EvalError> {
    let value = take_one_aether(args, "ceil", &line_info)?;
    Ok(EvalResult::data(Value::Arcana(value.ceil() as i64)))
}

fn take_one_value(
    args: Vec<CallArg>,
    name: &str,
    line_info: &Option<Span>,
) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            format!("{}() expects exactly one argument", name),
            *line_info,
        ));
    }
    let arg = args.into_iter().next().expect("argument exists");
    match arg.value {
        EvalResult::Data(value) => Ok(value),
        other => Err(EvalError::InvalidOperation(
            format!("{}() received a non-value argument: {:?}", name, other),
            *line_info,
        )),
    }
}

fn take_one_numeric(
    args: Vec<CallArg>,
    name: &str,
    line_info: &Option<Span>,
) -> Result<Value, EvalError> {
    match take_one_value(args, name, line_info)? {
        value @ (Value::Arcana(_) | Value::Aether(_)) => Ok(value),
        other => Err(EvalError::InvalidOperation(
            format!(
                "{}() expects an arcana or aether value, found {}",
                name,
                describe_value(&other)
            ),
            *line_info,
        )),
    }
}

fn take_one_aether(
    args: Vec<CallArg>,
    name: &str,
    line_info: &Option<Span>,
) -> Result<f64, EvalError> {
    match take_one_value(args, name, line_info)? {
        Value::Aether(n) => Ok(n),
        other => Err(EvalError::InvalidOperation(
            format!(
                "{}() expects an aether value, found {} (transmute first)",
                name,
                describe_value(&other)
            ),
            *line_info,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(value: Value) -> Vec<CallArg> {
        vec![CallArg {
            value: EvalResult::data(value),
            var_name: None,
        }]
    }

    #[test]
    fn abs_preserves_numeric_type() {
        let mut env = RuntimeEnv::new();
        assert!(matches!(
            native_abs(&mut env, arg(Value::Arcana(-7)), None),
            Ok(EvalResult::Data(Value::Arcana(7)))
        ));
        match native_abs(&mut env, arg(Value::Aether(-2.5)), None) {
            Ok(EvalResult::Data(Value::Aether(n))) => assert_eq!(n, 2.5),
            other => panic!("expected aether, got {:?}", other),
        }
    }

    #[test]
    fn sqrt_returns_fate_variants() {
        let mut env = RuntimeEnv::new();
        match native_sqrt(&mut env, arg(Value::Aether(-1.0)), None) {
            Ok(EvalResult::Data(Value::Artifact(handle))) => {
                assert_eq!(handle.borrow().type_name, "curse");
            }
            other => panic!("expected curse, got {:?}", other),
        }
        assert!(matches!(
            native_sqrt(&mut env, arg(Value::Arcana(4)), None),
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects an aether value, found arcana")
        ));
        match native_sqrt(&mut env, arg(Value::Aether(9.0)), None) {
            Ok(EvalResult::Data(Value::Artifact(handle))) => {
                let borrowed = handle.borrow();
                assert_eq!(borrowed.type_name, "bless");
                assert!(
                    matches!(borrowed.fields.get("value"), Some(Value::Aether(n)) if *n == 3.0)
                );
            }
            other => panic!("expected bless, got {:?}", other),
        }
    }

    #[test]
    fn floor_and_ceil_return_arcana() {
        let mut env = RuntimeEnv::new();
        assert!(matches!(
            native_floor(&mut env, arg(Value::Aether(2.9)), None),
            Ok(EvalResult::Data(Value::Arcana(2)))
        ));
        assert!(matches!(
            native_ceil(&mut env, arg(Value::Aether(2.1)), None),
            Ok(EvalResult::Data(Value::Arcana(3)))
        ));
        assert!(matches!(
            native_floor(&mut env, arg(Value::Aether(-2.1)), None),
            Ok(EvalResult::Data(Value::Arcana(-3)))
        ));
    }

    #[test]
    fn arity_is_enforced() {
        let mut env = RuntimeEnv::new();
        assert!(matches!(
            native_abs(&mut env, vec![], None),
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("exactly one argument")
        ));
    }
}
