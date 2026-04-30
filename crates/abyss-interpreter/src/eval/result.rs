use crate::env::{ArtifactHandle, Value};
use abyss_core::ast::LineInfo;
use ariadne::{Color, Label, Report, ReportKind, Source};
use std::fmt;

/// Represents the result of an evaluation in the interpreter.
#[derive(Debug, Clone)]
pub enum EvalResult {
    Data(Value),
    Artifact(ArtifactHandle),
    Revealed(Box<EvalResult>),
    Resume(Option<String>),
    Eject(Option<String>),
}

impl EvalResult {
    pub fn abyss() -> Self {
        EvalResult::Data(Value::Abyss)
    }

    pub fn data(value: Value) -> Self {
        EvalResult::Data(value)
    }

    pub fn artifact(handle: ArtifactHandle) -> Self {
        EvalResult::Artifact(handle)
    }
}

/// Represents possible errors that can occur during evaluation.
///
/// Marked `#[non_exhaustive]` so future variants (richer span tracking,
/// new diagnostic categories) can be added without breaking downstream
/// matchers — they will need a wildcard arm. Within this crate the
/// compiler still enforces full coverage.
#[derive(Debug)]
#[non_exhaustive]
pub enum EvalError {
    UndefinedVariable(String, Option<LineInfo>),
    InvalidOperation(String, Option<LineInfo>),
    NegativeExponent(Option<LineInfo>),
    TypeError(String, Option<LineInfo>),
}

impl EvalError {
    /// Returns the source-position metadata attached to this error, if any.
    pub fn line_info(&self) -> Option<&LineInfo> {
        match self {
            EvalError::UndefinedVariable(_, info)
            | EvalError::InvalidOperation(_, info)
            | EvalError::TypeError(_, info)
            | EvalError::NegativeExponent(info) => info.as_ref(),
        }
    }

    /// Short, human-readable category label used as the report header in
    /// [`display_error_with_source`].
    fn kind_label(&self) -> &'static str {
        match self {
            EvalError::UndefinedVariable(_, _) => "Undefined variable",
            EvalError::InvalidOperation(_, _) => "Invalid operation",
            EvalError::NegativeExponent(_) => "Negative exponent",
            EvalError::TypeError(_, _) => "Type error",
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UndefinedVariable(var, _) => write!(f, "Variable {} is not defined!", var),
            EvalError::InvalidOperation(op, _) => write!(f, "Invalid operation: {}", op),
            EvalError::NegativeExponent(_) => {
                write!(f, "PowArcana operation requires a non-negative exponent!")
            }
            EvalError::TypeError(var_type, _) => write!(f, "Type error: {}", var_type),
        }
    }
}

impl std::error::Error for EvalError {}

/// Default source identifier used by [`display_error_with_source`]. Mirrors
/// the placeholder the CLI passes to the parser so file invocations show a
/// consistent header across parser and runtime diagnostics.
pub const RUNTIME_SOURCE_ID: &str = "<script>";

/// Render a runtime error against the source script using the default
/// runtime source id (`"<script>"`).
///
/// For callers that want diagnostics to reflect a different context — REPL
/// buffer, test harness input, or an actual file path — use
/// [`display_error_with_source_id`] instead so the parser and runtime
/// reports stay consistent.
pub fn display_error_with_source(script: &str, error: &EvalError) {
    display_error_with_source_id(RUNTIME_SOURCE_ID, script, error);
}

/// Render a runtime error against the source script using
/// [`ariadne`]'s annotated report style — matching the parser's diagnostic
/// look. When the error carries a [`LineInfo`] that resolves into the source,
/// the offending column is underlined; otherwise a plain "Error: …" line is
/// printed to stderr as a fallback.
///
/// `source_id` controls the label shown by the diagnostic renderer (e.g.
/// `"<script>"`, `"<repl>"`, `"<test>"`, or a file path) so runtime reports
/// stay consistent with the parser diagnostics produced for the same input.
///
/// Output goes to stderr. Any I/O failure from ariadne is swallowed (matching
/// the pre-migration `eprintln!`-based behaviour).
pub fn display_error_with_source_id(source_id: &str, script: &str, error: &EvalError) {
    let message = error.to_string();
    if let Some(info) = error.line_info()
        && let Some(start) = line_col_to_byte_offset(script, info.line, info.column)
    {
        // Single-byte highlight at the reported column, clamped so the span
        // never extends past the source. `LineInfo` carries no end position
        // today, so finer-grained highlighting requires the larger Span
        // refactor planned for a later release.
        let end = (start + 1).min(script.len()).max(start);
        let span = start..end;
        let report = Report::build(ReportKind::Error, (source_id, span.clone()))
            .with_message(error.kind_label())
            .with_label(
                Label::new((source_id, span))
                    .with_message(&message)
                    .with_color(Color::Red),
            )
            .finish();
        let _ = report.eprint((source_id, Source::from(script)));
    } else {
        eprintln!("Error: {}", message);
    }
}

/// Convert a 1-based `(line, column)` pair (where `column` is a byte offset
/// within the line, matching the parser's `LineMap`) into a byte offset
/// suitable for ariadne's range-based labels.
///
/// Returns `None` when the position cannot be resolved against `source` —
/// this includes columns past the end of the named line, which would
/// otherwise spill into the next line and underline an unrelated character.
fn line_col_to_byte_offset(source: &str, line: usize, column: usize) -> Option<usize> {
    if line == 0 || column == 0 {
        return None;
    }
    let mut line_starts: Vec<usize> = vec![0];
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            line_starts.push(i + ch.len_utf8());
        }
    }
    let line_start = *line_starts.get(line - 1)?;
    // Stop the column at the end of its own line. For non-last lines that's
    // the offset of the next line's first byte (just past the newline); for
    // the last line we allow a one-past-end (EOF) position so errors
    // attached to the final character still resolve.
    let line_end_exclusive = line_starts.get(line).copied().unwrap_or(source.len() + 1);
    let offset = line_start + (column - 1);
    if offset >= line_end_exclusive || offset > source.len() {
        return None;
    }
    Some(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::ArtifactValue;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn sample_handle(name: &str) -> ArtifactHandle {
        Rc::new(RefCell::new(ArtifactValue {
            type_name: name.to_string(),
            fields: HashMap::new(),
            field_order: Vec::new(),
        }))
    }

    #[test]
    fn abyss_constructor_returns_abyss_value() {
        match EvalResult::abyss() {
            EvalResult::Data(Value::Abyss) => {}
            other => panic!("expected abyss value, got {:?}", other),
        }
    }

    #[test]
    fn data_constructor_wraps_value() {
        match EvalResult::data(Value::Arcana(42)) {
            EvalResult::Data(Value::Arcana(v)) => assert_eq!(v, 42),
            other => panic!("expected arcana value, got {:?}", other),
        }
    }

    #[test]
    fn artifact_constructor_preserves_handle() {
        let handle = sample_handle("Sigil");
        match EvalResult::artifact(handle.clone()) {
            EvalResult::Artifact(result_handle) => {
                assert!(Rc::ptr_eq(&result_handle, &handle))
            }
            other => panic!("expected artifact handle, got {:?}", other),
        }
    }

    #[test]
    fn line_info_returns_attached_position() {
        let err = EvalError::TypeError("x".into(), Some(LineInfo::new(3, 4)));
        let info = err.line_info().expect("line info attached");
        assert_eq!(info.line, 3);
        assert_eq!(info.column, 4);

        let plain = EvalError::NegativeExponent(None);
        assert!(plain.line_info().is_none());
    }

    #[test]
    fn line_col_to_byte_offset_resolves_simple_positions() {
        let source = "abc\ndef\nghi";
        assert_eq!(line_col_to_byte_offset(source, 1, 1), Some(0));
        assert_eq!(line_col_to_byte_offset(source, 2, 1), Some(4));
        assert_eq!(line_col_to_byte_offset(source, 3, 2), Some(9));
    }

    #[test]
    fn line_col_to_byte_offset_rejects_out_of_range() {
        let source = "abc\ndef";
        assert_eq!(line_col_to_byte_offset(source, 0, 1), None);
        assert_eq!(line_col_to_byte_offset(source, 1, 0), None);
        assert_eq!(line_col_to_byte_offset(source, 5, 1), None);
        // Column past end of source returns None.
        assert!(line_col_to_byte_offset(source, 2, 100).is_none());
    }

    #[test]
    fn line_col_to_byte_offset_rejects_column_past_line_end() {
        // Line 1 contains "abc" (3 bytes) followed by a newline; column 5
        // would otherwise spill into "def" on the next line. Reject it so
        // the highlight cannot point at an unrelated character.
        let source = "abc\ndef";
        assert_eq!(line_col_to_byte_offset(source, 1, 4), Some(3));
        assert!(line_col_to_byte_offset(source, 1, 5).is_none());
    }

    #[test]
    fn line_col_to_byte_offset_handles_multibyte_chars() {
        // "あ" is 3 bytes in UTF-8; the second line starts after the newline
        // following it.
        let source = "あ\nb";
        assert_eq!(line_col_to_byte_offset(source, 1, 1), Some(0));
        assert_eq!(line_col_to_byte_offset(source, 2, 1), Some(4));
    }

    #[test]
    fn display_error_with_valid_line_does_not_panic() {
        let script = "sigil = 1\nhex = sigil + 2";
        let err = EvalError::InvalidOperation("invalid op".into(), Some(LineInfo::new(2, 5)));
        display_error_with_source(script, &err);
    }

    #[test]
    fn display_error_without_matching_line_falls_back_to_generic() {
        let err = EvalError::TypeError("out of range".into(), Some(LineInfo::new(99, 1)));
        display_error_with_source("sigil = 1", &err);
    }

    #[test]
    fn display_error_without_line_info_still_prints_message() {
        let err = EvalError::NegativeExponent(None);
        display_error_with_source("sigil = 1", &err);
    }

    #[test]
    fn kind_label_covers_every_variant() {
        // If a new variant is added the compiler will flag this match
        // (`#[non_exhaustive]` only restricts external matching); keeping
        // this test guards against silently shipping a variant without a
        // human-readable label.
        let cases: [EvalError; 4] = [
            EvalError::UndefinedVariable("x".into(), None),
            EvalError::InvalidOperation("y".into(), None),
            EvalError::NegativeExponent(None),
            EvalError::TypeError("z".into(), None),
        ];
        for err in &cases {
            assert!(!err.kind_label().is_empty());
        }
    }
}
