use std::collections::HashMap;
use std::sync::Arc;

use chumsky::{
    error::Rich,
    extra,
    input::IterInput,
    prelude::*,
    recursive::{self, Direct},
};

use crate::ast::{
    AST, ArtifactField, ArtifactMethodTarget, AssignmentOp, ConditionalAssignment, LineInfo, Type,
};

use super::SimpleSpan;
use super::helpers::LineMap;
use super::tokens::{SpannedToken, Token};

type ParserInput<'src> = IterInput<std::vec::IntoIter<SpannedToken>, SimpleSpan<usize>>;
type ParserError<'src> = Rich<'src, Token, SimpleSpan<usize>>;
type ParserExtra<'src> = extra::Full<ParserError<'src>, (), ()>;
type BoxedParser<'src, T> = chumsky::Boxed<'src, 'src, ParserInput<'src>, T, ParserExtra<'src>>;
type RecursiveParser<'src, T> =
    recursive::Recursive<Direct<'src, 'src, ParserInput<'src>, T, ParserExtra<'src>>>;
type SpannedAst = (AST, SimpleSpan<usize>);
type IndexedTarget = (SpannedAst, SpannedAst);

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

pub fn build_parser<'src>(map: Arc<LineMap>) -> BoxedParser<'src, Vec<AST>> {
    let ctx = ParserContext { map };

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

fn block_parser<'src>(
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

fn expression_parser<'src>(
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

fn statement_body_parser<'src>(
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

fn artifact_def_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let ident =
        select! { Token::Identifier(name) => name }.map_with(|name, extra| (name, extra.span()));

    just(Token::Artifact)
        .map_with(|_, extra| extra.span())
        .then(ident)
        .then(artifact_fields_parser(ctx.clone()))
        .map(move |((artifact_span, (name, _)), (fields, body_span))| {
            let span = SimpleSpan::new(artifact_span.start(), body_span.end());
            let info = ctx_for_map.info(span);
            (
                AST::ArtifactDef {
                    name,
                    fields,
                    line_info: info.clone(),
                },
                span,
            )
        })
        .boxed()
}

fn artifact_fields_parser<'src>(
    ctx: ParserContext,
) -> BoxedParser<'src, (Vec<ArtifactField>, SimpleSpan<usize>)> {
    let field = select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .then_ignore(just(Token::Semicolon));

    just(Token::OpenBrace)
        .map_with(|_, extra| extra.span())
        .then(field.repeated().collect::<Vec<_>>())
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
        .try_map(move |((open_span, raw_fields), close_span), _extra| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            let mut seen: HashMap<String, SimpleSpan<usize>> = HashMap::new();
            let mut fields = Vec::with_capacity(raw_fields.len());
            for ((name, name_span), (field_type, type_span)) in raw_fields {
                if let Some(previous_span) = seen.insert(name.clone(), name_span) {
                    let dup_span = SimpleSpan::new(previous_span.start(), name_span.end());
                    return Err(Rich::custom(
                        dup_span,
                        format!("Duplicate field `{name}` in artifact definition"),
                    ));
                }
                let field_span = SimpleSpan::new(name_span.start(), type_span.end());
                let info = ctx.info(field_span);
                fields.push(ArtifactField {
                    name,
                    field_type,
                    line_info: info,
                });
            }
            Ok((fields, span))
        })
        .boxed()
}

fn forge_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let morph_flag = just(Token::Morph)
        .to(true)
        .or_not()
        .map(|flag| flag.unwrap_or(false));

    just(Token::Forge)
        .map_with(|_, extra| extra.span())
        .then(morph_flag)
        .then(
            select! { Token::Identifier(name) => name }
                .map_with(|name, extra| (name, extra.span())),
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

/// Internal representation for parsed engrave parameters before AST node creation.
/// Used during engrave parsing to distinguish receiver parameters from regular parameters.
#[derive(Clone)]
enum RawEngraveParam {
    Receiver {
        is_morph: bool,
        span: SimpleSpan<usize>,
    },
    Param(AST),
}

/// Internal representation for engrave target classification.
/// Used during engrave parsing to determine if the definition is a standalone function
/// or an artifact method, enabling proper parameter validation and AST construction.
enum EngraveTarget {
    Function {
        name: String,
    },
    Method {
        artifact: String,
        method: String,
        span: SimpleSpan<usize>,
    },
}

fn method_receiver_parser<'src>() -> BoxedParser<'src, RawEngraveParam> {
    just(Token::Morph)
        .map_with(|_, extra| extra.span())
        .or_not()
        .then(core_keyword_span())
        .map(
            |(maybe_morph, core_span): (Option<SimpleSpan<usize>>, SimpleSpan<usize>)| {
                let span = if let Some(morph_span) = maybe_morph {
                    SimpleSpan::new(morph_span.start(), core_span.end())
                } else {
                    core_span
                };
                RawEngraveParam::Receiver {
                    is_morph: maybe_morph.is_some(),
                    span,
                }
            },
        )
        .boxed()
}

fn core_keyword_span<'src>() -> BoxedParser<'src, SimpleSpan<usize>> {
    just(Token::Core).map_with(|_, extra| extra.span()).boxed()
}

fn engrave_parser<'src>(
    ctx: ParserContext,
    block: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();

    let params = choice((
        method_receiver_parser(),
        engrave_param_parser(ctx.clone()).map(RawEngraveParam::Param),
    ))
    .separated_by(just(Token::Comma))
    .collect::<Vec<_>>()
    .or_not();

    let ident_with_span: BoxedParser<'src, (String, SimpleSpan<usize>)> =
        select! { Token::Identifier(name) => name }
            .map_with(|name, extra| (name, extra.span()))
            .boxed();

    let target = ident_with_span
        .clone()
        .then(
            just(Token::DoubleColon)
                .ignore_then(ident_with_span.clone())
                .or_not(),
        )
        .map(|((name, name_span), maybe_method)| {
            if let Some((method, method_span)) = maybe_method {
                let span = SimpleSpan::new(name_span.start(), method_span.end());
                EngraveTarget::Method {
                    artifact: name,
                    method,
                    span,
                }
            } else {
                EngraveTarget::Function { name }
            }
        });

    just(Token::Engrave)
        .map_with(|_, extra| extra.span())
        .then(target)
        .then_ignore(just(Token::OpenParen))
        .then(params)
        .then_ignore(just(Token::CloseParen))
        .then(just(Token::Arrow).ignore_then(type_parser()).or_not())
        .then(block)
        .try_map(
            move |((((engrave_span, target), params_opt), ret_opt), (body_ast, body_span)), _extra| {
                let span = SimpleSpan::new(engrave_span.start(), body_span.end());
                let info = ctx_for_map.info(span);
                let raw_params = params_opt.unwrap_or_default();
                let return_type = ret_opt.map(|(ty, _)| ty).unwrap_or(Type::Abyss);

                match target {
                    EngraveTarget::Function { name } => {
                        let mut params = Vec::with_capacity(raw_params.len());
                        for entry in raw_params {
                            match entry {
                                RawEngraveParam::Param(node) => params.push(node),
                                RawEngraveParam::Receiver { span: recv_span, .. } => {
                                    return Err(Rich::custom(
                                        recv_span,
                                        "The `core` parameter is reserved for artifact methods. Use a regular parameter name for standalone functions.",
                                    ));
                                }
                            }
                        }

                        Ok((
                            AST::Engrave {
                                name,
                                params,
                                return_type,
                                body: Box::new(body_ast),
                                method_target: None,
                                line_info: info,
                            },
                            span,
                        ))
                    }
                    EngraveTarget::Method {
                        artifact,
                        method,
                        span: target_span,
                    } => {
                        let mut params = Vec::with_capacity(raw_params.len());
                        let mut target_meta = ArtifactMethodTarget {
                            artifact,
                            requires_morph: false,
                        };
                        let mut receiver_seen = false;

                        for (idx, entry) in raw_params.into_iter().enumerate() {
                            match entry {
                                RawEngraveParam::Receiver {
                                    is_morph,
                                    span: recv_span,
                                } => {
                                    if receiver_seen {
                                        return Err(Rich::custom(
                                            recv_span,
                                            "The `core` receiver can only appear once in the parameter list",
                                        ));
                                    }
                                    if idx != 0 {
                                        return Err(Rich::custom(
                                            recv_span,
                                            "`core` receiver must be the first parameter",
                                        ));
                                    }
                                    receiver_seen = true;
                                    target_meta.requires_morph = is_morph;
                                    let info = ctx_for_map.info(recv_span);
                                    params.push(AST::EngraveParam {
                                        name: "core".to_string(),
                                        param_type: Type::Artifact(target_meta.artifact.clone()),
                                        is_morph,
                                        line_info: info,
                                    });
                                }
                                RawEngraveParam::Param(node) => params.push(node),
                            }
                        }

                        if !receiver_seen {
                            return Err(Rich::custom(
                                target_span,
                                "Artifact methods must declare `core` (or `morph core`) as the first parameter",
                            ));
                        }

                        Ok((
                            AST::Engrave {
                                name: method,
                                params,
                                return_type,
                                body: Box::new(body_ast),
                                method_target: Some(target_meta),
                                line_info: info,
                            },
                            span,
                        ))
                    }
                }
            },
        )
        .boxed()
}

fn reveal_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    just(Token::Reveal)
        .map_with(|_, extra| extra.span())
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

fn orbit_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
    block: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let params = orbit_param_parser(ctx.clone(), expression.clone())
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
        .or_not();

    just(Token::Orbit)
        .map_with(|_, extra| extra.span())
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

fn orbit_flow_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedAst> {
    let ctx_resume = ctx.clone();
    let ident: BoxedParser<'src, (String, SimpleSpan<usize>)> = select! {
        Token::Identifier(name) => name
    }
    .map_with(|name, extra| (name, extra.span()))
    .boxed();

    let resume = just(Token::Resume)
        .map_with(|_, extra| extra.span())
        .then(ident.clone().or_not())
        .map(
            move |(resume_span, maybe_ident): (
                SimpleSpan<usize>,
                Option<(String, SimpleSpan<usize>)>,
            )| {
                let span = maybe_ident
                    .as_ref()
                    .map(|(_, id_span)| SimpleSpan::new(resume_span.start(), id_span.end()))
                    .unwrap_or(resume_span);
                let info = ctx_resume.info(span);
                (
                    AST::Resume(maybe_ident.map(|(name, _)| name), info.clone()),
                    span,
                )
            },
        );

    let ctx_eject = ctx;
    let eject = just(Token::Eject)
        .map_with(|_, extra| extra.span())
        .then(ident.or_not())
        .map(
            move |(eject_span, maybe_ident): (
                SimpleSpan<usize>,
                Option<(String, SimpleSpan<usize>)>,
            )| {
                let span = maybe_ident
                    .as_ref()
                    .map(|(_, id_span)| SimpleSpan::new(eject_span.start(), id_span.end()))
                    .unwrap_or(eject_span);
                let info = ctx_eject.info(span);
                (
                    AST::Eject(maybe_ident.map(|(name, _)| name), info.clone()),
                    span,
                )
            },
        );

    resume.or(eject).boxed()
}

fn assignment_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    let ident =
        select! { Token::Identifier(name) => name }.map_with(|name, extra| (name, extra.span()));

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
    .map_with(|token, extra| (token, extra.span()));

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

fn field_assignment_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    expression
        .clone()
        .then_ignore(just(Token::Assign))
        .then(expression)
        .try_map(
            move |((target_ast, target_span), (value_ast, value_span)), _extra| {
                if let AST::FieldAccess { target, field, .. } = target_ast {
                    let span = SimpleSpan::new(target_span.start(), value_span.end());
                    let info = ctx_for_map.info(span);
                    Ok((
                        AST::FieldAssignment {
                            target,
                            field,
                            value: Box::new(value_ast),
                            line_info: info.clone(),
                        },
                        span,
                    ))
                } else {
                    Err(Rich::custom(
                        target_span,
                        "Field assignment requires an artifact field target".to_string(),
                    ))
                }
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

fn index_assignment_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    indexed_target_parser(ctx, expression.clone())
        .then_ignore(just(Token::Assign))
        .then(expression)
        .map(
            move |(
                ((target_ast, target_span), (index_ast, index_span)),
                (value_ast, value_span),
            )| {
                let lhs_span = SimpleSpan::new(target_span.start(), index_span.end());
                let span = SimpleSpan::new(lhs_span.start(), value_span.end());
                let info = ctx_for_map.info(span);
                (
                    AST::IndexAssignment {
                        target: Box::new(target_ast),
                        index: Box::new(index_ast),
                        value: Box::new(value_ast),
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn indexed_target_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, IndexedTarget> {
    let ctx_for_map = ctx.clone();
    primary_expr_parser(ctx.clone(), expression.clone())
        .then(
            index_suffix_parser(ctx.clone(), expression)
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(move |(base, mut suffixes)| {
            let last = suffixes
                .pop()
                .expect("Parser guaranteed at least one index suffix via .at_least(1)");
            let target = suffixes.into_iter().fold(base, |acc, suffix| {
                create_index_access(ctx_for_map.clone(), acc, suffix)
            });
            (target, last)
        })
        .boxed()
}

fn oracle_expr_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
    block: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_cond = ctx.clone();

    let assignments = conditional_assignment_parser(ctx.clone(), expression.clone())
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>()
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
        .collect::<Vec<_>>()
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
        .map_with(|_, extra| extra.span())
        .then(conditional)
        .then_ignore(just(Token::OpenBrace))
        .then(branch.repeated().collect::<Vec<SpannedAst>>())
        .then(just(Token::CloseBrace).map_with(|_, extra| extra.span()))
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

fn conditional_assignment_parser<'src>(
    _ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, (String, AST)> {
    select! { Token::Identifier(name) => name }
        .then_ignore(just(Token::Assign))
        .then(expression)
        .map(|(name, (ast, _))| (name, ast))
        .boxed()
}

fn oracle_branch_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
    block: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_stmt = ctx.clone();

    let single_statement = statement_body_parser(ctx.clone(), expression.clone(), block.clone())
        .then(just(Token::Semicolon).map_with(|_, extra| extra.span()))
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

fn pattern_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, (Vec<AST>, SimpleSpan<usize>)> {
    let wildcard = select! { Token::Identifier(name) if name == "_" => () }
        .map_with(|_, extra| (Vec::new(), extra.span()));

    let list = just(Token::OpenParen)
        .map_with(|_, extra| extra.span())
        .then(
            pattern_element_parser(ctx.clone(), expression)
                .separated_by(just(Token::Comma))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then(just(Token::CloseParen).map_with(|_, extra| extra.span()))
        .map(|((open_span, elements), close_span)| {
            let span = SimpleSpan::new(open_span.start(), close_span.end());
            (elements, span)
        });

    wildcard.or(list).boxed()
}

fn pattern_element_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, AST> {
    let ctx_for_wild = ctx.clone();
    let dont_care =
        select! { Token::Identifier(name) if name == "_" => () }.map_with(move |_, extra| {
            let span = extra.span();
            AST::OracleDontCareItem(ctx_for_wild.info(span))
        });

    let expr = expression.map(|(ast, _)| ast);

    dont_care.or(expr).boxed()
}

fn orbit_param_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, AST> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
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

fn range_expr_parser<'src>(
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

fn or_expr_parser<'src>(
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
                    AST::LogicalOr(Box::new(left.0), Box::new(right.0), info.clone()),
                    span,
                )
            })
        })
        .boxed()
}

fn and_expr_parser<'src>(
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
                    AST::LogicalAnd(Box::new(left.0), Box::new(right.0), info.clone()),
                    span,
                )
            })
        })
        .boxed()
}

fn not_expr_parser<'src>(
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
                current = (
                    AST::LogicalNot(Box::new(current.0), info.clone()),
                    total_span,
                );
            }
            current
        })
        .boxed()
}

fn comp_expr_parser<'src>(
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

fn add_expr_parser<'src>(
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
                    Token::Plus => AST::Add(Box::new(left.0), Box::new(right.0), info.clone()),
                    Token::Minus => AST::Sub(Box::new(left.0), Box::new(right.0), info.clone()),
                    _ => unreachable!(),
                };
                (ast, span)
            })
        })
        .boxed()
}

fn mul_expr_parser<'src>(
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

fn pow_expr_parser<'src>(
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

fn factor_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedAst>,
) -> BoxedParser<'src, SpannedAst> {
    let primary = primary_expr_parser(ctx.clone(), expression.clone());
    apply_postfix_suffixes(ctx, expression, primary).boxed()
}

fn primary_expr_parser<'src>(
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

enum PostfixSuffix {
    Index((AST, SimpleSpan<usize>)),
    Field((String, SimpleSpan<usize>)),
    Method(MethodSuffix),
}

struct MethodSuffix {
    name: String,
    args: Vec<AST>,
    span: SimpleSpan<usize>,
}

fn apply_postfix_suffixes<'src>(
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

fn postfix_suffix_parser<'src>(
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

fn index_suffix_parser<'src>(
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

fn create_field_access(
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

fn create_method_call(
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

fn create_index_access(
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

fn func_call_parser<'src>(
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
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn list_literal_parser<'src>(
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
                    line_info: info.clone(),
                },
                span,
            )
        })
        .boxed()
}

fn map_literal_parser<'src>(
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
                    line_info: info.clone(),
                },
                span,
            )
        })
        .boxed()
}

fn artifact_literal_parser<'src>(
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
                        line_info: info.clone(),
                    },
                    span,
                )
            },
        )
        .boxed()
}

fn literal_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedAst> {
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

fn identifier_node<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedAst> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name, Token::Core => "core".to_string(), Token::Type(ty) => type_keyword_name(&ty) }
        .map_with(move |name, extra| {
            let span = extra.span();
            let info = ctx_for_map.info(span);
            (AST::Var(name, info), span)
        })
        .boxed()
}

fn engrave_param_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, AST> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(move |((name, name_span), (ty, ty_span))| {
            let span = merge_span(name_span, ty_span);
            let info = ctx_for_map.info(span);
            AST::EngraveParam {
                name,
                param_type: ty,
                is_morph: false,
                line_info: info,
            }
        })
        .boxed()
}

fn type_parser<'src>() -> BoxedParser<'src, (Type, SimpleSpan<usize>)> {
    choice((
        select! { Token::Type(ty) => ty }.map_with(|ty, extra| (ty, extra.span())),
        select! { Token::Identifier(name) => name }
            .map_with(|name, extra| (Type::Artifact(name), extra.span())),
    ))
    .boxed()
}

fn type_keyword_name(ty: &Type) -> String {
    match ty {
        Type::Arcana => "arcana",
        Type::Aether => "aether",
        Type::Rune => "rune",
        Type::Omen => "omen",
        Type::Abyss => "abyss",
        Type::Scroll => "scroll",
        Type::Lexicon => "lexicon",
        Type::Materia => "materia",
        Type::Glyph => "glyph",
        Type::Artifact(name) => name,
    }
    .to_string()
}
