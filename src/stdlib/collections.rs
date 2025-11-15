use crate::ast::LineInfo;
use crate::env::{CallArg, Environment, Value};
use crate::eval::{EvalError, EvalResult};

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

    match &args[0].value {
        EvalResult::Scroll(items) => Ok(EvalResult::Arcana(items.len() as i64)),
        EvalResult::Lexicon(entries) => Ok(EvalResult::Arcana(entries.len() as i64)),
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
            items.push(result_to_value(value_arg.value, &line, "inscribe()")?);
            Ok(EvalResult::Abyss)
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
            let value = items.pop().ok_or_else(|| {
                EvalError::InvalidOperation(
                    "retract() cannot pop from an empty scroll".to_string(),
                    line.clone(),
                )
            })?;
            Ok(value_to_result(&value))
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

    let key = match key_arg.value {
        EvalResult::Rune(s) => s,
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
            entries.remove(&key);
            Ok(EvalResult::Abyss)
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

    match &args[0].value {
        EvalResult::Lexicon(entries) => {
            let keys = entries
                .keys()
                .map(|key| EvalResult::Rune(key.clone()))
                .collect();
            Ok(EvalResult::Scroll(keys))
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
        EvalResult::Omen(b) => Ok(Value::Omen(b)),
        EvalResult::Arcana(n) => Ok(Value::Arcana(n)),
        EvalResult::Aether(n) => Ok(Value::Aether(n)),
        EvalResult::Rune(s) => Ok(Value::Rune(s)),
        EvalResult::Abyss => Ok(Value::Abyss),
        EvalResult::Scroll(items) => Ok(Value::Scroll(
            items
                .into_iter()
                .map(|item| result_to_value(item, line, context))
                .collect::<Result<_, _>>()?,
        )),
        EvalResult::Lexicon(entries) => Ok(Value::Lexicon(
            entries
                .into_iter()
                .map(|(k, v)| Ok((k, result_to_value(v, line, context)?)))
                .collect::<Result<_, EvalError>>()?,
        )),
        EvalResult::Revealed(_) | EvalResult::Resume(_) | EvalResult::Eject(_) => {
            Err(EvalError::InvalidOperation(
                format!("{} cannot accept control-flow results", context),
                line.clone(),
            ))
        }
    }
}

fn value_to_result(value: &Value) -> EvalResult {
    match value {
        Value::Omen(b) => EvalResult::Omen(*b),
        Value::Arcana(n) => EvalResult::Arcana(*n),
        Value::Aether(n) => EvalResult::Aether(*n),
        Value::Rune(s) => EvalResult::Rune(s.clone()),
        Value::Abyss => EvalResult::Abyss,
        Value::Scroll(items) => EvalResult::Scroll(items.iter().map(value_to_result).collect()),
        Value::Lexicon(entries) => EvalResult::Lexicon(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), value_to_result(v)))
                .collect(),
        ),
    }
}
