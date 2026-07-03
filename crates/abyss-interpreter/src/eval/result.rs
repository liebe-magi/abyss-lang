use crate::env::{ArtifactHandle, Value};
use abyss_core::ast::{Span, Type};
use ariadne::{Color, Label, Report, ReportKind, Source};
use std::fmt;

/// Represents the result of an evaluation in the interpreter.
///
/// Artifacts have no dedicated variant: they travel as
/// `Data(Value::Artifact(_))` like every other value, so consumers only
/// need to distinguish data from the control-flow variants
/// (`Revealed` / `Revolve` / `Eject`).
#[derive(Debug, Clone)]
pub enum EvalResult {
    Data(Value),
    Revealed(Box<EvalResult>),
    Revolve(Option<String>),
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
        EvalResult::Data(Value::Artifact(handle))
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
    UndefinedVariable(String, Option<Span>),
    InvalidOperation(String, Option<Span>),
    NegativeExponent(Option<Span>),
    TypeError(String, Option<Span>),
    /// A value of the given type was required but something else was
    /// produced (variable initialisation, argument conversion, builtin
    /// extraction). For `Type::Artifact` the message names the artifact
    /// type; use [`EvalError::ArtifactTypeMismatch`] when the *found*
    /// artifact type is also known.
    ExpectedType(Type, Option<Span>),
    /// An artifact of one type appeared where another artifact type was
    /// required.
    ArtifactTypeMismatch {
        expected: String,
        found: String,
        line_info: Option<Span>,
    },
    /// Assignment to a variable that was not declared `morph`.
    ImmutableAssignment(String, Option<Span>),
    /// Scroll index outside the valid range.
    ScrollIndexOutOfBounds(usize, Option<Span>),
    /// Lexicon lookup with a key that has no entry.
    MissingLexiconKey(String, Option<Span>),
}

/// Lowercase keyword name for a type, as used in "Expected … value"
/// messages (`arcana`, `rune`, …).
fn type_label(ty: &Type) -> &str {
    match ty {
        Type::Arcana => "arcana",
        Type::Aether => "aether",
        Type::Rune => "rune",
        Type::Omen => "omen",
        Type::Abyss => "abyss",
        Type::Scroll => "scroll",
        Type::Lexicon => "lexicon",
        Type::Glyph => "glyph",
        Type::Materia => "materia",
        Type::Artifact(name) => name,
    }
}

impl EvalError {
    /// Returns the source-position metadata attached to this error, if any.
    pub fn line_info(&self) -> Option<&Span> {
        match self {
            EvalError::UndefinedVariable(_, info)
            | EvalError::InvalidOperation(_, info)
            | EvalError::TypeError(_, info)
            | EvalError::ExpectedType(_, info)
            | EvalError::ArtifactTypeMismatch {
                line_info: info, ..
            }
            | EvalError::ImmutableAssignment(_, info)
            | EvalError::ScrollIndexOutOfBounds(_, info)
            | EvalError::MissingLexiconKey(_, info)
            | EvalError::NegativeExponent(info) => info.as_ref(),
        }
    }

    /// Short, human-readable category label used as the report header in
    /// [`display_error_with_source`].
    fn kind_label(&self) -> &'static str {
        match self {
            EvalError::UndefinedVariable(_, _) => "Undefined variable",
            EvalError::InvalidOperation(_, _)
            | EvalError::ImmutableAssignment(_, _)
            | EvalError::ScrollIndexOutOfBounds(_, _)
            | EvalError::MissingLexiconKey(_, _) => "Invalid operation",
            EvalError::NegativeExponent(_) => "Negative exponent",
            EvalError::TypeError(_, _)
            | EvalError::ExpectedType(_, _)
            | EvalError::ArtifactTypeMismatch { .. } => "Type error",
        }
    }
}

// The structured variants render byte-identically to the strings the
// legacy `InvalidOperation(String)` / `TypeError(String)` sites produced,
// so published docs and the VS Code extension keep matching. Wording
// changes are deliberate decisions, made here and nowhere else.
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UndefinedVariable(var, _) => write!(f, "Variable {} is not defined!", var),
            EvalError::InvalidOperation(op, _) => write!(f, "Invalid operation: {}", op),
            EvalError::NegativeExponent(_) => {
                write!(f, "PowArcana operation requires a non-negative exponent!")
            }
            EvalError::TypeError(var_type, _) => write!(f, "Type error: {}", var_type),
            EvalError::ExpectedType(Type::Artifact(name), _) => {
                write!(f, "Type error: Expected artifact of type {}", name)
            }
            EvalError::ExpectedType(ty, _) => {
                write!(f, "Type error: Expected {} value", type_label(ty))
            }
            EvalError::ArtifactTypeMismatch {
                expected, found, ..
            } => write!(
                f,
                "Type error: Expected artifact of type {} but received {}",
                expected, found
            ),
            EvalError::ImmutableAssignment(name, _) => write!(
                f,
                "Invalid operation: Cannot reassign to immutable variable {}",
                name
            ),
            EvalError::ScrollIndexOutOfBounds(index, _) => write!(
                f,
                "Invalid operation: Index {} is out of bounds for scroll",
                index
            ),
            EvalError::MissingLexiconKey(key, _) => {
                write!(f, "Invalid operation: Lexicon key '{}' does not exist", key)
            }
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
/// look. When the error carries a [`Span`] that resolves into the source,
/// the full offending range is underlined; otherwise a plain "Error: …" line
/// is printed to stderr as a fallback.
///
/// `source_id` controls the label shown by the diagnostic renderer (e.g.
/// `"<script>"`, `"<repl>"`, `"<test>"`, or a file path) so runtime reports
/// stay consistent with the parser diagnostics produced for the same input.
///
/// Output is streamed directly to a locked stderr handle — no intermediate
/// `String` allocation, and any I/O failure (broken pipe, etc.) is silently
/// dropped to preserve the non-panicking behaviour the pre-migration
/// `eprintln!`-based path had. Use [`render_error_with_source_id`] when
/// you want the rendered output as a string instead — for example, in a
/// Wasm playground or a future LSP.
pub fn display_error_with_source_id(source_id: &str, script: &str, error: &EvalError) {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    // The CLI path explicitly drops I/O errors so a closed pipe doesn't
    // panic the interpreter — matching the original `report.eprint(...)`
    // semantics that this refactor replaced.
    let _ = write_error_with_source_id(source_id, script, error, &mut handle);
}

/// Same default-source-id rendering as [`display_error_with_source`], but
/// returns the formatted output as a string instead of writing to stderr.
/// Useful for non-CLI consumers (Wasm playground, LSP, embedded REPL).
pub fn render_error_with_source(script: &str, error: &EvalError) -> String {
    render_error_with_source_id(RUNTIME_SOURCE_ID, script, error)
}

/// Render a runtime error to a `String` using the same `ariadne` formatting
/// (ANSI colour codes and all) that [`display_error_with_source_id`] would
/// print to stderr. The trailing newline is included where ariadne emits
/// one; the positionless fallback path appends `"Error: …\n"` to stay
/// byte-identical with the stderr behaviour. Consumers running on a non-CLI
/// surface — Wasm playground, future LSP, embedded REPL — can capture the
/// bytes and post-process them (strip ANSI, translate to HTML, etc.).
pub fn render_error_with_source_id(source_id: &str, script: &str, error: &EvalError) -> String {
    let mut buffer: Vec<u8> = Vec::new();
    // Writing into a `Vec<u8>` is infallible in practice; on the (impossible)
    // I/O failure path we still want a string back rather than a `Result`,
    // so fall back to the bare `Error: …` form so the caller has something
    // useful to display.
    if write_error_with_source_id(source_id, script, error, &mut buffer).is_err() {
        return format!("Error: {}\n", error);
    }
    String::from_utf8(buffer).unwrap_or_else(|err| {
        format!(
            "Error: {} (rendering produced invalid UTF-8: {})",
            error, err
        )
    })
}

/// Shared implementation for both [`display_error_with_source_id`] (stderr)
/// and [`render_error_with_source_id`] (`Vec<u8>`). Streams an `ariadne`
/// report directly into the supplied writer — the CLI path streams to a
/// locked stderr handle without buffering, while the string-returning
/// path streams into a `Vec<u8>` and converts.
fn write_error_with_source_id<W: std::io::Write>(
    source_id: &str,
    script: &str,
    error: &EvalError,
    writer: &mut W,
) -> std::io::Result<()> {
    let message = error.to_string();
    if let Some(info) = error.line_info()
        && info.start() <= script.len()
    {
        // Underline the full span the error carries, clamped so the range
        // never extends past the source. A zero-width span is widened to
        // one byte so ariadne still draws a caret at the position.
        let start = info.start();
        let end = info.end().clamp(start + 1, script.len().max(start + 1));
        let span = start..end;
        let report = Report::build(ReportKind::Error, (source_id, span.clone()))
            .with_message(error.kind_label())
            .with_label(
                Label::new((source_id, span))
                    .with_message(&message)
                    .with_color(Color::Red),
            )
            .finish();
        report.write((source_id, Source::from(script)), writer)
    } else {
        writeln!(writer, "Error: {}", message)
    }
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
            EvalResult::Data(Value::Artifact(result_handle)) => {
                assert!(Rc::ptr_eq(&result_handle, &handle))
            }
            other => panic!("expected artifact handle, got {:?}", other),
        }
    }

    /// Pins the Display contract of the structured variants: these strings
    /// are referenced by published docs and the VS Code extension, and must
    /// stay byte-identical to the legacy stringly-typed messages.
    #[test]
    fn structured_variants_render_legacy_messages() {
        let info = Some(Span::new(3, 7));

        let cases: Vec<(EvalError, &str, &str)> = vec![
            (
                EvalError::ExpectedType(Type::Arcana, info),
                "Type error: Expected arcana value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Aether, info),
                "Type error: Expected aether value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Rune, info),
                "Type error: Expected rune value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Omen, info),
                "Type error: Expected omen value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Abyss, info),
                "Type error: Expected abyss value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Scroll, info),
                "Type error: Expected scroll value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Lexicon, info),
                "Type error: Expected lexicon value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Glyph, info),
                "Type error: Expected glyph value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Materia, info),
                "Type error: Expected materia value",
                "Type error",
            ),
            (
                EvalError::ExpectedType(Type::Artifact("Player".into()), info),
                "Type error: Expected artifact of type Player",
                "Type error",
            ),
            (
                EvalError::ArtifactTypeMismatch {
                    expected: "Player".into(),
                    found: "Enemy".into(),
                    line_info: info,
                },
                "Type error: Expected artifact of type Player but received Enemy",
                "Type error",
            ),
            (
                EvalError::ImmutableAssignment("sigil".into(), info),
                "Invalid operation: Cannot reassign to immutable variable sigil",
                "Invalid operation",
            ),
            (
                EvalError::ScrollIndexOutOfBounds(9, info),
                "Invalid operation: Index 9 is out of bounds for scroll",
                "Invalid operation",
            ),
            (
                EvalError::MissingLexiconKey("port".into(), info),
                "Invalid operation: Lexicon key 'port' does not exist",
                "Invalid operation",
            ),
        ];

        for (err, expected_display, expected_kind) in cases {
            assert_eq!(err.to_string(), expected_display);
            assert_eq!(err.kind_label(), expected_kind);
            let returned = err.line_info().expect("span should be attached");
            assert_eq!((returned.start(), returned.end()), (3, 7));
        }
    }

    #[test]
    fn line_info_returns_attached_position() {
        let err = EvalError::TypeError("x".into(), Some(Span::new(3, 4)));
        let info = err.line_info().expect("span attached");
        assert_eq!((info.start(), info.end()), (3, 4));

        let plain = EvalError::NegativeExponent(None);
        assert!(plain.line_info().is_none());
    }

    #[test]
    fn display_error_with_valid_line_does_not_panic() {
        let script = "sigil = 1\nhex = sigil + 2";
        let err = EvalError::InvalidOperation("invalid op".into(), Some(Span::new(2, 5)));
        display_error_with_source(script, &err);
    }

    #[test]
    fn display_error_without_matching_line_falls_back_to_generic() {
        let err = EvalError::TypeError("out of range".into(), Some(Span::new(99, 1)));
        display_error_with_source("sigil = 1", &err);
    }

    #[test]
    fn display_error_without_line_info_still_prints_message() {
        let err = EvalError::NegativeExponent(None);
        display_error_with_source("sigil = 1", &err);
    }

    #[test]
    fn render_error_with_resolvable_position_returns_ariadne_report() {
        // The string-returning renderer is the surface the future Wasm
        // playground / LSP capture, so this locks down two things: it
        // returns a non-empty `String`, and the rendered output mentions
        // the underlying message rather than swallowing it.
        let script = "sigil: arcana = 1\nhex = sigil + 2";
        let err = EvalError::InvalidOperation("invalid op".into(), Some(Span::new(2, 5)));
        let rendered = render_error_with_source(script, &err);
        assert!(!rendered.is_empty(), "ariadne rendering produced no output");
        assert!(
            rendered.contains("invalid op"),
            "rendered output should mention the message; got {rendered}"
        );
    }

    #[test]
    fn render_error_without_position_falls_back_to_plain_line() {
        // With no `Span` the renderer drops down to the
        // `Error: …\n` fallback (matching what `display_error_with_source`
        // would print).
        let err = EvalError::NegativeExponent(None);
        let rendered = render_error_with_source("sigil = 1", &err);
        assert!(rendered.starts_with("Error: "), "got: {rendered}");
        assert!(rendered.ends_with('\n'));
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
