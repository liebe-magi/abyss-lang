use chumsky::{
    error::Rich,
    extra,
    input::IterInput,
    prelude::*,
    recursive::{self, Direct},
};

use crate::ast::{AST, Span};

use super::SimpleSpan;
use super::tokens::{SpannedToken, Token};

pub(super) type ParserInput<'src> = IterInput<std::vec::IntoIter<SpannedToken>, SimpleSpan<usize>>;
pub(super) type ParserError<'src> = Rich<'src, Token, SimpleSpan<usize>>;
pub(super) type ParserExtra<'src> = extra::Full<ParserError<'src>, (), ()>;
pub(super) type BoxedParser<'src, T> =
    chumsky::Boxed<'src, 'src, ParserInput<'src>, T, ParserExtra<'src>>;
pub(super) type RecursiveParser<'src, T> =
    recursive::Recursive<Direct<'src, 'src, ParserInput<'src>, T, ParserExtra<'src>>>;
pub(super) type SpannedAst = (AST, SimpleSpan<usize>);
pub(super) type IndexedTarget = (SpannedAst, SpannedAst);

/// Shared per-parser context. Since the span refactor it carries no
/// state — nodes store byte spans directly — but it is kept as the
/// construction point for node positions so a future context (e.g.
/// interning, config) has an obvious home.
#[derive(Clone)]
pub(super) struct ParserContext {}

impl ParserContext {
    fn info(&self, span: SimpleSpan<usize>) -> Option<Span> {
        Some(span)
    }

    fn wrap_statement(&self, ast: AST, span: SimpleSpan<usize>) -> AST {
        AST::Statement(Box::new(ast), self.info(span))
    }
}

pub(super) fn merge_span(a: SimpleSpan<usize>, b: SimpleSpan<usize>) -> SimpleSpan<usize> {
    SimpleSpan::new(a.start().min(b.start()), a.end().max(b.end()))
}

pub fn build_parser<'src>() -> BoxedParser<'src, Vec<AST>> {
    let ctx = ParserContext {};

    let ctx_for_recursive = ctx.clone();

    let statement = recursive(|statement| {
        let ctx = ctx_for_recursive.clone();

        let block = block_parser(ctx.clone(), statement.clone());
        let expression = expression_parser(ctx.clone(), block.clone());
        let body = statement_body_parser(ctx.clone(), expression.clone(), block.clone());
        let ctx_for_stmt = ctx.clone();

        body.then(just(Token::Semicolon).map_with(|_, extra| {
            let span: SimpleSpan<usize> = extra.span();
            span
        }))
        .map(move |((ast, body_span), semi_span)| {
            let total_span = merge_span(body_span, semi_span);
            ctx_for_stmt.wrap_statement(ast, total_span)
        })
        .boxed()
    })
    .boxed();

    statement
        .clone()
        .repeated()
        .collect::<Vec<AST>>()
        .then_ignore(end())
        .boxed()
}

pub(super) fn block_parser<'src>(
    ctx: ParserContext,
    statement: RecursiveParser<'src, AST>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::OpenBrace)
        .map_with(|_, extra| extra.span())
        .then(statement.clone().repeated().collect::<Vec<AST>>())
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(move |((open_span, statements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            (AST::Block(statements, info), span)
        })
        .boxed()
}

pub(super) fn expression_parser<'src>(
    ctx: ParserContext,
    block: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    recursive(|expression: RecursiveParser<'src, SpannedAst>| {
        let ctx = ctx.clone();
        let block = block.clone();
        let expr = expression.clone().boxed();
        let oracle = oracle_expr_parser(ctx.clone(), expr.clone(), block.clone());
        let logical = or_expr_parser(ctx, expr);
        oracle.or(logical).boxed()
    })
    .boxed()
}

pub(super) fn statement_body_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
    block: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    choice((
        artifact_def_parser(ctx.clone()),
        forge_parser(ctx.clone(), expression.clone()),
        engrave_parser(ctx.clone(), block.clone()),
        reveal_parser(ctx.clone(), expression.clone()),
        orbit_parser(ctx.clone(), expression.clone(), block.clone()),
        orbit_flow_parser(ctx.clone()),
        index_assignment_parser(ctx.clone(), expression.clone()),
        field_assignment_parser(ctx.clone(), expression.clone()),
        assignment_parser(ctx.clone(), expression.clone()),
        expression,
    ))
    .boxed()
}

mod expressions;
mod patterns;
mod statements;

use expressions::*;
use patterns::*;
use statements::*;
