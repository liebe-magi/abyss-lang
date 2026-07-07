use chumsky::{error::Rich, extra, prelude::*, span::SimpleSpan as ChumskySpan};
use ordered_float::OrderedFloat;
use std::fmt;

use crate::ast::Type;

use super::SimpleSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Forge,
    Morph,
    Core,
    Oracle,
    Orbit,
    Revolve,
    Eject,
    Engrave,
    Reveal,
    Artifact,
    Ward,
    Identifier(String),
    Type(Type),
    OmenLiteral(bool),
    Arcana(i64),
    Aether(OrderedFloat<f64>),
    Rune(String),
    Semicolon,
    Question,
    Colon,
    Comma,
    Arrow,
    FatArrow,
    DoubleColon,
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
    OpenBracket,
    CloseBracket,
    RangeInclusive,
    RangeExclusive,
    Dot,
}

pub type SpannedToken = (Token, SimpleSpan<usize>);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Forge => write!(f, "forge"),
            Token::Morph => write!(f, "morph"),
            Token::Core => write!(f, "core"),
            Token::Oracle => write!(f, "oracle"),
            Token::Orbit => write!(f, "orbit"),
            Token::Revolve => write!(f, "revolve"),
            Token::Eject => write!(f, "eject"),
            Token::Engrave => write!(f, "engrave"),
            Token::Reveal => write!(f, "reveal"),
            Token::Artifact => write!(f, "artifact"),
            Token::Ward => write!(f, "ward"),
            Token::Identifier(name) => write!(f, "identifier `{name}`"),
            Token::Type(ty) => write!(f, "type `{ty:?}`"),
            Token::OmenLiteral(true) => write!(f, "boon"),
            Token::OmenLiteral(false) => write!(f, "hex"),
            Token::Arcana(value) => write!(f, "arcana literal {value}"),
            Token::Aether(value) => write!(f, "aether literal {value}"),
            Token::Rune(value) => write!(f, "rune literal \"{value}\""),
            Token::Semicolon => write!(f, ";"),
            Token::Question => write!(f, "?"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::DoubleColon => write!(f, "::"),
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
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::RangeInclusive => write!(f, "..="),
            Token::RangeExclusive => write!(f, ".."),
            Token::Dot => write!(f, "."),
        }
    }
}

type LexerExtra<'src> = extra::Err<Rich<'src, char, ChumskySpan<usize>>>;

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<SpannedToken>, LexerExtra<'src>> {
    use chumsky::text;

    let sign = just::<&str, _, LexerExtra<'src>>("-")
        .to(String::from("-"))
        .or_not();

    let digits = |radix| text::digits::<_, LexerExtra<'src>>(radix).collect::<String>();

    let aether = sign
        .clone()
        .then(digits(10))
        .then_ignore(just::<&str, _, LexerExtra<'src>>("."))
        .then(digits(10))
        .map(|((sign, int_part), frac_part)| {
            let mut number = String::new();
            if let Some(sign) = sign {
                number.push_str(&sign);
            }
            number.push_str(&int_part);
            number.push('.');
            number.push_str(&frac_part);
            let value = number
                .parse::<f64>()
                .expect("parser should only construct valid f64 literals");
            Token::Aether(OrderedFloat(value))
        });

    let arcana = sign.then(digits(10)).map(|(sign, value)| {
        let mut number = String::new();
        if let Some(sign) = sign {
            number.push_str(&sign);
        }
        number.push_str(&value);
        Token::Arcana(
            number
                .parse::<i64>()
                .expect("parser should only construct valid i64 literals"),
        )
    });

    let escape = just::<char, _, LexerExtra<'src>>('\\').ignore_then(
        one_of::<_, _, LexerExtra<'src>>(r#""ntr\"#).map(|c| match c {
            '"' => '"',
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            other => other, // fallback: just use the char as is
        }),
    );
    let rune_char =
        escape.or(any::<_, LexerExtra<'src>>().filter(|c: &char| *c != '"' && *c != '\\'));
    let rune = just::<char, _, LexerExtra<'src>>('"')
        .ignore_then(rune_char.repeated().collect::<String>())
        .then_ignore(just::<char, _, LexerExtra<'src>>('"'))
        .map(Token::Rune);

    let ident = text::ident::<_, LexerExtra<'src>>().map(|ident: &'src str| match ident {
        "forge" => Token::Forge,
        "morph" => Token::Morph,
        "core" => Token::Core,
        "oracle" => Token::Oracle,
        "orbit" => Token::Orbit,
        "revolve" => Token::Revolve,
        "eject" => Token::Eject,
        "engrave" => Token::Engrave,
        "reveal" => Token::Reveal,
        "transmute" => Token::Identifier("transmute".to_string()),
        "as" => Token::Identifier("as".to_string()),
        "artifact" => Token::Artifact,
        "ward" => Token::Ward,
        "arcana" => Token::Type(Type::Arcana),
        "aether" => Token::Type(Type::Aether),
        "rune" => Token::Type(Type::Rune),
        "omen" => Token::Type(Type::Omen),
        "abyss" => Token::Type(Type::Abyss),
        "scroll" => Token::Type(Type::Scroll),
        "lexicon" => Token::Type(Type::Lexicon),
        "materia" => Token::Type(Type::Materia),
        "glyph" => Token::Type(Type::Glyph),
        "fate" => Token::Type(Type::Fate),
        "augury" => Token::Type(Type::Augury),
        "boon" => Token::OmenLiteral(true),
        "hex" => Token::OmenLiteral(false),
        _ => Token::Identifier(ident.to_string()),
    });

    let multi_char_symbols = choice((
        just::<&str, _, LexerExtra<'src>>("**=").to(Token::PowAetherAssign),
        just::<&str, _, LexerExtra<'src>>("**").to(Token::DoubleStar),
        just::<&str, _, LexerExtra<'src>>("^=").to(Token::PowArcanaAssign),
        just::<&str, _, LexerExtra<'src>>("+=").to(Token::AddAssign),
        just::<&str, _, LexerExtra<'src>>("-=").to(Token::SubAssign),
        just::<&str, _, LexerExtra<'src>>("*=").to(Token::MulAssign),
        just::<&str, _, LexerExtra<'src>>("/=").to(Token::DivAssign),
        just::<&str, _, LexerExtra<'src>>("%=").to(Token::ModAssign),
        just::<&str, _, LexerExtra<'src>>("=>").to(Token::FatArrow),
        just::<&str, _, LexerExtra<'src>>("::").to(Token::DoubleColon),
        just::<&str, _, LexerExtra<'src>>("->").to(Token::Arrow),
        just::<&str, _, LexerExtra<'src>>("||").to(Token::DoublePipe),
        just::<&str, _, LexerExtra<'src>>("&&").to(Token::DoubleAmpersand),
        just::<&str, _, LexerExtra<'src>>("==").to(Token::Equal),
        just::<&str, _, LexerExtra<'src>>("!=").to(Token::NotEqual),
        just::<&str, _, LexerExtra<'src>>("<=").to(Token::LessThanOrEqual),
        just::<&str, _, LexerExtra<'src>>(">=").to(Token::GreaterThanOrEqual),
        just::<&str, _, LexerExtra<'src>>("..=").to(Token::RangeInclusive),
        just::<&str, _, LexerExtra<'src>>("..").to(Token::RangeExclusive),
    ));

    let single_char_symbols = choice((
        just::<char, _, LexerExtra<'src>>('=').to(Token::Assign),
        just::<char, _, LexerExtra<'src>>('+').to(Token::Plus),
        just::<char, _, LexerExtra<'src>>('-').to(Token::Minus),
        just::<char, _, LexerExtra<'src>>('*').to(Token::Star),
        just::<char, _, LexerExtra<'src>>('/').to(Token::Slash),
        just::<char, _, LexerExtra<'src>>('%').to(Token::Percent),
        just::<char, _, LexerExtra<'src>>('^').to(Token::Caret),
        just::<char, _, LexerExtra<'src>>('<').to(Token::LessThan),
        just::<char, _, LexerExtra<'src>>('>').to(Token::GreaterThan),
        just::<char, _, LexerExtra<'src>>('!').to(Token::Bang),
        just::<char, _, LexerExtra<'src>>(';').to(Token::Semicolon),
        just::<char, _, LexerExtra<'src>>('?').to(Token::Question),
        just::<char, _, LexerExtra<'src>>(':').to(Token::Colon),
        just::<char, _, LexerExtra<'src>>(',').to(Token::Comma),
        just::<char, _, LexerExtra<'src>>('(').to(Token::OpenParen),
        just::<char, _, LexerExtra<'src>>(')').to(Token::CloseParen),
        just::<char, _, LexerExtra<'src>>('{').to(Token::OpenBrace),
        just::<char, _, LexerExtra<'src>>('}').to(Token::CloseBrace),
        just::<char, _, LexerExtra<'src>>('[').to(Token::OpenBracket),
        just::<char, _, LexerExtra<'src>>(']').to(Token::CloseBracket),
        just::<char, _, LexerExtra<'src>>('.').to(Token::Dot),
    ));

    let token = choice((
        aether,
        arcana,
        rune,
        ident,
        multi_char_symbols,
        single_char_symbols,
    ))
    .map_with(|tok, extra| {
        let span: ChumskySpan<usize> = extra.span();
        (tok, SimpleSpan::new(span.start(), span.end()))
    });

    token
        .padded_by(crate::parser::helpers::abyss_whitespace())
        .repeated()
        .collect()
        .then_ignore(end())
}
