//! One-shot subcommand implementations: `invoke` (script execution) and
//! `align` (formatting), plus the parser-diagnostic reporting shared with
//! the REPL.

use abyss_core::{
    format::format_program,
    parser::{ParserDiagnostic, collect_comments, emit_diagnostics, parse},
};
use abyss_interpreter::{
    eval::{display_error_with_source, evaluate},
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

/// Executes a given AbySS script by parsing and evaluating it in a new environment.
///
/// # Arguments
/// * `script` - A string containing the AbySS script to be executed.
pub fn execute_script(script: &str) {
    let mut env = stdlib::create_global_environment();
    let outcome = parse(script);
    if report_diagnostics("<script>", script, &outcome.diagnostics) {
        return;
    }

    for ast in outcome.ast {
        if let Err(error) = evaluate(&ast, &mut env) {
            display_error_with_source(script, &error);
            return;
        }
    }
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
