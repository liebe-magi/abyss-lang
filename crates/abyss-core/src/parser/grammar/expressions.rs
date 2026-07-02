//! Expression parsers: the precedence chain (range, logical, comparison,
//! arithmetic, power), postfix suffixes (field access, method calls,
//! indexing), literals, and call syntax.

use chumsky::prelude::*;

use crate::ast::AST;

use crate::parser::SimpleSpan;
use crate::parser::tokens::Token;

use super::*;

pub(super) fn range_expr_parser<'src>(
    _ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, (AST, AST, String, SimpleSpan<usize>)> {
    let op = choice((
        just(Token::RangeInclusive).to(String::from("..=")),
        just(Token::RangeExclusive).to(String::from("..")),
    ));

    expression
        .clone()
        .then(op)
        .then(expression)
        .map(|((start, op_str), end)| {
            let span = SimpleSpan::new(start.1.start(), end.1.end());
            (start.0, end.0, op_str, span)
        })
        .boxed()
}

pub(super) fn or_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let and_expr = and_expr_parser(ctx, expression);

    and_expr
        .clone()
        .then(
            just(Token::DoublePipe)
                .ignore_then(and_expr.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, right| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                (
                    AST::LogicalOr(Box::new(left.0), Box::new(right.0), info),
                    span,
                )
            })
        })
        .boxed()
}

pub(super) fn and_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let not_expr = not_expr_parser(ctx, expression);

    not_expr
        .clone()
        .then(
            just(Token::DoubleAmpersand)
                .ignore_then(not_expr.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, right| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                (
                    AST::LogicalAnd(Box::new(left.0), Box::new(right.0), info),
                    span,
                )
            })
        })
        .boxed()
}

pub(super) fn not_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Bang)
        .map_with(|_, extra| extra.span())
        .repeated()
        .collect::<Vec<_>>()
        .then(comp_expr_parser(ctx, expression))
        .map(move |(nots, mut current)| {
            for span in nots.into_iter().rev() {
                let total_span = SimpleSpan::new(span.start(), current.1.end());
                let info = ctx_for_map.info(total_span);
                current = (AST::LogicalNot(Box::new(current.0), info), total_span);
            }
            current
        })
        .boxed()
}

pub(super) fn comp_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let add_expr = add_expr_parser(ctx, expression);

    let comparator = choice((
        just(Token::Equal),
        just(Token::NotEqual),
        just(Token::LessThanOrEqual),
        just(Token::GreaterThanOrEqual),
        just(Token::LessThan),
        just(Token::GreaterThan),
    ));

    add_expr
        .clone()
        .then(comparator.then(add_expr.clone()).or_not())
        .map(move |(left, maybe)| {
            if let Some((op_token, right)) = maybe {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::Equal => AST::Equal(Box::new(left.0), Box::new(right.0), info),
                    Token::NotEqual => AST::NotEqual(Box::new(left.0), Box::new(right.0), info),
                    Token::LessThan => AST::LessThan(Box::new(left.0), Box::new(right.0), info),
                    Token::LessThanOrEqual => {
                        AST::LessThanOrEqual(Box::new(left.0), Box::new(right.0), info)
                    }
                    Token::GreaterThan => {
                        AST::GreaterThan(Box::new(left.0), Box::new(right.0), info)
                    }
                    Token::GreaterThanOrEqual => {
                        AST::GreaterThanOrEqual(Box::new(left.0), Box::new(right.0), info)
                    }
                    _ => unreachable!(),
                };
                (ast, span)
            } else {
                left
            }
        })
        .boxed()
}

pub(super) fn add_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let mul_expr = mul_expr_parser(ctx, expression);

    mul_expr
        .clone()
        .then(
            choice((just(Token::Plus), just(Token::Minus)))
                .then(mul_expr.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, (op_token, right)| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::Plus => AST::Add(Box::new(left.0), Box::new(right.0), info),
                    Token::Minus => AST::Sub(Box::new(left.0), Box::new(right.0), info),
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

pub(super) fn mul_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let pow_expr = pow_expr_parser(ctx, expression);

    pow_expr
        .clone()
        .then(
            choice((just(Token::Star), just(Token::Slash), just(Token::Percent)))
                .then(pow_expr.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, (op_token, right)| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::Star => AST::Mul(Box::new(left.0), Box::new(right.0), info),
                    Token::Slash => AST::Div(Box::new(left.0), Box::new(right.0), info),
                    Token::Percent => AST::Mod(Box::new(left.0), Box::new(right.0), info),
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

pub(super) fn pow_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let factor = factor_parser(ctx, expression);

    factor
        .clone()
        .then(
            choice((just(Token::DoubleStar), just(Token::Caret)))
                .then(factor.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, (op_token, right)| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::DoubleStar => AST::PowAether(Box::new(left.0), Box::new(right.0), info),
                    Token::Caret => AST::PowArcana(Box::new(left.0), Box::new(right.0), info),
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

pub(super) fn factor_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let primary = primary_expr_parser(ctx.clone(), expression.clone());
    apply_postfix_suffixes(ctx, expression, primary).boxed()
}

pub(super) fn primary_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    choice((
        list_literal_parser(ctx.clone(), expression.clone()),
        map_literal_parser(ctx.clone(), expression.clone()),
        artifact_literal_parser(ctx.clone(), expression.clone()),
        literal_parser(ctx.clone()),
        func_call_parser(ctx.clone(), expression.clone()),
        identifier_node(ctx.clone()),
        expression.delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
    ))
    .boxed()
}

pub(super) enum PostfixSuffix {
    Index((AST, SimpleSpan<usize>)),
    Field((String, SimpleSpan<usize>)),
    Method(MethodSuffix),
}

pub(super) struct MethodSuffix {
    name: String,
    args: Vec<AST>,
    span: SimpleSpan<usize>,
}

pub(super) fn apply_postfix_suffixes<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
    base: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();

    base.then(
        postfix_suffix_parser(ctx.clone(), expression)
            .repeated()
            .collect::<Vec<_>>(),
    )
    .map(move |(current, suffixes)| {
        suffixes
            .into_iter()
            .fold(current, |acc, suffix| match suffix {
                PostfixSuffix::Index(index_suffix) => {
                    create_index_access(ctx_for_map.clone(), acc, index_suffix)
                }
                PostfixSuffix::Field(field_suffix) => {
                    create_field_access(ctx_for_map.clone(), acc, field_suffix)
                }
                PostfixSuffix::Method(method_suffix) => {
                    create_method_call(ctx_for_map.clone(), acc, method_suffix)
                }
            })
    })
    .boxed()
}

pub(super) fn postfix_suffix_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, PostfixSuffix> {
    let index = index_suffix_parser(ctx.clone(), expression.clone()).map(PostfixSuffix::Index);

    let method = just(Token::Dot)
        .ignore_then(
            select! { Token::Identifier(name) => name }
                .map_with(|name, extra| (name, extra.span())),
        )
        .then(
            just(Token::OpenParen)
                .map_with(|_, extra| extra.span())
                .then(
                    expression
                        .separated_by(just(Token::Comma))
                        .collect::<Vec<_>>()
                        .or_not(),
                )
                .then(just(Token::CloseParen).map_with(|_, extra| extra.span()))
                .map(|((open_span, maybe_args), close_span)| {
                    let span = SimpleSpan::new(open_span.start(), close_span.end());
                    let args = maybe_args
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(ast, _)| ast)
                        .collect::<Vec<_>>();
                    (args, span)
                })
                .or_not(),
        )
        .map(|((name, name_span), maybe_call)| {
            if let Some((args, call_span)) = maybe_call {
                let suffix_span = SimpleSpan::new(name_span.start(), call_span.end());
                PostfixSuffix::Method(MethodSuffix {
                    name,
                    args,
                    span: suffix_span,
                })
            } else {
                PostfixSuffix::Field((name, name_span))
            }
        });

    choice((index, method)).boxed()
}

pub(super) fn index_suffix_parser<'src>(
    _ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, (AST, SimpleSpan<usize>)> {
    just(Token::OpenBracket)
        .map_with(|_, extra| extra.span())
        .then(expression)
        .then(just(Token::CloseBracket).map_with(|_, extra| extra.span()))
        .map(|((open_span, (index_ast, _)), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            (index_ast, span)
        })
        .boxed()
}

pub(super) fn create_field_access(
    ctx: ParserContext,
    current: SpannedAst,
    field_suffix: (String, SimpleSpan<usize>),
) -> SpannedAst {
    let span = SimpleSpan::new(current.1.start(), field_suffix.1.end());
    let info = ctx.info(span);
    (
        AST::FieldAccess {
            target: Box::new(current.0),
            field: field_suffix.0,
            line_info: info,
        },
        span,
    )
}

pub(super) fn create_method_call(
    ctx: ParserContext,
    current: SpannedAst,
    method_suffix: MethodSuffix,
) -> SpannedAst {
    let span = SimpleSpan::new(current.1.start(), method_suffix.span.end());
    let info = ctx.info(span);
    (
        AST::MethodCall {
            receiver: Box::new(current.0),
            method: method_suffix.name,
            args: method_suffix.args,
            line_info: info,
        },
        span,
    )
}

pub(super) fn create_index_access(
    ctx: ParserContext,
    current: SpannedAst,
    index_suffix: (AST, SimpleSpan<usize>),
) -> SpannedAst {
    let span = SimpleSpan::new(current.1.start(), index_suffix.1.end());
    let info = ctx.info(span);
    (
        AST::IndexAccess {
            target: Box::new(current.0),
            index: Box::new(index_suffix.0),
            line_info: info,
        },
        span,
    )
}

pub(super) fn func_call_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then(just(Token::OpenParen).map_with(|_, extra| extra.span()))
        .then(
            expression
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .or_not(),
        )
        .then(just(Token::CloseParen).map_with(|_, extra| extra.span()))
        .map(
            move |((((name, name_span), _open_span), args_opt), close_span)| {
                let span = SimpleSpan::new(name_span.start(), close_span.end());
                let info = ctx_for_map.info(span);
                let args = args_opt
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(ast, _)| ast)
                    .collect();
                (
                    AST::FuncCall {
                        name,
                        args,
                        line_info: info,
                    },
                    span,
                )
            },
        )
        .boxed()
}

pub(super) fn list_literal_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::OpenBracket)
        .map_with(|_, extra| extra.span())
        .then(
            expression
                .clone()
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .or_not(),
        )
        .then(just(Token::CloseBracket).map_with(|_, extra| extra.span()))
        .map(move |((open_span, maybe_items), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            let elements = maybe_items
                .unwrap_or_default()
                .into_iter()
                .map(|(ast, _)| ast)
                .collect();
            (
                AST::ListLiteral {
                    elements,
                    line_info: info,
                },
                span,
            )
        })
        .boxed()
}

pub(super) fn map_literal_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let entry = select! { Token::Rune(key) => key }
        .map_with(|key, extra| (key, extra.span()))
        .then_ignore(just(Token::Colon))
        .then(expression.clone());

    just(Token::OpenBrace)
        .map_with(|_, extra| extra.span())
        .then(
            entry
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .or_not(),
        )
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(move |((open_span, maybe_entries), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            let entries = maybe_entries
                .unwrap_or_default()
                .into_iter()
                .map(|((key, _), (value_ast, _))| (key, value_ast))
                .collect();
            (
                AST::MapLiteral {
                    entries,
                    line_info: info,
                },
                span,
            )
        })
        .boxed()
}

pub(super) fn artifact_literal_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let entry = select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then_ignore(just(Token::Colon))
        .then(expression.clone());

    select! { Token::Identifier(type_name) => type_name }
        .map_with(|type_name, extra| (type_name, extra.span()))
        .then(just(Token::OpenBrace).map_with(|_, extra| extra.span()))
        .then(
            entry
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .or_not(),
        )
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(
            move |((((type_name, type_span), _open_span), maybe_fields), close_span)| {
                let span = SimpleSpan::new(type_span.start(), close_span.end());
                let info = ctx_for_map.info(span);
                let fields = maybe_fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(|((field, _), (value_ast, _))| (field, value_ast))
                    .collect();
                (
                    AST::ArtifactLiteral {
                        type_name,
                        fields,
                        line_info: info,
                    },
                    span,
                )
            },
        )
        .boxed()
}

pub(super) fn literal_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedAst> {
    let ctx_omen = ctx.clone();
    let omen = select! { Token::OmenLiteral(value) => value }.map_with(move |value, extra| {
        let span = extra.span();
        let info = ctx_omen.info(span);
        (AST::Omen(value, info), span)
    });

    let ctx_arcana = ctx.clone();
    let arcana = select! { Token::Arcana(value) => value }.map_with(move |value, extra| {
        let span = extra.span();
        let info = ctx_arcana.info(span);
        (AST::Arcana(value, info), span)
    });

    let ctx_aether = ctx.clone();
    let aether = select! { Token::Aether(value) => value }.map_with(move |value, extra| {
        let span = extra.span();
        let info = ctx_aether.info(span);
        (AST::Aether(value.into_inner(), info), span)
    });

    let ctx_rune = ctx;
    let rune = select! { Token::Rune(value) => value }.map_with(move |value, extra| {
        let span = extra.span();
        let info = ctx_rune.info(span);
        (AST::Rune(value, info), span)
    });

    choice((omen, arcana, aether, rune)).boxed()
}

pub(super) fn identifier_node<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name, Token::Core => "core".to_string(), Token::Type(ty) => type_keyword_name(&ty) }
        .map_with(move |name, extra| {
            let span = extra.span();
            let info = ctx_for_map.info(span);
            (AST::Var(name, info), span)
        })
        .boxed()
}
