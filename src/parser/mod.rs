pub use helpers::{abyss_whitespace, attach_line_info, LineMap};
pub use span::SimpleSpan;
pub use tokens::{lexer, SpannedToken, Token};
mod diagnostics;
mod grammar;
mod helpers;
mod span;
mod tokens;

pub use diagnostics::{emit_diagnostics, ParserDiagnostic};

use std::sync::Arc;

use chumsky::Parser;
use chumsky::Stream;

use crate::ast::AST;

use diagnostics::convert_simple_error;
use grammar::build_parser;
pub struct ParseOutcome {
    pub ast: Vec<AST>,
    pub diagnostics: Vec<ParserDiagnostic>,
}

pub fn parse(source: &str) -> ParseOutcome {
    let map = Arc::new(LineMap::new(source));

    let (maybe_tokens, lex_errors) = lexer().parse_recovery(source);

    let mut diagnostics: Vec<ParserDiagnostic> = lex_errors
        .into_iter()
        .map(|err| convert_simple_error(err, &map, "Incantation unravelled at lexing stage"))
        .collect();

    let tokens = match maybe_tokens {
        Some(tokens) => tokens,
        None => {
            return ParseOutcome {
                ast: Vec::new(),
                diagnostics,
            };
        }
    };

    let len = source.len();
    let token_stream = Stream::from_iter(SimpleSpan::new(len, len), tokens.iter().cloned());

    let parser = build_parser(map.clone());
    let (maybe_ast, parse_errors) = parser.parse_recovery(token_stream);

    diagnostics.extend(
        parse_errors
            .into_iter()
            .map(|err| convert_simple_error(err, &map, "Spell error: Incantation failed")),
    );

    let ast = maybe_ast.unwrap_or_default();

    ParseOutcome { ast, diagnostics }
}
