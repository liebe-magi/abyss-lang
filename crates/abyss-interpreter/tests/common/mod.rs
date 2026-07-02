//! Shared driver for the integration suites: parses a source string and
//! evaluates every statement against a fresh global environment. Lives in
//! `tests/common/` (a directory, not a top-level file) so Cargo does not
//! compile it as a test target of its own.

use abyss_core::parser::{emit_diagnostics, parse};
use abyss_interpreter::{
    eval::{EvalResult, display_error_with_source_id, evaluate},
    stdlib,
};

const TEST_SOURCE_ID: &str = "<test>";

pub fn test_base(input: &str) -> Result<Vec<EvalResult>, Box<dyn std::error::Error>> {
    let mut env = stdlib::create_global_environment();
    let outcome = parse(input);

    if !outcome.diagnostics.is_empty() {
        emit_diagnostics(TEST_SOURCE_ID, input, &outcome.diagnostics)
            .expect("failed to emit parser diagnostics");
        panic!("Parser emitted diagnostics for test input");
    }

    let mut results = Vec::new();
    for ast in outcome.ast {
        match evaluate(&ast, &mut env) {
            Ok(result) => results.push(result),
            Err(e) => {
                display_error_with_source_id(TEST_SOURCE_ID, input, &e);
                return Err(Box::new(e));
            }
        }
    }

    Ok(results)
}
