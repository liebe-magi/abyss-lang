use std::sync::Arc;

use chumsky::prelude::*;

use crate::ast::{AssignmentOp, ConditionalAssignment, LineInfo, Type, AST};

use super::helpers::LineMap;
use super::tokens::Token;
use super::SimpleSpan;

type ParserError = Simple<Token, SimpleSpan<usize>>;
type Boxed<'a, T> = BoxedParser<'a, Token, T, ParserError>;
type SpannedAst = (AST, SimpleSpan<usize>);

#[derive(Clone)]
struct ParserContext {
    map: Arc<LineMap>,
}

impl ParserContext {
    fn info(&self, span: SimpleSpan<usize>) -> Option<LineInfo> {
        self.map.line_info(span)
    }

    fn wrap_statement(&self, ast: AST, span: SimpleSpan<usize>) -> AST {
        AST::Statement(Box::new(ast), self.info(span))
    }
}

fn merge_span(a: SimpleSpan<usize>, b: SimpleSpan<usize>) -> SimpleSpan<usize> {
    SimpleSpan::new(a.start().min(b.start()), a.end().max(b.end()))
}

pub fn build_parser(
    map: Arc<LineMap>,
) -> impl Parser<Token, Vec<AST>, Error = ParserError> + Clone {
    let ctx = ParserContext { map };

    let ctx_for_recursive = ctx.clone();

    let statement = recursive::<_, _, _, _, ParserError>(move |statement| {
        let ctx = ctx_for_recursive.clone();

        let block = block_parser(ctx.clone(), statement.clone());
        let expression = expression_parser(ctx.clone(), block.clone());
        let body = statement_body_parser(ctx.clone(), expression.clone(), block.clone());
        let ctx_for_stmt = ctx.clone();

        body.then(just(Token::Semicolon).map_with_span(|_, span: SimpleSpan<usize>| span))
            .map(move |((ast, body_span), semi_span)| {
                let total_span = merge_span(body_span, semi_span);
                ctx_for_stmt.wrap_statement(ast, total_span)
            })
            .boxed()
    });

    statement.clone().repeated().then_ignore(end()).boxed()
}

fn block_parser<'a>(
    ctx: ParserContext,
    statement: Recursive<'a, Token, AST, ParserError>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::OpenBrace)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(statement.clone().repeated())
        .then(just(Token::CloseBrace).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(move |((open_span, statements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            (AST::Block(statements, info), span)
        })
        .boxed()
}

fn expression_parser<'a>(
    ctx: ParserContext,
    block: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    recursive::<_, _, _, _, ParserError>(move |expression| {
        let ctx = ctx.clone();
        let block = block.clone();
        let expr = expression.clone().boxed();
        let oracle = oracle_expr_parser(ctx.clone(), expr.clone(), block.clone());
        let logical = or_expr_parser(ctx, expr);
        oracle.or(logical).boxed()
    })
    .boxed()
}

fn statement_body_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
    block: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    choice((
        forge_parser(ctx.clone(), expression.clone()),
        engrave_parser(ctx.clone(), block.clone()),
        unveil_parser(ctx.clone(), expression.clone()),
        reveal_parser(ctx.clone(), expression.clone()),
        orbit_parser(ctx.clone(), expression.clone(), block.clone()),
        orbit_flow_parser(ctx.clone()),
        assignment_parser(ctx.clone(), expression.clone()),
        expression,
    ))
    .boxed()
}

fn forge_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let morph_flag = just(Token::Morph)
        .to(true)
        .or_not()
        .map(|flag| flag.unwrap_or(false));

    just(Token::Forge)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(morph_flag)
        .then(
            select! { Token::Identifier(name) => name }
                .map_with_span(|name, span: SimpleSpan<usize>| (name, span)),
        )
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .then_ignore(just(Token::Assign))
        .then(expression)
        .map(
            move |(
                (((forge_span, is_morph), (name, _name_span)), (ty, _ty_span)),
                (value_ast, value_span),
            )| {
                let span = SimpleSpan::new(forge_span.start(), value_span.end());
                let info = ctx_for_map.info(span);
                (
                    AST::VarAssign {
                        name,
                        value: Box::new(value_ast),
                        var_type: ty,
                        is_morph,
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn engrave_parser<'a>(ctx: ParserContext, block: Boxed<'a, SpannedAst>) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();

    let params = engrave_param_parser(ctx.clone())
        .separated_by(just(Token::Comma))
        .or_not();

    just(Token::Engrave)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(select! { Token::Identifier(name) => name })
        .then_ignore(just(Token::OpenParen))
        .then(params)
        .then_ignore(just(Token::CloseParen))
        .then(just(Token::Arrow).ignore_then(type_parser()).or_not())
        .then(block)
        .map(
            move |((((engrave_span, name), params_opt), ret_opt), (body_ast, body_span))| {
                let span = SimpleSpan::new(engrave_span.start(), body_span.end());
                let info = ctx_for_map.info(span);
                let params = params_opt.unwrap_or_default();
                let return_type = ret_opt.map(|(ty, _)| ty).unwrap_or(Type::Abyss);
                (
                    AST::Engrave {
                        name,
                        params,
                        return_type,
                        body: Box::new(body_ast),
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn unveil_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Unveil)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then_ignore(just(Token::OpenParen))
        .then(expression.separated_by(just(Token::Comma)).at_least(1))
        .then(just(Token::CloseParen).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(move |((unveil_span, args), close_span)| {
            let span = SimpleSpan::new(unveil_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            let items = args.into_iter().map(|(ast, _)| ast).collect();
            (AST::Unveil(items, info.clone()), span)
        })
        .boxed()
}

fn reveal_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Reveal)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(expression.or_not())
        .map(move |(reveal_span, maybe_expr)| {
            let span = maybe_expr
                .as_ref()
                .map(|(_, expr_span)| SimpleSpan::new(reveal_span.start(), expr_span.end()))
                .unwrap_or(reveal_span);
            let info = ctx_for_map.info(span);
            let value = maybe_expr
                .map(|(ast, _)| Box::new(ast))
                .unwrap_or_else(|| Box::new(AST::Abyss(info.clone())));
            (AST::Reveal(value, info.clone()), span)
        })
        .boxed()
}

fn orbit_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
    block: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let params = orbit_param_parser(ctx.clone(), expression.clone())
        .separated_by(just(Token::Comma))
        .at_least(1)
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
        .or_not();

    just(Token::Orbit)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(params)
        .then(block)
        .map(move |((orbit_span, params_opt), (body_ast, body_span))| {
            let span = SimpleSpan::new(orbit_span.start(), body_span.end());
            let info = ctx_for_map.info(span);
            (
                AST::Orbit {
                    params: params_opt.unwrap_or_default(),
                    body: Box::new(body_ast),
                    line_info: info.clone(),
                },
                span,
            )
        })
        .boxed()
}

fn orbit_flow_parser<'a>(ctx: ParserContext) -> Boxed<'a, SpannedAst> {
    let ctx_resume = ctx.clone();
    let ident = select! { Token::Identifier(name) => name }
        .map_with_span(|name, span: SimpleSpan<usize>| (name, span));

    let resume = just(Token::Resume)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(ident.or_not())
        .map(move |(resume_span, maybe_ident)| {
            let span = maybe_ident
                .as_ref()
                .map(|(_, id_span)| SimpleSpan::new(resume_span.start(), id_span.end()))
                .unwrap_or(resume_span);
            let info = ctx_resume.info(span);
            (
                AST::Resume(maybe_ident.map(|(name, _)| name), info.clone()),
                span,
            )
        });

    let ctx_eject = ctx;
    let eject = just(Token::Eject)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(ident.or_not())
        .map(move |(eject_span, maybe_ident)| {
            let span = maybe_ident
                .as_ref()
                .map(|(_, id_span)| SimpleSpan::new(eject_span.start(), id_span.end()))
                .unwrap_or(eject_span);
            let info = ctx_eject.info(span);
            (
                AST::Eject(maybe_ident.map(|(name, _)| name), info.clone()),
                span,
            )
        });

    resume.or(eject).boxed()
}

fn assignment_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let ident = select! { Token::Identifier(name) => name }
        .map_with_span(|name, span: SimpleSpan<usize>| (name, span));

    let op = choice((
        just(Token::Assign),
        just(Token::AddAssign),
        just(Token::SubAssign),
        just(Token::MulAssign),
        just(Token::DivAssign),
        just(Token::ModAssign),
        just(Token::PowArcanaAssign),
        just(Token::PowAetherAssign),
    ))
    .map_with_span(|token, span: SimpleSpan<usize>| (token, span));

    ident
        .then(op)
        .then(expression)
        .map(
            move |(((name, name_span), (token, _op_span)), (value_ast, value_span))| {
                let span = SimpleSpan::new(name_span.start(), value_span.end());
                let info = ctx_for_map.info(span);
                (
                    AST::Assignment {
                        name,
                        value: Box::new(value_ast),
                        op: assignment_op_from_token(token),
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn assignment_op_from_token(token: Token) -> AssignmentOp {
    match token {
        Token::Assign => AssignmentOp::Assign,
        Token::AddAssign => AssignmentOp::AddAssign,
        Token::SubAssign => AssignmentOp::SubAssign,
        Token::MulAssign => AssignmentOp::MulAssign,
        Token::DivAssign => AssignmentOp::DivAssign,
        Token::ModAssign => AssignmentOp::ModAssign,
        Token::PowArcanaAssign => AssignmentOp::PowArcanaAssign,
        Token::PowAetherAssign => AssignmentOp::PowAetherAssign,
        other => unreachable!("Unhandled assignment operator: {other:?}"),
    }
}

fn oracle_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
    block: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_cond = ctx.clone();

    let assignments = conditional_assignment_parser(ctx.clone(), expression.clone())
        .separated_by(just(Token::Comma))
        .at_least(1)
        .map(|pairs| {
            pairs
                .into_iter()
                .map(|(name, ast)| ConditionalAssignment {
                    variable: name,
                    expression: Box::new(ast),
                    line_info: None,
                })
                .collect::<Vec<_>>()
        });

    let matches = expression
        .clone()
        .separated_by(just(Token::Comma))
        .at_least(1)
        .map(|exprs| {
            exprs
                .into_iter()
                .enumerate()
                .map(|(idx, (ast, _))| ConditionalAssignment {
                    variable: format!("__match_{}", idx),
                    expression: Box::new(ast),
                    line_info: None,
                })
                .collect::<Vec<_>>()
        });

    let conditional = just(Token::OpenParen)
        .ignore_then(
            assignments
                .map(|conds| (false, conds))
                .or(matches.map(|conds| (true, conds))),
        )
        .then_ignore(just(Token::CloseParen))
        .or_not();

    let branch = oracle_branch_parser(ctx.clone(), expression.clone(), block.clone());

    just(Token::Oracle)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(conditional)
        .then_ignore(just(Token::OpenBrace))
        .then(branch.repeated())
        .then(just(Token::CloseBrace).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(
            move |(((oracle_span, conditional_opt), branches), close_span)| {
                let span = SimpleSpan::new(oracle_span.start(), close_span.end());
                let info = ctx_for_cond.info(span);

                let (is_match, mut conditionals) = conditional_opt.unwrap_or((false, Vec::new()));
                for cond in &mut conditionals {
                    cond.line_info = info.clone();
                }

                let mut branch_asts = Vec::with_capacity(branches.len());
                for (branch_ast, _branch_span) in branches {
                    branch_asts.push(branch_ast);
                }

                (
                    AST::Oracle {
                        is_match,
                        conditionals,
                        branches: branch_asts,
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn conditional_assignment_parser<'a>(
    _ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, (String, AST)> {
    select! { Token::Identifier(name) => name }
        .then_ignore(just(Token::Assign))
        .then(expression)
        .map(|(name, (ast, _))| (name, ast))
        .boxed()
}

fn oracle_branch_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
    block: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_stmt = ctx.clone();

    let single_statement = statement_body_parser(ctx.clone(), expression.clone(), block.clone())
        .then(just(Token::Semicolon).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(move |((ast, body_span), semi_span)| {
            let span = merge_span(body_span, semi_span);
            let stmt = ctx_for_stmt.wrap_statement(ast, span);
            (stmt, span)
        });

    let body = block.clone().or(single_statement);

    pattern_parser(ctx.clone(), expression.clone())
        .then_ignore(just(Token::FatArrow))
        .then(body)
        .map(move |((pattern, pattern_span), (body_ast, body_span))| {
            let span = merge_span(pattern_span, body_span);
            let info = ctx.info(span);
            (
                AST::OracleBranch {
                    pattern,
                    body: Box::new(body_ast),
                    line_info: info.clone(),
                },
                span,
            )
        })
        .boxed()
}

fn pattern_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, (Vec<AST>, SimpleSpan<usize>)> {
    let wildcard = select! { Token::Identifier(name) if name == "_" => () }
        .map_with_span(|_, span: SimpleSpan<usize>| (Vec::new(), span));

    let list = just(Token::OpenParen)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then(
            pattern_element_parser(ctx.clone(), expression)
                .separated_by(just(Token::Comma))
                .at_least(1),
        )
        .then(just(Token::CloseParen).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(|((open_span, elements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            (elements, span)
        });

    wildcard.or(list).boxed()
}

fn pattern_element_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, AST> {
    let ctx_for_wild = ctx.clone();
    let dont_care = select! { Token::Identifier(name) if name == "_" => () }.map_with_span(
        move |_, span: SimpleSpan<usize>| AST::OracleDontCareItem(ctx_for_wild.info(span)),
    );

    let expr = expression.map(|(ast, _)| ast);

    dont_care.or(expr).boxed()
}

fn orbit_param_parser<'a>(ctx: ParserContext, expression: Boxed<'a, SpannedAst>) -> Boxed<'a, AST> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with_span(|name, span| (name, span))
        .then_ignore(just(Token::Assign))
        .then(range_expr_parser(ctx.clone(), expression))
        .map(
            move |((name, name_span), (start_ast, end_ast, op, range_span))| {
                let span = merge_span(name_span, range_span);
                let info = ctx_for_map.info(span);
                AST::OrbitParam {
                    name,
                    start: Box::new(start_ast),
                    end: Box::new(end_ast),
                    op,
                    line_info: info,
                }
            },
        )
        .boxed()
}

fn range_expr_parser<'a>(
    _ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, (AST, AST, String, SimpleSpan<usize>)> {
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

fn or_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let and_expr = and_expr_parser(ctx, expression);

    and_expr
        .clone()
        .then(
            just(Token::DoublePipe)
                .ignore_then(and_expr.clone())
                .repeated(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, right| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                (
                    AST::LogicalOr(Box::new(left.0), Box::new(right.0), info.clone()),
                    span,
                )
            })
        })
        .boxed()
}

fn and_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let not_expr = not_expr_parser(ctx, expression);

    not_expr
        .clone()
        .then(
            just(Token::DoubleAmpersand)
                .ignore_then(not_expr.clone())
                .repeated(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, right| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                (
                    AST::LogicalAnd(Box::new(left.0), Box::new(right.0), info.clone()),
                    span,
                )
            })
        })
        .boxed()
}

fn not_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Bang)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .repeated()
        .then(comp_expr_parser(ctx, expression))
        .map(move |(nots, mut current)| {
            for span in nots.into_iter().rev() {
                let total_span = SimpleSpan::new(span.start(), current.1.end());
                let info = ctx_for_map.info(total_span);
                current = (
                    AST::LogicalNot(Box::new(current.0), info.clone()),
                    total_span,
                );
            }
            current
        })
        .boxed()
}

fn comp_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
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
                    Token::Equal => AST::Equal(Box::new(left.0), Box::new(right.0), info.clone()),
                    Token::NotEqual => {
                        AST::NotEqual(Box::new(left.0), Box::new(right.0), info.clone())
                    }
                    Token::LessThan => {
                        AST::LessThan(Box::new(left.0), Box::new(right.0), info.clone())
                    }
                    Token::LessThanOrEqual => {
                        AST::LessThanOrEqual(Box::new(left.0), Box::new(right.0), info.clone())
                    }
                    Token::GreaterThan => {
                        AST::GreaterThan(Box::new(left.0), Box::new(right.0), info.clone())
                    }
                    Token::GreaterThanOrEqual => {
                        AST::GreaterThanOrEqual(Box::new(left.0), Box::new(right.0), info.clone())
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

fn add_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let mul_expr = mul_expr_parser(ctx, expression);

    mul_expr
        .clone()
        .then(
            choice((just(Token::Plus), just(Token::Minus)))
                .then(mul_expr.clone())
                .repeated(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, (op_token, right)| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::Plus => AST::Add(Box::new(left.0), Box::new(right.0), info.clone()),
                    Token::Minus => AST::Sub(Box::new(left.0), Box::new(right.0), info.clone()),
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

fn mul_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let pow_expr = pow_expr_parser(ctx, expression);

    pow_expr
        .clone()
        .then(
            choice((just(Token::Star), just(Token::Slash), just(Token::Percent)))
                .then(pow_expr.clone())
                .repeated(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, (op_token, right)| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::Star => AST::Mul(Box::new(left.0), Box::new(right.0), info.clone()),
                    Token::Slash => AST::Div(Box::new(left.0), Box::new(right.0), info.clone()),
                    Token::Percent => AST::Mod(Box::new(left.0), Box::new(right.0), info.clone()),
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

fn pow_expr_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let factor = factor_parser(ctx, expression);

    factor
        .clone()
        .then(
            choice((just(Token::DoubleStar), just(Token::Caret)))
                .then(factor.clone())
                .repeated(),
        )
        .map(move |(first, rest)| {
            rest.into_iter().fold(first, |left, (op_token, right)| {
                let span = merge_span(left.1, right.1);
                let info = ctx_for_map.info(span);
                let ast = match op_token {
                    Token::DoubleStar => {
                        AST::PowAether(Box::new(left.0), Box::new(right.0), info.clone())
                    }
                    Token::Caret => {
                        AST::PowArcana(Box::new(left.0), Box::new(right.0), info.clone())
                    }
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

fn factor_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    choice((
        trans_parser(ctx.clone(), expression.clone()),
        summon_parser(ctx.clone()),
        literal_parser(ctx.clone()),
        func_call_parser(ctx.clone(), expression.clone()),
        identifier_node(ctx.clone()),
        expression.delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
    ))
    .boxed()
}

fn trans_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Trans)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then_ignore(just(Token::OpenParen))
        .then(expression.clone())
        .then_ignore(just(Token::As))
        .then(type_parser())
        .then(just(Token::CloseParen).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(
            move |(((trans_span, expr_node), (target_type, _)), close_span)| {
                let span = SimpleSpan::new(trans_span.start(), close_span.end());
                let info = ctx_for_map.info(span);
                (
                    AST::Trans(Box::new(expr_node.0), target_type, info.clone()),
                    span,
                )
            },
        )
        .boxed()
}

fn summon_parser<'a>(ctx: ParserContext) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Summon)
        .map_with_span(|_, span: SimpleSpan<usize>| span)
        .then_ignore(just(Token::OpenParen))
        .then(select! { Token::Rune(value) => value }.map_with_span(|value, span| (value, span)))
        .then_ignore(just(Token::Comma))
        .then(type_parser())
        .then(just(Token::CloseParen).map_with_span(|_, span: SimpleSpan<usize>| span))
        .map(move |(((summon_span, (prompt, _)), (ty, _)), close_span)| {
            let span = SimpleSpan::new(summon_span.start(), close_span.end());
            let info = ctx_for_map.info(span);
            (AST::Summon(prompt, ty, info.clone()), span)
        })
        .boxed()
}

fn func_call_parser<'a>(
    ctx: ParserContext,
    expression: Boxed<'a, SpannedAst>,
) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with_span(|name, span| (name, span))
        .then(just(Token::OpenParen).map_with_span(|_, span: SimpleSpan<usize>| span))
        .then(expression.separated_by(just(Token::Comma)).or_not())
        .then(just(Token::CloseParen).map_with_span(|_, span: SimpleSpan<usize>| span))
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
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn literal_parser<'a>(ctx: ParserContext) -> Boxed<'a, SpannedAst> {
    let ctx_omen = ctx.clone();
    let omen = select! { Token::OmenLiteral(value) => value }.map_with_span(move |value, span| {
        let info = ctx_omen.info(span);
        (AST::Omen(value, info), span)
    });

    let ctx_arcana = ctx.clone();
    let arcana = select! { Token::Arcana(value) => value }.map_with_span(move |value, span| {
        let info = ctx_arcana.info(span);
        (AST::Arcana(value, info), span)
    });

    let ctx_aether = ctx.clone();
    let aether = select! { Token::Aether(value) => value }.map_with_span(move |value, span| {
        let info = ctx_aether.info(span);
        (AST::Aether(value.into_inner(), info), span)
    });

    let ctx_rune = ctx;
    let rune = select! { Token::Rune(value) => value }.map_with_span(move |value, span| {
        let info = ctx_rune.info(span);
        (AST::Rune(value, info), span)
    });

    choice((omen, arcana, aether, rune)).boxed()
}

fn identifier_node<'a>(ctx: ParserContext) -> Boxed<'a, SpannedAst> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with_span(move |name, span| {
            let info = ctx_for_map.info(span);
            (AST::Var(name, info), span)
        })
        .boxed()
}

fn engrave_param_parser<'a>(ctx: ParserContext) -> Boxed<'a, AST> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with_span(|name, span| (name, span))
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(move |((name, name_span), (ty, ty_span))| {
            let span = merge_span(name_span, ty_span);
            let info = ctx_for_map.info(span);
            AST::EngraveParam {
                name,
                param_type: ty,
                line_info: info,
            }
        })
        .boxed()
}

fn type_parser<'a>() -> Boxed<'a, (Type, SimpleSpan<usize>)> {
    select! { Token::Type(ty) => ty }
        .map_with_span(|ty, span| (ty, span))
        .boxed()
}
