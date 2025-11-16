use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{AST, LineInfo, Type};
use crate::env::{BuiltinMethodRegistry, CallArg, Environment, Value};
use crate::eval::{EvalError, EvalResult};

use super::{call_arg_to_value, ensure_mutable_receiver, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Lexicon);
    table.insert("tally".to_string(), lexicon_tally);
    table.insert("define".to_string(), lexicon_define);
    table.insert("expunge".to_string(), lexicon_expunge);
    table.insert("glossary".to_string(), lexicon_glossary);
}

fn lexicon_tally(
    _env: &mut Environment,
    _receiver_ast: &AST,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let entries = expect_lexicon(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "tally() does not take any arguments".to_string(),
            line_info.clone(),
        ));
    }
    Ok(EvalResult::data(Value::Arcana(
        entries.borrow().len() as i64
    )))
}

fn lexicon_define(
    env: &mut Environment,
    receiver_ast: &AST,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let entries = expect_lexicon(receiver_value);
    ensure_mutable_receiver(
        env,
        receiver_ast,
        receiver_var_name,
        "lexicon",
        "define",
        line_info,
    )?;
    if args.len() != 2 {
        return Err(EvalError::InvalidOperation(
            "define() expects a rune key and a value".to_string(),
            line_info.clone(),
        ));
    }

    let mut iter = args.into_iter();
    let key_value = call_arg_to_value(iter.next().expect("key"), "define() key", line_info)?;
    let key = match key_value {
        Value::Rune(text) => text.as_ref().clone(),
        _ => {
            return Err(EvalError::TypeError(
                "define() key must be a rune".to_string(),
                line_info.clone(),
            ));
        }
    };
    let value = call_arg_to_value(iter.next().expect("value"), "define() value", line_info)?;
    entries.borrow_mut().insert(key, value);
    Ok(EvalResult::abyss())
}

fn lexicon_expunge(
    env: &mut Environment,
    receiver_ast: &AST,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let entries = expect_lexicon(receiver_value);
    ensure_mutable_receiver(
        env,
        receiver_ast,
        receiver_var_name,
        "lexicon",
        "expunge",
        line_info,
    )?;
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "expunge() expects exactly one rune key".to_string(),
            line_info.clone(),
        ));
    }

    let key_value = call_arg_to_value(
        args.into_iter().next().expect("key"),
        "expunge() key",
        line_info,
    )?;
    let key = match key_value {
        Value::Rune(text) => text.as_ref().clone(),
        _ => {
            return Err(EvalError::TypeError(
                "expunge() key must be a rune".to_string(),
                line_info.clone(),
            ));
        }
    };
    entries.borrow_mut().remove(&key);
    Ok(EvalResult::abyss())
}

fn lexicon_glossary(
    _env: &mut Environment,
    _receiver_ast: &AST,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let entries = expect_lexicon(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "glossary() does not take any arguments".to_string(),
            line_info.clone(),
        ));
    }

    let keys: Vec<Value> = entries
        .borrow()
        .keys()
        .map(|key| Value::Rune(Rc::new(key.clone())))
        .collect();
    Ok(EvalResult::data(Value::Scroll(Rc::new(RefCell::new(keys)))))
}

fn expect_lexicon(value: Value) -> Rc<RefCell<HashMap<String, Value>>> {
    if let Value::Lexicon(entries) = value {
        entries
    } else {
        panic!("lexicon builtin dispatched with non-lexicon value");
    }
}
