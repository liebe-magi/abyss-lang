//! Wasm adapter for the AbySS interpreter.
//!
//! Exposes a single `evaluate(source)` entry point (defined in
//! [`bindings`]) that the docs-site Playground (introduced in PR3 of the
//! v0.6.0 cycle) calls from JavaScript. Output is captured into a
//! [`String`] via the [`PlaygroundBridge`] implementation of
//! [`abyss_interpreter::io_bridge::IoBridge`], and diagnostics flow
//! through the same `render_*` renderers the CLI uses (so the
//! playground sees the same ariadne formatting the user would see in a
//! terminal — ANSI colour codes included).

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use abyss_core::parser::{parse, render_diagnostics};
use abyss_interpreter::eval::{evaluate as evaluate_ast, render_error_with_source_id};
use abyss_interpreter::io_bridge::IoBridge;
use abyss_interpreter::stdlib::create_global_environment;
use serde::Serialize;

mod bindings;

/// Result handed back to JavaScript for each `evaluate(source)` call.
///
/// `stdout` captures everything `unveil` would print on the CLI.
/// `stderr` carries the parser / runtime diagnostic, ANSI-coloured the
/// same way the CLI prints it. `error` is `Some(_)` when evaluation
/// could not complete; `stdout` may still contain partial output up to
/// the failure point.
#[derive(Debug, Default, Serialize)]
struct EvalOutcome {
    stdout: String,
    stderr: String,
    error: Option<String>,
}

/// Captures `unveil` writes into a shared `String`. `read_line` is
/// rejected so a `summon` call surfaces a clear "input is not available
/// in the Playground" error rather than blocking forever or panicking.
struct PlaygroundBridge {
    stdout: Rc<RefCell<String>>,
}

impl IoBridge for PlaygroundBridge {
    fn write_str(&mut self, content: &str) -> io::Result<()> {
        self.stdout.borrow_mut().push_str(content);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn read_line(&mut self, _buffer: &mut String) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "summon (interactive input) is not available in the Playground",
        ))
    }
}

const PLAYGROUND_SOURCE_ID: &str = "<playground>";

pub(crate) fn run(source: &str) -> EvalOutcome {
    let mut outcome = EvalOutcome::default();

    let parsed = parse(source);
    if !parsed.diagnostics.is_empty() {
        // Parser-level diagnostics are an error: nothing was evaluated,
        // so stdout stays empty. We surface the rendered report under
        // `stderr` and a short summary under `error` for the host to
        // display inline.
        match render_diagnostics(PLAYGROUND_SOURCE_ID, source, &parsed.diagnostics) {
            Ok(rendered) => outcome.stderr = rendered,
            Err(err) => outcome.stderr = format!("Failed to render diagnostics: {err}"),
        }
        outcome.error = Some(format!(
            "{} parser diagnostic(s) — see stderr for details",
            parsed.diagnostics.len()
        ));
        return outcome;
    }

    // Run with a custom bridge so `unveil` output flows into a String
    // rather than the native stdout (which is a no-op on
    // wasm32-unknown-unknown anyway). The handle stays live until after
    // `evaluate` returns, so partial output is preserved when a
    // mid-script runtime error truncates execution.
    let stdout_buf: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let bridge = PlaygroundBridge {
        stdout: stdout_buf.clone(),
    };
    let mut env = create_global_environment();
    env.set_io_bridge(Box::new(bridge));

    for ast in parsed.ast {
        if let Err(error) = evaluate_ast(&ast, &mut env) {
            // Use the same `<playground>` source id the parser
            // diagnostics carry so the labels stay consistent across
            // both rendering paths.
            outcome.stderr = render_error_with_source_id(PLAYGROUND_SOURCE_ID, source, &error);
            outcome.error = Some(format!("{}", error));
            // Drop borrow before assigning into the outcome.
            outcome.stdout = std::mem::take(&mut *stdout_buf.borrow_mut());
            return outcome;
        }
    }

    outcome.stdout = std::mem::take(&mut *stdout_buf.borrow_mut());
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_captures_unveil_output_via_bridge() {
        // The capture mechanism the Playground depends on: `unveil`
        // writes flow through `PlaygroundBridge` into a shared `String`,
        // not to `stdout`. The newline is the trailing one `unveil`
        // appends after each call.
        let outcome = run("unveil(\"hello\");");
        assert_eq!(outcome.stdout, "hello\n");
        assert!(outcome.stderr.is_empty(), "got stderr: {}", outcome.stderr);
        assert!(outcome.error.is_none(), "got error: {:?}", outcome.error);
    }

    #[test]
    fn run_surfaces_parser_diagnostics_with_no_eval() {
        // `forge x = 1` is missing a type annotation. Parsing fails;
        // `stdout` stays empty (nothing was evaluated), `stderr`
        // carries the rendered diagnostic, `error` summarises.
        let outcome = run("forge x = 1;");
        assert!(outcome.stdout.is_empty());
        assert!(!outcome.stderr.is_empty());
        assert!(outcome.error.is_some());
        let summary = outcome.error.unwrap();
        assert!(
            summary.contains("parser diagnostic"),
            "unexpected error summary: {summary}"
        );
    }

    #[test]
    fn run_preserves_partial_stdout_on_runtime_error() {
        // First `unveil` succeeds and ends up in the captured stdout;
        // the second statement hits a runtime error which truncates
        // execution. The Playground UI shows the captured "first" line
        // alongside the diagnostic, matching the CLI experience.
        let source = "unveil(\"first\");\nreveal y;";
        let outcome = run(source);
        assert_eq!(outcome.stdout, "first\n");
        assert!(!outcome.stderr.is_empty());
        assert!(outcome.error.is_some());
        let summary = outcome.error.unwrap();
        assert!(summary.contains("Variable y"), "got: {summary}");
    }

    #[test]
    fn run_rejects_summon_with_unsupported_message() {
        // Interactive input is intentionally unsupported in the
        // Playground bridge — `summon` raises a runtime error rather
        // than blocking forever or panicking. The interpreter side
        // preserves the bridge's `io::Error` message
        // (`stdlib::functions::io::native_summon`), so the
        // playground-specific text reaches the user.
        let source = r#"forge name: rune = summon("name?");"#;
        let outcome = run(source);
        assert!(outcome.error.is_some());
        let combined = format!("{}\n{}", outcome.stderr, outcome.error.unwrap());
        assert!(
            combined.contains("not available in the Playground"),
            "expected playground-specific summon error to reach the user; got: {combined}"
        );
    }
}
