use std::cell::RefCell;
use std::rc::Rc;

use crate::env::{BuiltinMethodRegistry, CallArg, RuntimeEnv, Value};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{Expr, Span, Type};

use super::{call_arg_to_value, ensure_mutable_receiver, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Scroll);
    table.insert("tally".to_string(), scroll_tally);
    table.insert("scribe".to_string(), scroll_scribe);
    table.insert("extract".to_string(), scroll_extract);
    table.insert("sort".to_string(), scroll_sort);
    table.insert("unique".to_string(), scroll_unique);
    table.insert("sum".to_string(), scroll_sum);
    table.insert("min".to_string(), scroll_min);
    table.insert("max".to_string(), scroll_max);
}

/// The element kinds the ordering/aggregation methods understand. A scroll
/// must be homogeneous in one of these for `sort` / `sum` / `min` / `max`.
#[derive(Clone, Copy, PartialEq)]
enum ScalarKind {
    Arcana,
    Aether,
    Rune,
}

fn scalar_kind(value: &Value) -> Option<ScalarKind> {
    match value {
        Value::Arcana(_) => Some(ScalarKind::Arcana),
        Value::Aether(_) => Some(ScalarKind::Aether),
        Value::Rune(_) => Some(ScalarKind::Rune),
        _ => None,
    }
}

/// Validate that every element shares one orderable kind; error otherwise.
fn homogeneous_kind(
    items: &[Value],
    method: &str,
    line_info: &Option<Span>,
) -> Result<ScalarKind, EvalError> {
    let mut kind: Option<ScalarKind> = None;
    for item in items {
        let item_kind = scalar_kind(item).ok_or_else(|| {
            EvalError::InvalidOperation(
                format!(
                    "{}() requires a scroll of arcana, aether, or rune values",
                    method
                ),
                *line_info,
            )
        })?;
        match kind {
            None => kind = Some(item_kind),
            Some(existing) if existing == item_kind => {}
            Some(_) => {
                return Err(EvalError::InvalidOperation(
                    format!(
                        "{}() requires all scroll elements to share one type",
                        method
                    ),
                    *line_info,
                ));
            }
        }
    }
    kind.ok_or_else(|| {
        EvalError::InvalidOperation(
            format!("{}() cannot operate on an empty scroll", method),
            *line_info,
        )
    })
}

fn compare_scalars(kind: ScalarKind, a: &Value, b: &Value) -> std::cmp::Ordering {
    match (kind, a, b) {
        (ScalarKind::Arcana, Value::Arcana(x), Value::Arcana(y)) => x.cmp(y),
        (ScalarKind::Aether, Value::Aether(x), Value::Aether(y)) => x.total_cmp(y),
        (ScalarKind::Rune, Value::Rune(x), Value::Rune(y)) => x.as_ref().cmp(y.as_ref()),
        _ => unreachable!("homogeneity validated before comparing"),
    }
}

/// Returns a new sorted scroll (ascending); the receiver is untouched, so
/// no `morph` receiver is required.
fn scroll_sort(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "sort() does not take any arguments".to_string(),
            *line_info,
        ));
    }
    let mut sorted: Vec<Value> = items.borrow().clone();
    if sorted.is_empty() {
        return Ok(EvalResult::data(Value::Scroll(Rc::new(RefCell::new(
            sorted,
        )))));
    }
    let kind = homogeneous_kind(&sorted, "sort", line_info)?;
    sorted.sort_by(|a, b| compare_scalars(kind, a, b));
    Ok(EvalResult::data(Value::Scroll(Rc::new(RefCell::new(
        sorted,
    )))))
}

/// Returns a new scroll keeping the first occurrence of each scalar value
/// (arcana / aether / rune / omen); the receiver is untouched.
fn scroll_unique(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "unique() does not take any arguments".to_string(),
            *line_info,
        ));
    }
    let source: Vec<Value> = items.borrow().clone();
    let mut seen: Vec<Value> = Vec::new();
    for item in source {
        let is_scalar = matches!(
            item,
            Value::Arcana(_) | Value::Aether(_) | Value::Rune(_) | Value::Omen(_)
        );
        if !is_scalar {
            return Err(EvalError::InvalidOperation(
                "unique() requires a scroll of scalar values (arcana, aether, rune, omen)"
                    .to_string(),
                *line_info,
            ));
        }
        let duplicate = seen.iter().any(|existing| match (existing, &item) {
            (Value::Arcana(x), Value::Arcana(y)) => x == y,
            (Value::Aether(x), Value::Aether(y)) => x.total_cmp(y).is_eq(),
            (Value::Rune(x), Value::Rune(y)) => x.as_ref() == y.as_ref(),
            (Value::Omen(x), Value::Omen(y)) => x == y,
            _ => false,
        });
        if !duplicate {
            seen.push(item);
        }
    }
    Ok(EvalResult::data(Value::Scroll(Rc::new(RefCell::new(seen)))))
}

/// Sums a homogeneous numeric scroll. Empty scrolls error for now; the
/// v0.7.x fallible-API revision moves the aggregation methods to `augury`
/// returns (see the roadmap).
fn scroll_sum(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "sum() does not take any arguments".to_string(),
            *line_info,
        ));
    }
    let values: Vec<Value> = items.borrow().clone();
    let kind = homogeneous_kind(&values, "sum", line_info)?;
    match kind {
        ScalarKind::Arcana => {
            let mut total: i64 = 0;
            for value in &values {
                if let Value::Arcana(n) = value {
                    total += n;
                }
            }
            Ok(EvalResult::data(Value::Arcana(total)))
        }
        ScalarKind::Aether => {
            let mut total: f64 = 0.0;
            for value in &values {
                if let Value::Aether(n) = value {
                    total += n;
                }
            }
            Ok(EvalResult::data(Value::Aether(total)))
        }
        ScalarKind::Rune => Err(EvalError::InvalidOperation(
            "sum() requires a scroll of arcana or aether values".to_string(),
            *line_info,
        )),
    }
}

fn scroll_extremum(
    receiver_value: Value,
    args: Vec<CallArg>,
    method: &str,
    want_max: bool,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            format!("{}() does not take any arguments", method),
            *line_info,
        ));
    }
    let values: Vec<Value> = items.borrow().clone();
    let kind = homogeneous_kind(&values, method, line_info)?;
    let mut best = values[0].clone();
    for value in &values[1..] {
        let ordering = compare_scalars(kind, value, &best);
        let better = if want_max {
            ordering.is_gt()
        } else {
            ordering.is_lt()
        };
        if better {
            best = value.clone();
        }
    }
    Ok(EvalResult::data(best))
}

fn scroll_min(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    scroll_extremum(receiver_value, args, "min", false, line_info)
}

fn scroll_max(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    scroll_extremum(receiver_value, args, "max", true, line_info)
}

fn scroll_tally(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "tally() does not take any arguments".to_string(),
            *line_info,
        ));
    }
    Ok(EvalResult::data(Value::Arcana(items.borrow().len() as i64)))
}

fn scroll_scribe(
    env: &mut RuntimeEnv,
    receiver_ast: &Expr,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    ensure_mutable_receiver(
        env,
        receiver_ast,
        receiver_var_name,
        "scroll",
        "scribe",
        line_info,
    )?;

    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "scribe() expects exactly one argument".to_string(),
            *line_info,
        ));
    }

    let value = call_arg_to_value(
        args.into_iter().next().expect("argument exists"),
        "scribe()",
        line_info,
    )?;
    items.borrow_mut().push(value);
    Ok(EvalResult::abyss())
}

fn scroll_extract(
    env: &mut RuntimeEnv,
    receiver_ast: &Expr,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    ensure_mutable_receiver(
        env,
        receiver_ast,
        receiver_var_name,
        "scroll",
        "extract",
        line_info,
    )?;

    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "extract() does not take any arguments".to_string(),
            *line_info,
        ));
    }

    let value = items.borrow_mut().pop().ok_or_else(|| {
        EvalError::InvalidOperation(
            "extract() cannot pop from an empty scroll".to_string(),
            *line_info,
        )
    })?;

    Ok(EvalResult::data(value))
}

fn expect_scroll(value: Value) -> Rc<RefCell<Vec<Value>>> {
    if let Value::Scroll(items) = value {
        items
    } else {
        panic!("scroll builtin dispatched with non-scroll value");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Value;

    fn dummy_scroll() -> Value {
        Value::Scroll(Rc::new(RefCell::new(Vec::new())))
    }

    fn dummy_args(count: usize) -> Vec<CallArg> {
        (0..count)
            .map(|_| CallArg {
                value: EvalResult::data(Value::Abyss),
                var_name: None,
            })
            .collect()
    }

    #[test]
    fn test_tally_arguments() {
        let mut env = RuntimeEnv::new();
        let result = scroll_tally(
            &mut env,
            &Expr::Abyss(None),
            None,
            dummy_scroll(),
            dummy_args(1),
            &None,
        );
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("does not take any arguments")
        ));
    }

    #[test]
    fn test_scribe_arguments() {
        let mut env = RuntimeEnv::new();
        env.set_var("list".to_string(), dummy_scroll(), Type::Scroll, true, None);

        let result = scroll_scribe(
            &mut env,
            &Expr::Var("list".to_string(), None),
            Some("list"),
            dummy_scroll(),
            dummy_args(0), // Needs 1
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects exactly one argument")
        ));
    }

    #[test]
    fn test_extract_arguments() {
        let mut env = RuntimeEnv::new();
        env.set_var("list".to_string(), dummy_scroll(), Type::Scroll, true, None);

        let result = scroll_extract(
            &mut env,
            &Expr::Var("list".to_string(), None),
            Some("list"),
            dummy_scroll(),
            dummy_args(1), // Needs 0
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("does not take any arguments")
        ));
    }

    #[test]
    fn test_extract_empty() {
        let mut env = RuntimeEnv::new();
        env.set_var("list".to_string(), dummy_scroll(), Type::Scroll, true, None);

        let result = scroll_extract(
            &mut env,
            &Expr::Var("list".to_string(), None),
            Some("list"),
            dummy_scroll(),
            dummy_args(0),
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("cannot pop from an empty scroll")
        ));
    }
}
