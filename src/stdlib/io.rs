use crate::ast::LineInfo;
use crate::env::{CallArg, Environment, Value};
use crate::eval::{EvalError, EvalResult};
use std::io::{self, Write};
use std::rc::Rc;

pub fn native_unveil(
    _env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "unveil() requires at least 1 argument".to_string(),
            line,
        ));
    }

    let outputs: Result<Vec<String>, EvalError> = args
        .iter()
        .map(|arg| format_eval_result(&arg.value, &line))
        .collect();

    let output_str = outputs?.join("");
    println!("{}", output_str);
    Ok(EvalResult::abyss())
}

pub fn native_summon(
    _env: &mut Environment,
    args: Vec<CallArg>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "summon() requires exactly 1 argument (prompt)".to_string(),
            line,
        ));
    }

    let prompt = match &args[0].value {
        EvalResult::Data(Value::Rune(r)) => r.as_ref(),
        _ => {
            return Err(EvalError::TypeError(
                "summon() argument must be a Rune (prompt)".to_string(),
                line,
            ));
        }
    };

    print!("{}", prompt);
    io::stdout().flush().map_err(|_| {
        EvalError::InvalidOperation("Failed to flush stdout".to_string(), line.clone())
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| {
        EvalError::InvalidOperation("Failed to read input".to_string(), line.clone())
    })?;

    Ok(EvalResult::data(Value::Rune(Rc::new(
        input.trim().to_string(),
    ))))
}

fn format_eval_result(value: &EvalResult, line: &Option<LineInfo>) -> Result<String, EvalError> {
    match value {
        EvalResult::Data(inner) => format_value(inner, line),
        EvalResult::Revealed(_) => Err(EvalError::InvalidOperation(
            "Cannot unveil a Revealed value (control flow construct)".to_string(),
            line.clone(),
        )),
        EvalResult::Resume(_) => Err(EvalError::InvalidOperation(
            "Cannot unveil a Resume value (control flow construct)".to_string(),
            line.clone(),
        )),
        EvalResult::Eject(_) => Err(EvalError::InvalidOperation(
            "Cannot unveil an Eject value (control flow construct)".to_string(),
            line.clone(),
        )),
    }
}

fn format_value(value: &Value, _line: &Option<LineInfo>) -> Result<String, EvalError> {
    match value {
        Value::Omen(b) => Ok(if *b { "boon" } else { "hex" }.to_string()),
        Value::Arcana(n) => Ok(n.to_string()),
        Value::Aether(n) => Ok(n.to_string()),
        Value::Rune(s) => Ok(s.replace("\\n", "\n")),
        Value::Abyss => Ok(String::new()),
        Value::Scroll(items) => {
            let parts: Result<Vec<String>, EvalError> = items
                .borrow()
                .iter()
                .map(|item| format_value(item, _line))
                .collect();
            Ok(format!("[{}]", parts?.join(", ")))
        }
        Value::Lexicon(entries) => {
            let mut pieces = Vec::new();
            for (key, val) in entries.borrow().iter() {
                let formatted_value = format_value(val, _line)?;
                pieces.push(format!("\"{}\": {}", key, formatted_value));
            }
            Ok(format!("{{{}}}", pieces.join(", ")))
        }
    }
}
