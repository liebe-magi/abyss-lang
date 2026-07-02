use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::env::{BuiltinMethodRegistry, CallArg, RuntimeEnv, Value};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{Expr, Span, Type};

use super::{call_arg_to_value, ensure_mutable_receiver, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Lexicon);
    table.insert("tally".to_string(), lexicon_tally);
    table.insert("define".to_string(), lexicon_define);
    table.insert("expunge".to_string(), lexicon_expunge);
    table.insert("glossary".to_string(), lexicon_glossary);
}

fn lexicon_tally(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let entries = expect_lexicon(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "tally() does not take any arguments".to_string(),
            *line_info,
        ));
    }
    Ok(EvalResult::data(Value::Arcana(
        entries.borrow().len() as i64
    )))
}

fn lexicon_define(
    env: &mut RuntimeEnv,
    receiver_ast: &Expr,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
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
            *line_info,
        ));
    }

    let mut iter = args.into_iter();
    let key_value = call_arg_to_value(iter.next().expect("key"), "define() key", line_info)?;
    let key = match key_value {
        Value::Rune(text) => text.as_ref().clone(),
        _ => {
            return Err(EvalError::TypeError(
                "define() key must be a rune".to_string(),
                *line_info,
            ));
        }
    };
    let value = call_arg_to_value(iter.next().expect("value"), "define() value", line_info)?;
    entries.borrow_mut().insert(key, value);
    Ok(EvalResult::abyss())
}

fn lexicon_expunge(
    env: &mut RuntimeEnv,
    receiver_ast: &Expr,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
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
            *line_info,
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
                *line_info,
            ));
        }
    };
    entries.borrow_mut().remove(&key);
    Ok(EvalResult::abyss())
}

fn lexicon_glossary(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let entries = expect_lexicon(receiver_value);
    if !args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "glossary() does not take any arguments".to_string(),
            *line_info,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Value;

    fn dummy_lexicon() -> Value {
        Value::Lexicon(Rc::new(RefCell::new(HashMap::new())))
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
        let result = lexicon_tally(
            &mut env,
            &Expr::Abyss(None),
            None,
            dummy_lexicon(),
            dummy_args(1),
            &None,
        );
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("does not take any arguments")
        ));
    }

    #[test]
    fn test_define_arguments() {
        let mut env = RuntimeEnv::new();
        let _result = lexicon_define(
            &mut env,
            &Expr::Abyss(None),
            Some("lex"), // Mutable receiver needs a name
            dummy_lexicon(),
            dummy_args(1), // Needs 2
            &None,
        );
        // Note: ensure_mutable_receiver might fail first if we don't set up env correctly,
        // but here we are testing argument count which comes after mutable check?
        // Actually, mutable check is first.
        // So we need to mock the env to have the variable.
        // But lexicon_define takes env.
        // Let's just check the argument count logic if we can bypass mutable check?
        // No, mutable check is hardcoded.
        // We need to set up a mutable variable in env.

        env.set_var(
            "lex".to_string(),
            dummy_lexicon(),
            Type::Lexicon,
            true,
            None,
        );

        // Now call with wrong args
        let result = lexicon_define(
            &mut env,
            &Expr::Var("lex".to_string(), None),
            Some("lex"),
            dummy_lexicon(),
            dummy_args(1),
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects a rune key and a value")
        ));
    }

    #[test]
    fn test_expunge_arguments() {
        let mut env = RuntimeEnv::new();
        env.set_var(
            "lex".to_string(),
            dummy_lexicon(),
            Type::Lexicon,
            true,
            None,
        );

        let result = lexicon_expunge(
            &mut env,
            &Expr::Var("lex".to_string(), None),
            Some("lex"),
            dummy_lexicon(),
            dummy_args(0), // Needs 1
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects exactly one rune key")
        ));
    }

    #[test]
    fn test_glossary_arguments() {
        let mut env = RuntimeEnv::new();
        let result = lexicon_glossary(
            &mut env,
            &Expr::Abyss(None),
            None,
            dummy_lexicon(),
            dummy_args(1),
            &None,
        );
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("does not take any arguments")
        ));
    }
}
