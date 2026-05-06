use std::sync::Arc;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    error::{Rich, RichReason},
    span::Span as ChumskySpan,
};

use super::SimpleSpan;
use super::helpers::LineMap;

#[derive(Debug, Clone)]
pub struct ParserDiagnostic {
    pub title: String,
    pub label: String,
    pub span: SimpleSpan<usize>,
    pub help: Option<String>,
}

pub fn convert_rich_error<'a, T, S>(
    error: Rich<'a, T, S>,
    _map: &Arc<LineMap>,
    title: &str,
) -> ParserDiagnostic
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
    let rendered = render_diagnostics(source_id, source, diagnostics)?;
    use std::io::Write;
    std::io::stdout().write_all(rendered.as_bytes())?;
    std::io::stdout().flush()
}

/// Render parser diagnostics as a single string instead of writing to
/// stdout. Output includes the same `ariadne` formatting (ANSI colour
/// codes and all) that [`emit_diagnostics`] would print, so consumers
/// running on a non-CLI surface — Wasm playground, future LSP, embedded
/// REPL — can capture and post-process the bytes themselves (strip ANSI,
/// translate to HTML, etc.). The CLI renders via [`emit_diagnostics`],
/// which is now a thin wrapper around this function.
pub fn render_diagnostics(
    source_id: &str,
    source: &str,
    diagnostics: &[ParserDiagnostic],
) -> Result<String, std::io::Error> {
    let mut buffer: Vec<u8> = Vec::new();
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
            .write((source_id, Source::from(source)), &mut buffer)?;
    }
    // ariadne writes valid UTF-8 (it formats `&str` content), so the
    // conversion is infallible in practice; surface the (impossible)
    // error as a `std::io::Error` to match the stdout path's signature.
    String::from_utf8(buffer)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}
