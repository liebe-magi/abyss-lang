use chumsky::prelude::*;
use ordered_float::OrderedFloat;
use std::fmt;

use crate::ast::Type;

use super::SimpleSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Forge,
    Morph,
    Oracle,
    Orbit,
    Resume,
    Eject,
    Engrave,
    Unveil,
    Reveal,
    Trans,
    As,
    Summon,
    Identifier(String),
    Type(Type),
    OmenLiteral(bool),
    Arcana(i64),
    Aether(OrderedFloat<f64>),
    Rune(String),
    Semicolon,
    Colon,
    Comma,
    Arrow,
    FatArrow,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowArcanaAssign,
    PowAetherAssign,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    DoubleStar,
    DoublePipe,
    DoubleAmpersand,
    Bang,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    RangeInclusive,
    RangeExclusive,
}

pub type SpannedToken = (Token, SimpleSpan<usize>);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Forge => write!(f, "forge"),
            Token::Morph => write!(f, "morph"),
            Token::Oracle => write!(f, "oracle"),
            Token::Orbit => write!(f, "orbit"),
            Token::Resume => write!(f, "resume"),
            Token::Eject => write!(f, "eject"),
            Token::Engrave => write!(f, "engrave"),
            Token::Unveil => write!(f, "unveil"),
            Token::Reveal => write!(f, "reveal"),
            Token::Trans => write!(f, "trans"),
            Token::As => write!(f, "as"),
            Token::Summon => write!(f, "summon"),
            Token::Identifier(name) => write!(f, "identifier `{name}`"),
            Token::Type(ty) => write!(f, "type `{ty:?}`"),
            Token::OmenLiteral(true) => write!(f, "boon"),
            Token::OmenLiteral(false) => write!(f, "hex"),
            Token::Arcana(value) => write!(f, "arcana literal {value}"),
            Token::Aether(value) => write!(f, "aether literal {value}"),
            Token::Rune(value) => write!(f, "rune literal \"{value}\""),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Assign => write!(f, "="),
            Token::AddAssign => write!(f, "+="),
            Token::SubAssign => write!(f, "-="),
            Token::MulAssign => write!(f, "*="),
            Token::DivAssign => write!(f, "/="),
            Token::ModAssign => write!(f, "%="),
            Token::PowArcanaAssign => write!(f, "^="),
            Token::PowAetherAssign => write!(f, "**="),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::LessThan => write!(f, "<"),
            Token::LessThanOrEqual => write!(f, "<="),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterThanOrEqual => write!(f, ">="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Caret => write!(f, "^"),
            Token::DoubleStar => write!(f, "**"),
            Token::DoublePipe => write!(f, "||"),
            Token::DoubleAmpersand => write!(f, "&&"),
            Token::Bang => write!(f, "!"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::RangeInclusive => write!(f, "..="),
            Token::RangeExclusive => write!(f, ".."),
        }
    }
}

pub fn lexer() -> impl Parser<char, Vec<SpannedToken>, Error = Simple<char>> {
    use chumsky::text;

    let sign = just('-').to(String::from("-")).or_not();

    let digits = text::digits(10);

    let aether = sign
        .clone()
        .then(digits.clone())
        .then_ignore(just('.'))
        .then(digits.clone())
        .map(|((sign, int_part), frac_part)| {
            let mut number = String::new();
            if let Some(sign) = sign {
                number.push_str(&sign);
            }
            number.push_str(&int_part);
            number.push('.');
            number.push_str(&frac_part);
            let value = number.parse::<f64>().unwrap();
            Token::Aether(OrderedFloat(value))
        });

    let arcana = sign.then(digits.clone()).map(|(sign, value)| {
        let mut number = String::new();
        if let Some(sign) = sign {
            number.push_str(&sign);
        }
        number.push_str(&value);
        Token::Arcana(number.parse::<i64>().unwrap())
    });

    let escape = just('\\').ignore_then(
        one_of(r#""ntr\"#).map(|c| match c {
            '"' => '"',
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            other => other, // fallback: just use the char as is
        })
    );
    let rune_char = escape.or(filter(|c| *c != '"' && *c != '\\'));
    let rune = just('"')
        .ignore_then(rune_char.repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(Token::Rune);

    let ident = text::ident().map(|ident: String| match ident.as_str() {
        "forge" => Token::Forge,
        "morph" => Token::Morph,
        "oracle" => Token::Oracle,
        "orbit" => Token::Orbit,
        "resume" => Token::Resume,
        "eject" => Token::Eject,
        "engrave" => Token::Engrave,
        "unveil" => Token::Unveil,
        "reveal" => Token::Reveal,
        "trans" => Token::Trans,
        "as" => Token::As,
        "summon" => Token::Summon,
        "arcana" => Token::Type(Type::Arcana),
        "aether" => Token::Type(Type::Aether),
        "rune" => Token::Type(Type::Rune),
        "omen" => Token::Type(Type::Omen),
        "abyss" => Token::Type(Type::Abyss),
        "boon" => Token::OmenLiteral(true),
        "hex" => Token::OmenLiteral(false),
        _ => Token::Identifier(ident),
    });

    let multi_char_symbols = choice((
        just("**=").to(Token::PowAetherAssign),
        just("**").to(Token::DoubleStar),
        just("^=").to(Token::PowArcanaAssign),
        just("+=").to(Token::AddAssign),
        just("-=").to(Token::SubAssign),
        just("*=").to(Token::MulAssign),
        just("/=").to(Token::DivAssign),
        just("%=").to(Token::ModAssign),
        just("=>").to(Token::FatArrow),
        just("->").to(Token::Arrow),
        just("||").to(Token::DoublePipe),
        just("&&").to(Token::DoubleAmpersand),
        just("==").to(Token::Equal),
        just("!=").to(Token::NotEqual),
        just("<=").to(Token::LessThanOrEqual),
        just(">=").to(Token::GreaterThanOrEqual),
        just("..=").to(Token::RangeInclusive),
        just("..").to(Token::RangeExclusive),
    ));

    let single_char_symbols = choice((
        just('=').to(Token::Assign),
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just('%').to(Token::Percent),
        just('^').to(Token::Caret),
        just('<').to(Token::LessThan),
        just('>').to(Token::GreaterThan),
        just('!').to(Token::Bang),
        just(';').to(Token::Semicolon),
        just(':').to(Token::Colon),
        just(',').to(Token::Comma),
        just('(').to(Token::OpenParen),
        just(')').to(Token::CloseParen),
        just('{').to(Token::OpenBrace),
        just('}').to(Token::CloseBrace),
    ));

    let token = choice((
        aether,
        arcana,
        rune,
        ident,
        multi_char_symbols,
        single_char_symbols,
    ))
    .map_with_span(|tok, span: std::ops::Range<usize>| (tok, SimpleSpan::from(span)));

    token
        .padded_by(crate::parser::helpers::abyss_whitespace())
        .repeated()
        .then_ignore(end())
}
