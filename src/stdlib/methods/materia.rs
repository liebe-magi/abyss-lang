use std::rc::Rc;

use crate::ast::{AST, LineInfo, Type};
use crate::env::{BuiltinMethodRegistry, CallArg, Environment, Value};
use crate::eval::{EvalError, EvalResult};

use super::{expect_glyph_argument, method_table_for};

pub(super) fn register_methods(registry: &mut BuiltinMethodRegistry) {
    let table = method_table_for(registry, Type::Materia);
    table.insert("trans".to_string(), materia_trans_method);
}

fn materia_trans_method(
    _env: &mut Environment,
    _receiver_ast: &AST,
    _receiver_var_name: Option<&str>,
    receiver_value: Value,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let target_type = expect_glyph_argument(args, "trans()", line_info)?;
    let converted = convert_value_via_trans(receiver_value, &target_type, line_info)?;
    Ok(EvalResult::data(converted))
}

fn convert_value_via_trans(
    receiver: Value,
    target_type: &Type,
    line_info: &Option<LineInfo>,
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
                        line_info.clone(),
                    )
                }),
            _ => Err(EvalError::InvalidOperation(
                "cannot convert to arcana".to_string(),
                line_info.clone(),
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
                        line_info.clone(),
                    )
                }),
            _ => Err(EvalError::InvalidOperation(
                "cannot convert to aether".to_string(),
                line_info.clone(),
            )),
        },
        Type::Rune => match receiver {
            Value::Arcana(n) => Ok(Value::Rune(Rc::new(n.to_string()))),
            Value::Aether(n) => Ok(Value::Rune(Rc::new(n.to_string()))),
            _ => Err(EvalError::InvalidOperation(
                "cannot convert to rune".to_string(),
                line_info.clone(),
            )),
        },
        Type::Omen => Err(EvalError::InvalidOperation(
            "cannot convert to omen".to_string(),
            line_info.clone(),
        )),
        Type::Glyph => Err(EvalError::InvalidOperation(
            "cannot convert to glyph".to_string(),
            line_info.clone(),
        )),
        Type::Artifact(name) => Err(EvalError::InvalidOperation(
            format!("cannot convert to artifact type {}", name),
            line_info.clone(),
        )),
        _ => Err(EvalError::InvalidOperation(
            format!("cannot convert to type {:?}", target_type),
            line_info.clone(),
        )),
    }
}
