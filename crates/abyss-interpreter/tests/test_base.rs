use abyss_core::parser::{emit_diagnostics, parse};
use abyss_interpreter::{
    eval::{EvalError, EvalResult, display_error_with_source, evaluate},
    stdlib,
};

#[allow(unused_imports)]
pub use abyss_interpreter::env::Value;

pub fn test_base(input: &str) -> Result<Vec<EvalResult>, Box<dyn std::error::Error>> {
    let mut env = stdlib::create_global_environment();
    let outcome = parse(input);

    if !outcome.diagnostics.is_empty() {
        emit_diagnostics("<test>", input, &outcome.diagnostics)
            .expect("failed to emit parser diagnostics");
        panic!("Parser emitted diagnostics for test input");
    }

    let mut results = Vec::new();
    for ast in outcome.ast {
        match evaluate(&ast, &mut env) {
            Ok(result) => results.push(result),
            Err(e) => {
                let error_message = e.to_string();
                match &e {
                    EvalError::UndefinedVariable(_, line_info)
                    | EvalError::InvalidOperation(_, line_info)
                    | EvalError::NegativeExponent(line_info)
                    | EvalError::TypeError(_, line_info) => {
                        display_error_with_source(input, line_info.clone(), &error_message);
                    }
                }
                return Err(Box::new(e));
            }
        }
    }

    Ok(results)
}
