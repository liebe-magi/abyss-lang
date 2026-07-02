//! Oracle parsers: the `oracle` expression itself, its branch arms, and
//! the pattern shapes (scroll / artifact / lexicon destructuring).

use chumsky::prelude::*;

use crate::ast::{ConditionalAssignment, Expr, OracleBranch, Pattern};

use crate::parser::SimpleSpan;
use crate::parser::tokens::Token;

use super::*;

pub(super) fn oracle_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
    block: BoxedParser<'src, SpannedStmt>,
) -> BoxedParser<'src, SpannedExpr> {
    let ctx_for_cond = ctx.clone();

    let match_scrutinee = expression
        .clone()
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|exprs| {
            exprs
                .into_iter()
                .enumerate()
                .map(|(idx, (expr, _))| ConditionalAssignment {
                    variable: format!("__match_{}", idx),
                    expression: Box::new(expr),
                    span: None,
                })
                .collect::<Vec<_>>()
        });

    let conditional = just(Token::OpenParen)
        .ignore_then(match_scrutinee)
        .then_ignore(just(Token::CloseParen))
        .or_not();

    let branch = oracle_branch_parser(ctx.clone(), expression.clone(), block.clone());

    just(Token::Oracle)
        .map_with(|_, extra| extra.span())
        .then(conditional)
        .then_ignore(just(Token::OpenBrace))
        .then(
            branch
                .repeated()
                .collect::<Vec<(OracleBranch, SimpleSpan<usize>)>>(),
        )
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(
            move |(((oracle_span, conditional_opt), branches), close_span)| {
                let span = SimpleSpan::new(oracle_span.start(), close_span.end());
                let info = ctx_for_cond.info(span);

                let (is_match, mut conditionals) = if let Some(conds) = conditional_opt {
                    (true, conds)
                } else {
                    (false, Vec::new())
                };
                for cond in &mut conditionals {
                    cond.span = info;
                }

                let branches = branches.into_iter().map(|(branch, _)| branch).collect();

                (
                    Expr::Oracle {
                        is_match,
                        conditionals,
                        branches,
                        span: info,
                    },
                    span,
                )
            },
        )
        .boxed()
}

pub(super) fn oracle_branch_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
    block: BoxedParser<'src, SpannedStmt>,
) -> BoxedParser<'src, (OracleBranch, SimpleSpan<usize>)> {
    let single_statement = statement_body_parser(ctx.clone(), expression.clone(), block.clone())
        .then(just(Token::Semicolon).map_with(|_, extra| extra.span()))
        .map(move |((stmt, body_span), semi_span)| {
            let span = merge_span(body_span, semi_span);
            (stmt, span)
        });

    let body = block.clone().or(single_statement);

    let guard = just(Token::Ward)
        .ignore_then(expression.clone())
        .map(|(expr, _)| expr)
        .or_not();

    // The pattern element parser is mutually recursive with the scroll- and
    // artifact-pattern parsers (since a scroll element or an artifact field
    // value may itself be a scroll / artifact / wildcard pattern). Build it
    // once via `recursive` and thread it into both the top-level pattern
    // recogniser and the inner sub-parsers so nested patterns like
    // `Player { items: [head, ..rest] }` actually parse into the pattern
    // nodes the evaluator already knows how to match.
    let pattern_element = pattern_element_parser(ctx.clone(), expression.clone());
    let pattern = pattern_parser(ctx.clone(), expression.clone(), pattern_element);

    pattern
        .then(guard)
        .then_ignore(just(Token::FatArrow))
        .then(body)
        .map(
            move |(((pattern, pattern_span), guard_expr), (body_stmt, body_span))| {
                let span = merge_span(pattern_span, body_span);
                let info = ctx.info(span);
                (
                    OracleBranch {
                        pattern,
                        guard: guard_expr,
                        body: body_stmt,
                        span: info,
                    },
                    span,
                )
            },
        )
        .boxed()
}

pub(super) fn pattern_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
    pattern_element: BoxedParser<'src, Pattern>,
) -> BoxedParser<'src, (Vec<Pattern>, SimpleSpan<usize>)> {
    let wildcard = select! { Token::Identifier(name) if name == "_" => () }
        .map_with(|_, extra| (Vec::new(), extra.span()));

    let list = just(Token::OpenParen)
        .map_with(|_, extra| extra.span())
        .then(
            pattern_element
                .clone()
                .separated_by(just(Token::Comma))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then(just(Token::CloseParen).map_with(|_, extra| extra.span()))
        .map(|((open_span, elements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            (elements, span)
        });

    // A scroll pattern `[…]` at the top of an arm targets a single-scrutinee
    // oracle and is wrapped in a one-element pattern vector to match the
    // shape produced by `wildcard` and `list`. The same scroll-pattern parser
    // is also referenced through `pattern_element` so scroll patterns can sit
    // alongside other elements in a multi-scrutinee tuple pattern.
    let scroll = scroll_pattern_parser(ctx.clone(), pattern_element.clone(), expression.clone())
        .map_with(|pattern, extra| {
            let span: SimpleSpan<usize> = SimpleSpan::new(extra.span().start(), extra.span().end());
            (vec![pattern], span)
        });

    // An artifact pattern `TypeName { … }` at the top of an arm — same shape
    // wrapping logic as `scroll`. The recursive `pattern_element` lets each
    // field value itself be a nested scroll / artifact / wildcard pattern.
    let artifact =
        artifact_pattern_parser(ctx.clone(), pattern_element.clone(), expression.clone()).map_with(
            |pattern, extra| {
                let span: SimpleSpan<usize> =
                    SimpleSpan::new(extra.span().start(), extra.span().end());
                (vec![pattern], span)
            },
        );

    // A lexicon pattern `{ "key": value, … }` at the top of an arm — same
    // shape wrapping logic. Distinguished from artifact pattern by the
    // absence of a leading identifier, and from a generic block `{ stmt; }`
    // by the rune-literal-and-colon entries. Each value uses the recursive
    // `pattern_element`, so nested patterns work the same way as inside
    // artifact patterns.
    let lexicon =
        lexicon_pattern_parser(ctx, pattern_element, expression).map_with(|pattern, extra| {
            let span: SimpleSpan<usize> = SimpleSpan::new(extra.span().start(), extra.span().end());
            (vec![pattern], span)
        });

    wildcard
        .or(list)
        .or(scroll)
        .or(artifact)
        .or(lexicon)
        .boxed()
}

pub(super) fn pattern_element_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, Pattern> {
    recursive(|pattern_elem: RecursiveParser<'src, Pattern>| {
        let ctx_for_wild = ctx.clone();
        let dont_care =
            select! { Token::Identifier(name) if name == "_" => () }.map_with(move |_, extra| {
                let span = extra.span();
                Pattern::DontCare(ctx_for_wild.info(span))
            });

        let pattern_elem_boxed = pattern_elem.clone().boxed();
        let scroll =
            scroll_pattern_parser(ctx.clone(), pattern_elem_boxed.clone(), expression.clone());
        // Run before the generic expression branch so `Player { … }` is
        // parsed as a destructuring pattern rather than the artifact literal
        // it would be in expression position.
        let artifact =
            artifact_pattern_parser(ctx.clone(), pattern_elem_boxed.clone(), expression.clone());
        // Lexicon pattern shares the brace token but starts directly with a
        // rune-literal key followed by `:`, which neither an empty block
        // nor an artifact literal (which needs a leading identifier) can
        // start with. Run before the expression branch so `{ "k": v }` in
        // pattern position becomes a destructuring pattern rather than the
        // map literal it would otherwise be.
        let lexicon = lexicon_pattern_parser(ctx.clone(), pattern_elem_boxed, expression.clone());

        let expr = expression.clone().map(|(expr, _)| Pattern::Expr(expr));

        dont_care
            .or(scroll)
            .or(artifact)
            .or(lexicon)
            .or(expr)
            .boxed()
    })
    .boxed()
}

/// Parses an artifact-shape pattern `TypeName { f0, f1: pat, … }` for an
/// oracle match-mode arm. Each field entry is one of:
///
/// - `field_name` — shorthand binding; the field value is bound to a fresh
///   variable named after the field.
/// - `field_name: _` — explicit wildcard; the field is matched but its
///   value is discarded.
/// - `field_name: ident` — explicit binding; the field value is bound to
///   `ident` (which may differ from the field name).
/// - `field_name: <expr>` — literal compare against the field value.
///
/// Fields not listed here are not matched against — the pattern is
/// non-exhaustive by default, so users can pick out only the fields they
/// care about. We must run before the generic expression branch in
/// `pattern_element_parser` so `Player { name }` in pattern position is
/// parsed as a destructuring pattern rather than the artifact literal it
/// would be in expression position.
pub(super) fn artifact_pattern_parser<'src>(
    ctx: ParserContext,
    pattern_element: BoxedParser<'src, Pattern>,
    _expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, Pattern> {
    // Each field entry is `name (: value)?`. The shorthand expands to a
    // `Pattern::Expr(Expr::Var(name))` so the evaluator's existing
    // `Var`-as-binding path applies uniformly with tuple and scroll
    // patterns. The explicit `: value` form accepts any pattern element
    // (wildcard, nested scroll/artifact patterns, bindings, literal
    // expressions), so nested destructuring like
    // `Player { items: [head, ..rest], friend: Enemy { name } }` parses
    // into the pattern nodes the evaluator already knows how to match.
    let ctx_for_field = ctx.clone();
    let field = select! { Token::Identifier(name) => name }
        .map_with(move |name, extra| (name, extra.span()))
        .then(just(Token::Colon).ignore_then(pattern_element).or_not())
        .map(move |((name, name_span), value)| {
            let pattern = value.unwrap_or_else(|| {
                Pattern::Expr(Expr::Var(
                    name.clone(),
                    ctx_for_field.info(SimpleSpan::new(name_span.start(), name_span.end())),
                ))
            });
            (name, pattern)
        });

    let ctx_for_outer = ctx;
    select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then_ignore(just(Token::OpenBrace))
        .then(
            field
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(move |(((type_name, type_span), fields), close_span)| {
            let span = SimpleSpan::new(type_span.start(), close_span.end());
            Pattern::Artifact {
                type_name,
                fields,
                span: ctx_for_outer.info(span),
            }
        })
        .boxed()
}

/// Parses a lexicon-shape pattern `{ "k0": v0, "k1": v1, … }` for an
/// oracle match-mode arm. Each entry's value is a recursive pattern
/// element (binding, wildcard, nested scroll/artifact pattern, literal
/// expression). Keys not listed here are not matched against — the pattern
/// is non-exhaustive by default, mirroring the artifact pattern's
/// "pick what you need" ergonomics.
///
/// Empty `{}` matches any lexicon (a "match by shape" catch-all), reusing
/// the same brace tokens as the lexicon literal it would be in expression
/// position. Disambiguation: the parser tries this before the generic
/// expression branch in `pattern_element_parser`, so the brace form in
/// pattern position is a destructuring pattern rather than the map literal.
pub(super) fn lexicon_pattern_parser<'src>(
    ctx: ParserContext,
    pattern_element: BoxedParser<'src, Pattern>,
    _expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, Pattern> {
    let entry = select! { Token::Rune(key) => key }
        .map_with(|key, extra| (key, extra.span()))
        .then_ignore(just(Token::Colon))
        .then(pattern_element)
        .map(|((key, _), value)| (key, value));

    let ctx_for_outer = ctx;
    just(Token::OpenBrace)
        .map_with(|_, extra| extra.span())
        .then(
            entry
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .map(move |((open_span, entries), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            Pattern::Lexicon {
                entries,
                span: ctx_for_outer.info(span),
            }
        })
        .boxed()
}

/// Parses a scroll-shape pattern `[e0, e1, …, ..rest]` for an oracle
/// match-mode arm. Each inner element is one of:
///
/// - `_` — wildcard, matches and discards a single element.
/// - `..` — anonymous rest, drops the trailing slice (only one allowed,
///   only at the end).
/// - `..ident` — named rest, binds the trailing slice to a fresh sub-scroll.
/// - bare identifier — binding, captures the element at this position.
/// - any other expression — literal compare against the element value.
///
/// We must run before the generic expression branch in `pattern_element_parser`
/// so `[a, b]` in pattern position becomes a destructuring pattern rather
/// than the list literal it would be in expression position.
pub(super) fn scroll_pattern_parser<'src>(
    ctx: ParserContext,
    pattern_element: BoxedParser<'src, Pattern>,
    _expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, Pattern> {
    let ctx_for_rest = ctx.clone();
    let rest = just(Token::RangeExclusive)
        .map_with(|_, extra| extra.span())
        .then(
            select! { Token::Identifier(name) => name }
                .map_with(|name, extra| (name, extra.span()))
                .or_not(),
        )
        .map(move |(open_span, named)| {
            let close_span: SimpleSpan<usize> =
                named.as_ref().map(|(_, span)| *span).unwrap_or(open_span);
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            Pattern::Rest {
                name: named.map(|(name, _)| name),
                span: ctx_for_rest.info(span),
            }
        });

    // Non-rest scroll elements use the recursive `pattern_element`, which
    // already covers wildcards, nested scroll patterns, nested artifact
    // patterns, and arbitrary expressions. `rest` is scroll-specific so it
    // lives here as the alternation's first branch.
    let element = rest.or(pattern_element);

    let ctx_for_outer = ctx;
    just(Token::OpenBracket)
        .map_with(|_, extra| extra.span())
        .then(
            element
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>(),
        )
        .then(just(Token::CloseBracket).map_with(|_, extra| extra.span()))
        .map(move |((open_span, elements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            Pattern::Scroll {
                elements,
                span: ctx_for_outer.info(span),
            }
        })
        .boxed()
}
