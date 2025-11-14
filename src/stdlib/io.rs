use crate::ast::LineInfo;
use crate::eval::{EvalError, EvalResult};
use std::io::{self, Write};

pub fn native_unveil(
    args: Vec<EvalResult>,
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
        .map(|result| match result {
            EvalResult::Omen(b) => Ok(if *b {
                "boon".to_string()
            } else {
                "hex".to_string()
            }),
            EvalResult::Arcana(n) => Ok(n.to_string()),
            EvalResult::Aether(n) => Ok(n.to_string()),
            EvalResult::Rune(s) => Ok(s.replace("\\n", "\n")),
            EvalResult::Abyss => Ok("".to_string()),
            _ => Err(EvalError::InvalidOperation(
                "Unsupported type passed to unveil()".to_string(),
                line.clone(),
            )),
        })
        .collect();

    let output_str = outputs?.join("");
    println!("{}", output_str);
    Ok(EvalResult::Abyss)
}

pub fn native_summon(
    args: Vec<EvalResult>,
    line: Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "summon() requires exactly 1 argument (prompt)".to_string(),
            line,
        ));
    }

    let prompt = match &args[0] {
        EvalResult::Rune(s) => s,
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

    Ok(EvalResult::Rune(input.trim().to_string()))
}
