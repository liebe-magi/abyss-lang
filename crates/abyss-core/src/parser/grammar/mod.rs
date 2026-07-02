use chumsky::{
    error::Rich,
    extra,
    input::IterInput,
    prelude::*,
    recursive::{self, Direct},
};

use crate::ast::{Expr, Span, Stmt};

use super::SimpleSpan;
use super::tokens::{SpannedToken, Token};

pub(super) type ParserInput<'src> = IterInput<std::vec::IntoIter<SpannedToken>, SimpleSpan<usize>>;
pub(super) type ParserError<'src> = Rich<'src, Token, SimpleSpan<usize>>;
pub(super) type ParserExtra<'src> = extra::Full<ParserError<'src>, (), ()>;
pub(super) type BoxedParser<'src, T> =
    chumsky::Boxed<'src, 'src, ParserInput<'src>, T, ParserExtra<'src>>;
pub(super) type RecursiveParser<'src, T> =
    recursive::Recursive<Direct<'src, 'src, ParserInput<'src>, T, ParserExtra<'src>>>;
pub(super) type SpannedExpr = (Expr, SimpleSpan<usize>);
pub(super) type SpannedStmt = (Stmt, SimpleSpan<usize>);
pub(super) type IndexedTarget = (SpannedExpr, SpannedExpr);

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
}

pub(super) fn merge_span(a: SimpleSpan<usize>, b: SimpleSpan<usize>) -> SimpleSpan<usize> {
    SimpleSpan::new(a.start().min(b.start()), a.end().max(b.end()))
}

pub fn build_parser<'src>() -> BoxedParser<'src, Vec<Stmt>> {
    let ctx = ParserContext {};

    let ctx_for_recursive = ctx.clone();

    let statement = recursive(|statement| {
        let ctx = ctx_for_recursive.clone();

        let block = block_parser(ctx.clone(), statement.clone());
        let expression = expression_parser(ctx.clone(), block.clone());
        let body = statement_body_parser(ctx.clone(), expression.clone(), block.clone());

        body.then_ignore(just(Token::Semicolon))
            .map(|(stmt, _)| stmt)
            .boxed()
    })
    .boxed();

    statement
        .clone()
        .repeated()
        .collect::<Vec<Stmt>>()
        .then_ignore(end())
        .boxed()
}

pub(super) fn block_parser<'src>(
    ctx: ParserContext,
    statement: RecursiveParser<'src, Stmt>,
) -> BoxedParser<'src, SpannedStmt> {
    let ctx_for_map = ctx.clone();
    just(Token::OpenBrace)
        .map_with(|_, extra| extra.span())
        .then(statement.clone().repeated().collect::<Vec<Stmt>>())
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(move |((open_span, statements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            (Stmt::Block(statements, info), span)
        })
        .boxed()
}

pub(super) fn expression_parser<'src>(
    ctx: ParserContext,
    block: BoxedParser<'src, SpannedStmt>,
) -> BoxedParser<'src, SpannedExpr> {
    recursive(|expression: RecursiveParser<'src, SpannedExpr>| {
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
    expression: BoxedParser<'src, SpannedExpr>,
    block: BoxedParser<'src, SpannedStmt>,
) -> BoxedParser<'src, SpannedStmt> {
    let ctx_for_expr = ctx.clone();
    let expression_stmt = expression
        .clone()
        .map(move |(expr, span)| (Stmt::Expr(expr, ctx_for_expr.info(span)), span))
        .boxed();
    choice((
        artifact_def_parser(ctx.clone()),
        forge_parser(ctx.clone(), expression.clone()),
        engrave_parser(ctx.clone(), block.clone()),
        reveal_parser(ctx.clone(), expression.clone()),
        orbit_parser(ctx.clone(), expression.clone(), block.clone()),
        orbit_flow_parser(ctx.clone()),
        index_assignment_parser(ctx.clone(), expression.clone()),
        field_assignment_parser(ctx.clone(), expression.clone()),
        assignment_parser(ctx.clone(), expression),
        expression_stmt,
    ))
    .boxed()
}

mod expressions;
mod patterns;
mod statements;

use expressions::*;
use patterns::*;
use statements::*;
