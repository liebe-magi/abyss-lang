use abyss_lang::parser::{SimpleSpan, SpannedToken, Token, lexer};
use chumsky::Parser;
use ordered_float::OrderedFloat;

fn lex(source: &str) -> Vec<SpannedToken> {
    lexer()
        .parse(source)
        .into_result()
        .expect("lexer should succeed on valid input")
}

fn span_for(source: &str, snippet: &str) -> SimpleSpan<usize> {
    let start = source
        .find(snippet)
        .unwrap_or_else(|| panic!("snippet `{}` not found in source", snippet));
    SimpleSpan::new(start, start + snippet.len())
}

#[test]
fn lexes_rune_escape_sequences() {
    let tokens = lex("\"line\\n\\\"quote\\t\\\\\"");
    assert_eq!(tokens.len(), 1);
    match &tokens[0].0 {
        Token::Rune(value) => assert_eq!(value, "line\n\"quote\t\\"),
        other => panic!("expected rune literal, found {other:?}"),
    }
}

#[test]
fn lexes_multi_character_symbols_in_order() {
    let source = "**= ** ^= += -= *= /= %= => :: -> || && == != <= >= ..= ..";
    let tokens: Vec<Token> = lex(source).into_iter().map(|(tok, _)| tok).collect();

    let expected = vec![
        Token::PowAetherAssign,
        Token::DoubleStar,
        Token::PowArcanaAssign,
        Token::AddAssign,
        Token::SubAssign,
        Token::MulAssign,
        Token::DivAssign,
        Token::ModAssign,
        Token::FatArrow,
        Token::DoubleColon,
        Token::Arrow,
        Token::DoublePipe,
        Token::DoubleAmpersand,
        Token::Equal,
        Token::NotEqual,
        Token::LessThanOrEqual,
        Token::GreaterThanOrEqual,
        Token::RangeInclusive,
        Token::RangeExclusive,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn lexes_keywords_numbers_and_spans() {
    let source = "forge arcana boon hex foo123 -42 3.50;";
    let tokens = lex(source);

    let mut iter = tokens.iter();
    assert!(matches!(
        iter.next().map(|(tok, _)| tok),
        Some(Token::Forge)
    ));
    assert!(matches!(
        iter.next().map(|(tok, _)| tok),
        Some(Token::Type(_))
    ));
    assert!(matches!(
        iter.next().map(|(tok, _)| tok),
        Some(Token::OmenLiteral(true))
    ));
    assert!(matches!(
        iter.next().map(|(tok, _)| tok),
        Some(Token::OmenLiteral(false))
    ));

    match iter.next() {
        Some((Token::Identifier(name), _)) => assert_eq!(name, "foo123"),
        other => panic!("expected identifier token, found {other:?}"),
    }

    match iter.next() {
        Some((Token::Arcana(value), span)) => {
            assert_eq!(*value, -42);
            assert_eq!(*span, span_for(source, "-42"));
        }
        other => panic!("expected arcana token, found {other:?}"),
    }

    match iter.next() {
        Some((Token::Aether(value), span)) => {
            assert_eq!(*value, OrderedFloat::from(3.50));
            assert_eq!(*span, span_for(source, "3.50"));
        }
        other => panic!("expected aether token, found {other:?}"),
    }

    assert!(matches!(
        iter.next().map(|(tok, _)| tok),
        Some(Token::Semicolon)
    ));
    assert!(iter.next().is_none());
}
