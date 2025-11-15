use crate::ast::LineInfo;
use crate::env::{CallArg, Environment, Value};
use crate::eval::{EvalError, EvalResult};
use std::cell::RefCell;
use std::rc::Rc;

pub fn measure(
    _env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "measure() expects exactly one argument".to_string(),
            line,
        ));
    }

    let value = result_to_value(
        args.into_iter().next().expect("measure() arg").value,
        &line,
        "measure()",
    )?;

    match value {
        Value::Scroll(items) => Ok(EvalResult::data(Value::Arcana(items.borrow().len() as i64))),
        Value::Lexicon(entries) => Ok(EvalResult::data(Value::Arcana(
            entries.borrow().len() as i64
        ))),
        _ => Err(EvalError::TypeError(
            "measure() requires a scroll or lexicon".to_string(),
            line,
        )),
    }
}

pub fn inscribe(
    env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::InvalidOperation(
            "inscribe() expects a target scroll and a value".to_string(),
            line,
        ));
    }

    let mut iter = args.into_iter();
    let target = iter.next().expect("target argument should exist");
    let value_arg = iter.next().expect("value argument should exist");

    let var_name = target.var_name.ok_or_else(|| {
        EvalError::InvalidOperation(
            "inscribe() target must be a morph scroll variable".to_string(),
            line.clone(),
        )
    })?;
    let var_info = env
        .get_var_mut(&var_name)
        .ok_or_else(|| EvalError::UndefinedVariable(var_name.clone(), line.clone()))?;
    if !var_info.is_morph {
        return Err(EvalError::InvalidOperation(
            "inscribe() target must be morph".to_string(),
            line,
        ));
    }

    match &mut var_info.value {
        Value::Scroll(items) => {
            let value = result_to_value(value_arg.value, &line, "inscribe()")?;
            items.borrow_mut().push(value);
            Ok(EvalResult::abyss())
        }
        _ => Err(EvalError::TypeError(
            "inscribe() target must be a scroll".to_string(),
            line,
        )),
    }
}

pub fn retract(
    env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "retract() expects only the target scroll".to_string(),
            line,
        ));
    }

    let target = args.into_iter().next().expect("target should exist");
    let var_name = target.var_name.ok_or_else(|| {
        EvalError::InvalidOperation(
            "retract() target must be a morph scroll variable".to_string(),
            line.clone(),
        )
    })?;
    let var_info = env
        .get_var_mut(&var_name)
        .ok_or_else(|| EvalError::UndefinedVariable(var_name.clone(), line.clone()))?;
    if !var_info.is_morph {
        return Err(EvalError::InvalidOperation(
            "retract() target must be morph".to_string(),
            line,
        ));
    }

    match &mut var_info.value {
        Value::Scroll(items) => {
            let value = items.borrow_mut().pop().ok_or_else(|| {
                EvalError::InvalidOperation(
                    "retract() cannot pop from an empty scroll".to_string(),
                    line.clone(),
                )
            })?;
            Ok(EvalResult::data(value))
        }
        _ => Err(EvalError::TypeError(
            "retract() target must be a scroll".to_string(),
            line,
        )),
    }
}

pub fn expunge(
    env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::InvalidOperation(
            "expunge() expects a lexicon target and a rune key".to_string(),
            line.clone(),
        ));
    }

    let mut iter = args.into_iter();
    let target = iter.next().expect("lexicon target should exist");
    let key_arg = iter.next().expect("key argument should exist");

    let key_value = result_to_value(key_arg.value, &line, "expunge()")?;
    let key = match key_value {
        Value::Rune(rune) => rune.as_ref().clone(),
        _ => {
            return Err(EvalError::TypeError(
                "expunge() key must be a rune".to_string(),
                line,
            ));
        }
    };

    let var_name = target.var_name.ok_or_else(|| {
        EvalError::InvalidOperation(
            "expunge() target must be a morph lexicon variable".to_string(),
            line.clone(),
        )
    })?;
    let var_info = env
        .get_var_mut(&var_name)
        .ok_or_else(|| EvalError::UndefinedVariable(var_name.clone(), line.clone()))?;
    if !var_info.is_morph {
        return Err(EvalError::InvalidOperation(
            "expunge() target must be morph".to_string(),
            line,
        ));
    }

    match &mut var_info.value {
        Value::Lexicon(entries) => {
            entries.borrow_mut().remove(&key);
            Ok(EvalResult::abyss())
        }
        _ => Err(EvalError::TypeError(
            "expunge() target must be a lexicon".to_string(),
            line,
        )),
    }
}

pub fn contents(
    _env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "contents() expects a single lexicon argument".to_string(),
            line,
        ));
    }

    let value = result_to_value(
        args.into_iter().next().expect("contents() arg").value,
        &line,
        "contents()",
    )?;

    match value {
        Value::Lexicon(entries) => {
            let keys: Vec<Value> = entries
                .borrow()
                .keys()
                .map(|key| Value::Rune(Rc::new(key.clone())))
                .collect();
            Ok(EvalResult::data(Value::Scroll(Rc::new(RefCell::new(keys)))))
        }
        _ => Err(EvalError::TypeError(
            "contents() argument must be a lexicon".to_string(),
            line,
        )),
    }
}

fn result_to_value(
    result: EvalResult,
    line: &Option<LineInfo>,
    context: &str,
) -> Result<Value, EvalError> {
    match result {
        EvalResult::Data(value) => Ok(value),
        EvalResult::Artifact(handle) => Ok(Value::Artifact(handle)),
        EvalResult::Revealed(_) | EvalResult::Resume(_) | EvalResult::Eject(_) => {
            Err(EvalError::InvalidOperation(
                format!("{} cannot accept control-flow results", context),
                line.clone(),
            ))
        }
    }
}
