mod lexicon;
mod materia;
mod scroll;

use crate::diagnostics::did_you_mean_hint;
use crate::env::{BuiltinMethodHandler, BuiltinMethodRegistry, CallArg, RuntimeEnv, Value};
use crate::eval::artifacts::collect_field_chain;
use crate::eval::values::describe_value;
use crate::eval::values::eval_result_to_value_checked;
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{AST, LineInfo, Type};
use std::collections::HashMap;

pub fn get_all_builtin_methods() -> BuiltinMethodRegistry {
    let mut registry = BuiltinMethodRegistry::new();
    materia::register_methods(&mut registry);
    scroll::register_methods(&mut registry);
    lexicon::register_methods(&mut registry);
    registry
}

pub fn dispatch_builtin_method(
    env: &mut RuntimeEnv,
    receiver_ast: &AST,
    receiver_var_name: Option<&str>,
    receiver_value: Value,
    method_name: &str,
    args: Vec<CallArg>,
    line_info: &Option<LineInfo>,
) -> Result<EvalResult, EvalError> {
    let receiver_type = receiver_type(&receiver_value);
    let fallback_type = Type::Materia;
    let registry = env.builtin_methods();

    let handler = lookup_handler(registry, &receiver_type, method_name)
        .or_else(|| lookup_handler(registry, &fallback_type, method_name))
        .ok_or_else(|| {
            // Pull candidate method names from both the receiver type's table
            // and the Materia fallback table — the lookup itself walks both,
            // so the suggestion list should mirror that surface.
            let mut candidates: Vec<&str> = Vec::new();
            if let Some(table) = registry.get(&receiver_type) {
                candidates.extend(table.keys().map(String::as_str));
            }
            if receiver_type != fallback_type
                && let Some(table) = registry.get(&fallback_type)
            {
                candidates.extend(table.keys().map(String::as_str));
            }
            let hint = did_you_mean_hint(method_name, candidates, 3)
                .map(|h| format!(" {}", h))
                .unwrap_or_default();
            EvalError::InvalidOperation(
                format!(
                    "Method {} is not defined for {}{}",
                    method_name,
                    describe_value(&receiver_value),
                    hint
                ),
                line_info.clone(),
            )
        })?;

    handler(
        env,
        receiver_ast,
        receiver_var_name,
        receiver_value,
        args,
        line_info,
    )
}

fn lookup_handler<'a>(
    registry: &'a BuiltinMethodRegistry,
    ty: &Type,
    method_name: &str,
) -> Option<&'a BuiltinMethodHandler> {
    registry.get(ty).and_then(|table| table.get(method_name))
}

fn receiver_type(value: &Value) -> Type {
    match value {
        Value::Omen(_) => Type::Omen,
        Value::Arcana(_) => Type::Arcana,
        Value::Aether(_) => Type::Aether,
        Value::Rune(_) => Type::Rune,
        Value::Abyss => Type::Abyss,
        Value::Scroll(_) => Type::Scroll,
        Value::Lexicon(_) => Type::Lexicon,
        Value::Glyph(_) => Type::Glyph,
        Value::Artifact(handle) => Type::Artifact(handle.borrow().type_name.clone()),
    }
}

pub(super) fn method_table_for(
    registry: &mut BuiltinMethodRegistry,
    ty: Type,
) -> &mut HashMap<String, BuiltinMethodHandler> {
    registry.entry(ty).or_default()
}

pub(super) fn ensure_mutable_receiver(
    env: &RuntimeEnv,
    receiver_ast: &AST,
    receiver_var_name: Option<&str>,
    collection_kind: &str,
    method_name: &str,
    line_info: &Option<LineInfo>,
) -> Result<(), EvalError> {
    let base_name = receiver_var_name
        .map(|name| name.to_string())
        .or_else(|| collect_field_chain(receiver_ast).map(|(base, _)| base));

    let Some(var_name) = base_name else {
        return Err(EvalError::InvalidOperation(
            format!(
                "Method {}::{} requires a morph receiver, but the expression is not tied to a mutable variable",
                collection_kind, method_name
            ),
            line_info.clone(),
        ));
    };

    let var_info = env
        .get_var(&var_name)
        .ok_or_else(|| EvalError::UndefinedVariable(var_name.clone(), line_info.clone()))?;

    if var_info.is_morph {
        Ok(())
    } else {
        Err(EvalError::InvalidOperation(
            format!(
                "Cannot call {}::{} with immutable receiver '{}'",
                collection_kind, method_name, var_name
            ),
            line_info.clone(),
        ))
    }
}

pub(super) fn call_arg_to_value(
    arg: CallArg,
    context: &str,
    line_info: &Option<LineInfo>,
) -> Result<Value, EvalError> {
    match arg.value {
        EvalResult::Data(value) => Ok(value),
        EvalResult::Artifact(handle) => Ok(Value::Artifact(handle)),
        EvalResult::Revealed(_) | EvalResult::Resume(_) | EvalResult::Eject(_) => {
            Err(EvalError::InvalidOperation(
                format!("{} cannot accept control-flow results", context),
                line_info.clone(),
            ))
        }
    }
}

pub(super) fn expect_glyph_argument(
    args: Vec<CallArg>,
    method_name: &str,
    line_info: &Option<LineInfo>,
) -> Result<Type, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            format!("{} expects exactly one glyph argument", method_name),
            line_info.clone(),
        ));
    }

    let glyph_arg = args.into_iter().next().expect("argument is present");
    let glyph_value = eval_result_to_value_checked(glyph_arg.value, line_info.clone())?;
    match glyph_value {
        Value::Glyph(ty) => Ok(ty),
        other => Err(EvalError::InvalidOperation(
            format!(
                "{} argument must be a glyph value, found {}",
                method_name,
                describe_value(&other)
            ),
            line_info.clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Value;

    fn env_with_builtin_methods() -> RuntimeEnv {
        let mut env = RuntimeEnv::new();
        env.set_builtin_methods(get_all_builtin_methods());
        env
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let mut env = RuntimeEnv::new();
        let result = dispatch_builtin_method(
            &mut env,
            &AST::Abyss(None),
            None,
            Value::Abyss,
            "unknown_method",
            vec![],
            &None,
        );

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("Method unknown_method is not defined")
        ));
    }

    #[test]
    fn dispatch_unknown_method_suggests_close_match_from_receiver_table() {
        // `Type::Scroll`'s real method table includes `scribe` — a typo of
        // `"scrieb"` should surface that as a suggestion.
        let mut env = env_with_builtin_methods();
        let scroll_value = Value::Scroll(std::rc::Rc::new(std::cell::RefCell::new(vec![])));
        let err = dispatch_builtin_method(
            &mut env,
            &AST::Abyss(None),
            None,
            scroll_value,
            "scrieb",
            vec![],
            &None,
        )
        .unwrap_err();

        match err {
            EvalError::InvalidOperation(msg, _) => {
                assert!(msg.contains("scrieb"), "msg: {msg}");
                assert!(msg.contains("did you mean: scribe"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_unknown_method_falls_back_to_materia_table_for_suggestions() {
        // `trans` lives on the Materia fallback table; a typo on a non-Materia
        // receiver should still surface it as a suggestion.
        let mut env = env_with_builtin_methods();
        let scroll_value = Value::Scroll(std::rc::Rc::new(std::cell::RefCell::new(vec![])));
        let err = dispatch_builtin_method(
            &mut env,
            &AST::Abyss(None),
            None,
            scroll_value,
            "tarns",
            vec![],
            &None,
        )
        .unwrap_err();

        match err {
            EvalError::InvalidOperation(msg, _) => {
                assert!(msg.contains("did you mean: trans"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }

    #[test]
    fn dispatch_unknown_method_omits_hint_when_no_close_match() {
        let mut env = env_with_builtin_methods();
        let scroll_value = Value::Scroll(std::rc::Rc::new(std::cell::RefCell::new(vec![])));
        let err = dispatch_builtin_method(
            &mut env,
            &AST::Abyss(None),
            None,
            scroll_value,
            "completely_unrelated_method_name",
            vec![],
            &None,
        )
        .unwrap_err();

        match err {
            EvalError::InvalidOperation(msg, _) => {
                assert!(!msg.contains("did you mean"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        }
    }
}
