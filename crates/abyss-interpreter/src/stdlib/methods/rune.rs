use std::cell::RefCell;
use std::rc::Rc;

use crate::env::{BuiltinMethodRegistry, CallArg, RuntimeEnv, Value};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{Expr, Span, Type};

use super::{call_arg_to_value, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Rune);
    table.insert("upper".to_string(), rune_upper);
    table.insert("lower".to_string(), rune_lower);
    table.insert("trim".to_string(), rune_trim);
    table.insert("tally".to_string(), rune_tally);
    table.insert("contains".to_string(), rune_contains);
    table.insert("replace".to_string(), rune_replace);
    table.insert("split".to_string(), rune_split);
}

fn rune_upper(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    ensure_no_args(&args, "upper", line_info)?;
    Ok(EvalResult::data(Value::Rune(Rc::new(text.to_uppercase()))))
}

fn rune_lower(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    ensure_no_args(&args, "lower", line_info)?;
    Ok(EvalResult::data(Value::Rune(Rc::new(text.to_lowercase()))))
}

fn rune_trim(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    ensure_no_args(&args, "trim", line_info)?;
    Ok(EvalResult::data(Value::Rune(Rc::new(
        text.trim().to_string(),
    ))))
}

/// Character count (Unicode scalar values), mirroring `scroll.tally()`'s
/// element count rather than a byte length.
fn rune_tally(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    ensure_no_args(&args, "tally", line_info)?;
    Ok(EvalResult::data(Value::Arcana(text.chars().count() as i64)))
}

fn rune_contains(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    let [needle] = take_rune_args::<1>(args, "contains", line_info)?;
    Ok(EvalResult::data(Value::Omen(
        text.contains(needle.as_str()),
    )))
}

fn rune_replace(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    let [from, to] = take_rune_args::<2>(args, "replace", line_info)?;
    if from.is_empty() {
        return Err(EvalError::InvalidOperation(
            "replace() requires a non-empty search rune".to_string(),
            *line_info,
        ));
    }
    Ok(EvalResult::data(Value::Rune(Rc::new(
        text.replace(from.as_str(), to.as_str()),
    ))))
}

fn rune_split(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let text = expect_rune(receiver_value);
    let [separator] = take_rune_args::<1>(args, "split", line_info)?;
    if separator.is_empty() {
        return Err(EvalError::InvalidOperation(
            "split() requires a non-empty separator rune".to_string(),
            *line_info,
        ));
    }
    let pieces: Vec<Value> = text
        .split(separator.as_str())
        .map(|piece| Value::Rune(Rc::new(piece.to_string())))
        .collect();
    Ok(EvalResult::data(Value::Scroll(Rc::new(RefCell::new(
        pieces,
    )))))
}

fn expect_rune(value: Value) -> Rc<String> {
    if let Value::Rune(text) = value {
        text
    } else {
        panic!("rune builtin dispatched with non-rune value");
    }
}

fn ensure_no_args(
    args: &[CallArg],
    method: &str,
    line_info: &Option<Span>,
) -> Result<(), EvalError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(EvalError::InvalidOperation(
            format!("{}() does not take any arguments", method),
            *line_info,
        ))
    }
}

/// Extract exactly `N` rune arguments, with the shared wording for arity
/// and type errors.
fn take_rune_args<const N: usize>(
    args: Vec<CallArg>,
    method: &str,
    line_info: &Option<Span>,
) -> Result<[Rc<String>; N], EvalError> {
    if args.len() != N {
        let plural = if N == 1 { "argument" } else { "arguments" };
        return Err(EvalError::InvalidOperation(
            format!("{}() expects exactly {} rune {}", method, N, plural),
            *line_info,
        ));
    }
    let mut runes = Vec::with_capacity(N);
    for arg in args {
        let label = format!("{}()", method);
        match call_arg_to_value(arg, &label, line_info)? {
            Value::Rune(text) => runes.push(text),
            other => {
                return Err(EvalError::InvalidOperation(
                    format!(
                        "{}() expects rune arguments, found {}",
                        method,
                        crate::eval::values::describe_value(&other)
                    ),
                    *line_info,
                ));
            }
        }
    }
    Ok(runes
        .try_into()
        .unwrap_or_else(|_| unreachable!("length checked above")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rune_value(text: &str) -> Value {
        Value::Rune(Rc::new(text.to_string()))
    }

    fn rune_arg(text: &str) -> CallArg {
        CallArg {
            value: EvalResult::data(rune_value(text)),
            var_name: None,
        }
    }

    fn unwrap_rune(result: Result<EvalResult, EvalError>) -> String {
        match result.expect("method should succeed") {
            EvalResult::Data(Value::Rune(text)) => text.as_ref().clone(),
            other => panic!("expected rune result, got {:?}", other),
        }
    }

    #[test]
    fn upper_lower_trim_transform_the_receiver() {
        let mut env = RuntimeEnv::new();
        let recv = &Expr::Abyss(None);
        assert_eq!(
            unwrap_rune(rune_upper(
                &mut env,
                recv,
                None,
                rune_value("abyss"),
                vec![],
                &None
            )),
            "ABYSS"
        );
        assert_eq!(
            unwrap_rune(rune_lower(
                &mut env,
                recv,
                None,
                rune_value("ABYSS"),
                vec![],
                &None
            )),
            "abyss"
        );
        assert_eq!(
            unwrap_rune(rune_trim(
                &mut env,
                recv,
                None,
                rune_value("  rune  "),
                vec![],
                &None
            )),
            "rune"
        );
    }

    #[test]
    fn tally_counts_characters_not_bytes() {
        let mut env = RuntimeEnv::new();
        let result = rune_tally(
            &mut env,
            &Expr::Abyss(None),
            None,
            rune_value("呪文"),
            vec![],
            &None,
        );
        assert!(matches!(result, Ok(EvalResult::Data(Value::Arcana(2)))));
    }

    #[test]
    fn contains_returns_omen() {
        let mut env = RuntimeEnv::new();
        let result = rune_contains(
            &mut env,
            &Expr::Abyss(None),
            None,
            rune_value("dark incantation"),
            vec![rune_arg("cant")],
            &None,
        );
        assert!(matches!(result, Ok(EvalResult::Data(Value::Omen(true)))));
    }

    #[test]
    fn replace_rejects_empty_search() {
        let mut env = RuntimeEnv::new();
        let result = rune_replace(
            &mut env,
            &Expr::Abyss(None),
            None,
            rune_value("aaa"),
            vec![rune_arg(""), rune_arg("b")],
            &None,
        );
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("non-empty search")
        ));
    }

    #[test]
    fn split_produces_scroll_of_runes() {
        let mut env = RuntimeEnv::new();
        let result = rune_split(
            &mut env,
            &Expr::Abyss(None),
            None,
            rune_value("a,b,c"),
            vec![rune_arg(",")],
            &None,
        )
        .expect("split should succeed");
        match result {
            EvalResult::Data(Value::Scroll(items)) => {
                let items = items.borrow();
                assert_eq!(items.len(), 3);
                assert!(matches!(&items[1], Value::Rune(t) if t.as_ref() == "b"));
            }
            other => panic!("expected scroll, got {:?}", other),
        }
    }

    #[test]
    fn arity_and_type_errors_share_wording() {
        let mut env = RuntimeEnv::new();
        let recv = &Expr::Abyss(None);
        let result = rune_contains(&mut env, recv, None, rune_value("x"), vec![], &None);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects exactly 1 rune argument")
        ));

        let bad_type = CallArg {
            value: EvalResult::data(Value::Arcana(1)),
            var_name: None,
        };
        let result = rune_contains(&mut env, recv, None, rune_value("x"), vec![bad_type], &None);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects rune arguments, found arcana")
        ));
    }
}
