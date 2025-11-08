use std::sync::Arc;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::{Simple, SimpleReason};

use super::helpers::LineMap;
use super::SimpleSpan;

#[derive(Debug, Clone)]
pub struct ParserDiagnostic {
    pub title: String,
    pub label: String,
    pub span: SimpleSpan<usize>,
    pub help: Option<String>,
}

pub fn convert_simple_error<A, S>(
    error: Simple<A, S>,
    _map: &Arc<LineMap>,
    title: &str,
) -> ParserDiagnostic
where
    A: std::fmt::Display + Eq + std::hash::Hash,
    S: Into<SimpleSpan<usize>> + Clone,
{
    let label = match error.reason() {
        SimpleReason::Unexpected => match error.found() {
            Some(found) => format!("Unexpected token `{found}`"),
            None => "Unexpected end of incantation".to_string(),
        },
        SimpleReason::Unclosed { span, .. } => {
            let span: SimpleSpan<usize> = span.clone().into();
            format!(
                "Spell fragment opened here but never closed at byte {}",
                span.start()
            )
        }
        SimpleReason::Custom(msg) => msg.clone(),
    };

    let expected = error
        .expected()
        .map(|exp| match exp {
            Some(value) => format!("`{value}`"),
            None => "`<end of incantation>`".to_string(),
        })
        .collect::<Vec<_>>();

    let help = if !expected.is_empty() {
        Some(format!("Perhaps you meant one of: {}", expected.join(", ")))
    } else {
        None
    };

    ParserDiagnostic {
        title: title.to_string(),
        label,
        span: error.span().into(),
        help,
    }
}

pub fn emit_diagnostics(
    source_id: &str,
    source: &str,
    diagnostics: &[ParserDiagnostic],
) -> Result<(), std::io::Error> {
    for diagnostic in diagnostics {
        let mut report = Report::build(ReportKind::Error, source_id, diagnostic.span.start())
            .with_message(&diagnostic.title)
            .with_label(
                Label::new((source_id, diagnostic.span.start()..diagnostic.span.end()))
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
