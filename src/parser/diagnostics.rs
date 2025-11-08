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

        report.finish().print((source_id, Source::from(source)))?;
    }
    Ok(())
}
