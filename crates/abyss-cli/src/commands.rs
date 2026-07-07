//! One-shot subcommand implementations: `invoke` (script execution) and
//! `align` (formatting), plus the parser-diagnostic reporting shared with
//! the REPL.

use abyss_core::{
    format::format_program,
    parser::{ParserDiagnostic, collect_comments, emit_diagnostics, parse},
};
use abyss_interpreter::{
    eval::{EvalError, display_error_with_source, evaluate},
    stdlib,
};

/// Emit parser diagnostics when present. Returns `true` when diagnostics
/// were reported (i.e. the caller should stop processing the input).
pub fn report_diagnostics(source_id: &str, source: &str, diagnostics: &[ParserDiagnostic]) -> bool {
    if diagnostics.is_empty() {
        return false;
    }

    if let Err(err) = emit_diagnostics(source_id, source, diagnostics) {
        eprintln!("Failed to emit diagnostics: {err}");
    }

    true
}

/// Executes a given AbySS script by parsing and evaluating it in a new
/// environment, with `args` installed as the `invocation` scroll.
/// Returns the process exit code: 0 on success, 1 when parsing or
/// evaluation failed (the diagnostic has already been rendered), or the
/// code a `perish(code)` requested (terminating silently — perishing is
/// deliberate, not an error).
pub fn execute_script(script: &str, args: &[String]) -> i32 {
    let mut env = stdlib::create_global_environment();
    stdlib::set_invocation(&mut env, args);
    let outcome = parse(script);
    if report_diagnostics("<script>", script, &outcome.diagnostics) {
        return 1;
    }

    for ast in outcome.ast {
        match evaluate(&ast, &mut env) {
            Ok(_) => {}
            Err(EvalError::Perished(code, _)) => {
                return i32::try_from(code).unwrap_or(1);
            }
            Err(error) => {
                display_error_with_source(script, &error);
                return 1;
            }
        }
    }
    0
}

/// Formats the provided AbySS script by parsing and reconstructing it with proper indentation.
///
/// # Arguments
/// * `script` - A string containing the AbySS script to be formatted.
pub fn execute_format(script: &str) {
    let outcome = parse(script);
    if report_diagnostics("<format>", script, &outcome.diagnostics) {
        return;
    }

    let comments = collect_comments(script);
    print!("{}", format_program(script, &outcome.ast, &comments));
}
