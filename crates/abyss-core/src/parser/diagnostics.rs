use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    error::{Rich, RichReason},
    span::Span as ChumskySpan,
};

use super::SimpleSpan;

#[derive(Debug, Clone)]
pub struct ParserDiagnostic {
    pub title: String,
    pub label: String,
    pub span: SimpleSpan<usize>,
    pub help: Option<String>,
}

pub fn convert_rich_error<'a, T, S>(error: Rich<'a, T, S>, title: &str) -> ParserDiagnostic
where
    T: std::fmt::Display + Clone,
    S: ChumskySpan<Context = (), Offset = usize> + Clone,
{
    let label = match error.reason() {
        RichReason::ExpectedFound { .. } => match error.found() {
            Some(found) => format!("Unexpected token `{found}`"),
            None => "Unexpected end of incantation".to_string(),
        },
        RichReason::Custom(msg) => msg.clone(),
    };

    let expected: Vec<String> = match error.reason() {
        RichReason::ExpectedFound { expected, .. } => {
            expected.iter().map(|pat| pat.to_string()).collect()
        }
        RichReason::Custom(_) => Vec::new(),
    };

    let help = if expected.is_empty() {
        None
    } else {
        Some(format!("Perhaps you meant one of: {}", expected.join(", ")))
    };

    let span_source = error.span().clone();
    let span = SimpleSpan::new(span_source.start(), span_source.end());

    ParserDiagnostic {
        title: title.to_string(),
        label,
        span,
        help,
    }
}

pub fn emit_diagnostics(
    source_id: &str,
    source: &str,
    diagnostics: &[ParserDiagnostic],
) -> Result<(), std::io::Error> {
    // Stream the report directly to a locked stdout handle — no
    // intermediate `String` allocation. `write_diagnostics` is the shared
    // core that `render_diagnostics` also uses with a `Vec<u8>` writer.
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    write_diagnostics(source_id, source, diagnostics, &mut handle)?;
    use std::io::Write;
    handle.flush()
}

/// Render parser diagnostics as a single string instead of writing to
/// stdout. Output includes the same `ariadne` formatting (ANSI colour
/// codes and all) that [`emit_diagnostics`] would print, so consumers
/// running on a non-CLI surface — Wasm playground, future LSP, embedded
/// REPL — can capture and post-process the bytes themselves (strip ANSI,
/// translate to HTML, etc.).
pub fn render_diagnostics(
    source_id: &str,
    source: &str,
    diagnostics: &[ParserDiagnostic],
) -> Result<String, std::io::Error> {
    let mut buffer: Vec<u8> = Vec::new();
    write_diagnostics(source_id, source, diagnostics, &mut buffer)?;
    // ariadne writes valid UTF-8 (it formats `&str` content), so the
    // conversion is infallible in practice; surface the (impossible)
    // error as a `std::io::Error` to match the stdout path's signature.
    String::from_utf8(buffer)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

/// Shared implementation for both [`emit_diagnostics`] (stdout) and
/// [`render_diagnostics`] (`Vec<u8>`). Writes each diagnostic as an
/// `ariadne` report directly into the supplied writer — the CLI path
/// streams to a locked stdout handle without buffering, while the
/// string-returning path streams into a `Vec<u8>` and converts.
fn write_diagnostics<W: std::io::Write>(
    source_id: &str,
    source: &str,
    diagnostics: &[ParserDiagnostic],
    writer: &mut W,
) -> Result<(), std::io::Error> {
    for diagnostic in diagnostics {
        let span_range = diagnostic.span.into_range();
        let mut report = Report::build(ReportKind::Error, (source_id, span_range.clone()))
            .with_message(&diagnostic.title)
            .with_label(
                Label::new((source_id, span_range))
                    .with_message(diagnostic.label.clone())
                    .with_color(Color::Red),
            );

        if let Some(help) = &diagnostic.help {
            report = report.with_help(help.clone());
        }

        report
            .finish()
            .write((source_id, Source::from(source)), &mut *writer)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_diagnostics_returns_ariadne_report_for_each_entry() {
        // Locks down the public renderer's contract for the future Wasm
        // playground / LSP consumers: it produces non-empty output that
        // mentions the title, label, and help text from each diagnostic.
        let source = "forge x = 1;";
        let diagnostics = vec![ParserDiagnostic {
            title: "Spell error: Incantation failed".into(),
            label: "Unexpected token `=`".into(),
            span: SimpleSpan::new(8, 9),
            help: Some("Perhaps you meant one of: ':'".into()),
        }];

        let rendered = render_diagnostics("<test>", source, &diagnostics)
            .expect("rendering an in-memory diagnostic should not fail");

        assert!(
            !rendered.is_empty(),
            "render_diagnostics returned empty output"
        );
        assert!(
            rendered.contains("Spell error: Incantation failed"),
            "rendered output should include the diagnostic title; got: {rendered}"
        );
        assert!(
            rendered.contains("Unexpected token `=`"),
            "rendered output should include the diagnostic label; got: {rendered}"
        );
        assert!(
            rendered.contains("Perhaps you meant one of: ':'"),
            "rendered output should include the help text; got: {rendered}"
        );
    }

    #[test]
    fn render_diagnostics_returns_empty_string_when_no_diagnostics() {
        let rendered = render_diagnostics("<test>", "", &[])
            .expect("empty diagnostic list should render successfully");
        assert!(rendered.is_empty());
    }
}
