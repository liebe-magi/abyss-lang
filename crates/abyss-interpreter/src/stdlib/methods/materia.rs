use std::rc::Rc;

use crate::env::{BuiltinMethodRegistry, CallArg, RuntimeEnv, Value};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{Expr, Span, Type};

use super::{expect_glyph_argument, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Materia);
    table.insert("transmute".to_string(), materia_transmute_method);
}

fn materia_transmute_method(
    _env: &mut RuntimeEnv,
    _receiver_ast: &Expr,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<Span>,
) -> Result<EvalResult, EvalError> {
    let target_type = expect_glyph_argument(args, "transmute()", line_info)?;
    let converted = convert_value_via_transmute(receiver_value, &target_type, line_info)?;
    Ok(EvalResult::data(converted))
}

fn convert_value_via_transmute(
    receiver: Value,
    target_type: &Type,
    line_info: &Option<Span>,
) -> Result<Value, EvalError> {
    match target_type {
        Type::Arcana => match receiver {
            Value::Aether(n) => Ok(Value::Arcana(n as i64)),
            Value::Rune(text) => text
                .as_ref()
                .parse::<i64>()
                .map(Value::Arcana)
                .map_err(|_| {
                    EvalError::InvalidOperation(
                        "failed to convert rune to arcana".to_string(),
                        *line_info,
                    )
                }),
            _ => Err(EvalError::InvalidOperation(
                "cannot convert to arcana".to_string(),
                *line_info,
            )),
        },
        Type::Aether => match receiver {
            Value::Arcana(n) => Ok(Value::Aether(n as f64)),
            Value::Rune(text) => text
                .as_ref()
                .parse::<f64>()
                .map(Value::Aether)
                .map_err(|_| {
                    EvalError::InvalidOperation(
                        "failed to convert rune to aether".to_string(),
                        *line_info,
                    )
                }),
            _ => Err(EvalError::InvalidOperation(
                "cannot convert to aether".to_string(),
                *line_info,
            )),
        },
        Type::Rune => match receiver {
            Value::Arcana(n) => Ok(Value::Rune(Rc::new(n.to_string()))),
            Value::Aether(n) => Ok(Value::Rune(Rc::new(n.to_string()))),
            _ => Err(EvalError::InvalidOperation(
                "cannot convert to rune".to_string(),
                *line_info,
            )),
        },
        Type::Omen => Err(EvalError::InvalidOperation(
            "cannot convert to omen".to_string(),
            *line_info,
        )),
        Type::Glyph => Err(EvalError::InvalidOperation(
            "cannot convert to glyph".to_string(),
            *line_info,
        )),
        Type::Artifact(name) => Err(EvalError::InvalidOperation(
            format!("cannot convert to artifact type {}", name),
            *line_info,
        )),
        _ => Err(EvalError::InvalidOperation(
            format!("cannot convert to type {:?}", target_type),
            *line_info,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Value;

    fn dummy_args(val: Value) -> Vec<CallArg> {
        vec![CallArg {
            value: EvalResult::data(val),
            var_name: None,
        }]
    }

    #[test]
    fn test_transmute_arguments() {
        let mut env = RuntimeEnv::new();
        let result = materia_transmute_method(
            &mut env,
            &Expr::Abyss(None),
            None,
            Value::Abyss,
            vec![], // Needs 1
            &None,
        );
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("expects exactly one glyph argument")
        ));
    }

    #[test]
    fn test_trans_invalid_target_type() {
        let mut env = RuntimeEnv::new();
        // Pass a non-glyph (e.g. integer) as target type
        let args = dummy_args(Value::Arcana(1));

        let result = materia_transmute_method(
            &mut env,
            &Expr::Abyss(None),
            None,
            Value::Abyss,
            args,
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("argument must be a glyph value")
        ));
    }

    #[test]
    fn test_trans_unknown_type() {
        let mut env = RuntimeEnv::new();
        let args = dummy_args(Value::Rune(Rc::new("unknown".to_string())));

        let result = materia_transmute_method(
            &mut env,
            &Expr::Abyss(None),
            None,
            Value::Abyss,
            args,
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("argument must be a glyph value")
        ));
    }
}
