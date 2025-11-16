use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{AST, LineInfo, Type};
use crate::env::{BuiltinMethodRegistry, CallArg, Environment, Value};
use crate::eval::{EvalError, EvalResult};

use super::{call_arg_to_value, ensure_mutable_receiver, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Scroll);
    table.insert("tally".to_string(), scroll_tally);
    table.insert("scribe".to_string(), scroll_scribe);
    table.insert("extract".to_string(), scroll_extract);
}

fn scroll_tally(
    _env: &mut Environment,
    _receiver_ast: &AST,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let items = expect_scroll(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "tally() does not take any arguments".to_string(),
            line_info.clone(),
        ));
    }
    Ok(EvalResult::data(Value::Arcana(items.borrow().len() as i64)))
}

fn scroll_scribe(
    env: &mut Environment,
    receiver_ast: &AST,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
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
            line_info.clone(),
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
    env: &mut Environment,
    receiver_ast: &AST,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
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
            line_info.clone(),
        ));
    }

    let value = items.borrow_mut().pop().ok_or_else(|| {
        EvalError::InvalidOperation(
            "extract() cannot pop from an empty scroll".to_string(),
            line_info.clone(),
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
