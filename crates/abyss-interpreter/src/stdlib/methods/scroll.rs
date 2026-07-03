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
