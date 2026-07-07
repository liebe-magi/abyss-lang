//! Script-runtime rituals: deliberate termination.

use crate::env::{CallArg, RuntimeEnv, Value};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::Span;

/// `perish(code)` — terminate the script with an exit code; `perish()`
/// means code 0. Rides the error channel so every enclosing scope
/// unwinds; the host decides what termination means (process exit for
/// `invoke`, a printed notice for the REPL, an error report for the
/// Playground).
pub fn native_perish(
    _env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line_info: Option<Span>,
) -> Result<EvalResult, EvalError> {
    let code = match args.len() {
        0 => 0,
        1 => match args.into_iter().next().expect("argument exists").value {
            EvalResult::Data(Value::Arcana(code)) => code,
            other => {
                return Err(EvalError::InvalidOperation(
                    format!("perish() expects an arcana exit code, got {:?}", other),
                    line_info,
                ));
            }
        },
        _ => {
            return Err(EvalError::InvalidOperation(
                "perish() expects at most one arcana argument".to_string(),
                line_info,
            ));
        }
    };
    Err(EvalError::Perished(code, line_info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perish_defaults_to_zero_and_accepts_code() {
        let mut env = RuntimeEnv::new();
        assert!(matches!(
            native_perish(&mut env, vec![], None),
            Err(EvalError::Perished(0, _))
        ));
        let arg = CallArg {
            value: EvalResult::data(Value::Arcana(3)),
            var_name: None,
        };
        assert!(matches!(
            native_perish(&mut env, vec![arg], None),
            Err(EvalError::Perished(3, _))
        ));
    }
}
