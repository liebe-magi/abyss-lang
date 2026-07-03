pub use crate::span::SimpleSpan;
pub use helpers::{
    SourceComment, abyss_whitespace, collect_comments, scrub_comments_preserve_layout,
};
pub use tokens::{SpannedToken, Token, lexer};
mod diagnostics;
mod grammar;
mod helpers;
mod tokens;

pub use diagnostics::{ParserDiagnostic, emit_diagnostics, render_diagnostics};

use chumsky::{Parser, input::IterInput};
use ordered_float::OrderedFloat;

use crate::ast::Stmt;

use diagnostics::convert_rich_error;
use grammar::build_parser;
pub struct ParseOutcome {
    pub ast: Vec<Stmt>,
    pub diagnostics: Vec<ParserDiagnostic>,
}

pub fn parse(source: &str) -> ParseOutcome {
    let scrubbed_source = scrub_comments_preserve_layout(source);

    let (maybe_tokens, lex_errors) = lexer().parse(scrubbed_source.as_str()).into_output_errors();

    let mut diagnostics: Vec<ParserDiagnostic> = lex_errors
        .into_iter()
        .map(|err| convert_rich_error(err, "Incantation unravelled at lexing stage"))
        .collect();

    let tokens = match maybe_tokens {
        Some(tokens) => normalize_negative_literals(tokens),
        None => {
            return ParseOutcome {
                ast: Vec::new(),
                diagnostics,
            };
        }
    };

    let len = source.len();
    let token_input = IterInput::new(tokens.into_iter(), SimpleSpan::new(len, len));

    let parser = build_parser();
    let (maybe_ast, parse_errors) = parser.parse(token_input).into_output_errors();

    diagnostics.extend(
        parse_errors
            .into_iter()
            .map(|err| convert_rich_error(err, "Spell error: Incantation failed")),
    );

    let ast = maybe_ast.unwrap_or_default();

    ParseOutcome { ast, diagnostics }
}

fn normalize_negative_literals(tokens: Vec<SpannedToken>) -> Vec<SpannedToken> {
    use Token::*;

    fn allows_unary(prev: Option<&Token>) -> bool {
        match prev {
            None => true,
            Some(token) => matches!(
                token,
                Assign
                    | AddAssign
                    | SubAssign
                    | MulAssign
                    | DivAssign
                    | ModAssign
                    | PowArcanaAssign
                    | PowAetherAssign
                    | Plus
                    | Minus
                    | Star
                    | Slash
                    | Percent
                    | Caret
                    | DoubleStar
                    | DoublePipe
                    | DoubleAmpersand
                    | Equal
                    | NotEqual
                    | LessThan
                    | LessThanOrEqual
                    | GreaterThan
                    | GreaterThanOrEqual
                    | OpenParen
                    | OpenBrace
                    | Comma
                    | Colon
                    | Semicolon
                    | Arrow
                    | FatArrow
                    | Bang
            ),
        }
    }

    let mut result = Vec::with_capacity(tokens.len() + 4);
    let mut prev_token: Option<Token> = None;

    for (token, span) in tokens {
        match token {
            Arcana(value) if value < 0 && !allows_unary(prev_token.as_ref()) => {
                let abs_val = -value;
                let minus_span = SimpleSpan::new(span.start(), span.start() + 1);
                let literal_span = SimpleSpan::new(span.start() + 1, span.end());
                result.push((Minus, minus_span));
                result.push((Arcana(abs_val), literal_span));
                prev_token = Some(Arcana(abs_val));
            }
            Aether(value)
                if value < OrderedFloat::from(0.0) && !allows_unary(prev_token.as_ref()) =>
            {
                let abs_val = OrderedFloat::from(value.into_inner().abs());
                let minus_span = SimpleSpan::new(span.start(), span.start() + 1);
                let literal_span = SimpleSpan::new(span.start() + 1, span.end());
                result.push((Minus, minus_span));
                result.push((Aether(abs_val), literal_span));
                prev_token = Some(Aether(abs_val));
            }
            other => {
                prev_token = Some(other.clone());
                result.push((other, span));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;

    fn span(start: usize, end: usize) -> SimpleSpan<usize> {
        SimpleSpan::new(start, end)
    }

    #[test]
    fn splits_negative_arcana_after_value_tokens() {
        let tokens = vec![
            (Token::Arcana(1), span(0, 1)),
            (Token::Arcana(-2), span(1, 3)),
        ];

        let normalized = normalize_negative_literals(tokens);

        let expected = vec![
            (Token::Arcana(1), span(0, 1)),
            (Token::Minus, span(1, 2)),
            (Token::Arcana(2), span(2, 3)),
        ];

        assert_eq!(normalized, expected);
    }

    #[test]
    fn keeps_unary_negative_literals_intact_after_operators() {
        let tokens = vec![(Token::Minus, span(0, 1)), (Token::Arcana(-2), span(1, 3))];

        let normalized = normalize_negative_literals(tokens.clone());

        assert_eq!(normalized, tokens);
    }

    #[test]
    fn splits_negative_aether_after_closing_delimiters() {
        let tokens = vec![
            (Token::CloseParen, span(0, 1)),
            (Token::Aether(OrderedFloat::from(-1.25)), span(1, 5)),
        ];

        let normalized = normalize_negative_literals(tokens);

        let expected = vec![
            (Token::CloseParen, span(0, 1)),
            (Token::Minus, span(1, 2)),
            (Token::Aether(OrderedFloat::from(1.25)), span(2, 5)),
        ];

        assert_eq!(normalized, expected);
    }
}
