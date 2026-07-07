//! Statement-level parsers: `forge`, `engrave`, artifact definitions,
//! `reveal`, `orbit` (with `revolve` / `eject`), and the assignment family.

use std::collections::HashMap;

use chumsky::{error::Rich, prelude::*};

use crate::ast::{
    ArtifactField, ArtifactMethodTarget, AssignmentOp, EngraveParam, Expr, OrbitParam, Stmt, Type,
};

use crate::parser::SimpleSpan;
use crate::parser::tokens::Token;

use super::*;

pub(super) fn artifact_def_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedStmt> {
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
                Stmt::ArtifactDef {
                    name,
                    fields,
                    span: info,
                },
                span,
            )
        })
        .boxed()
}

pub(super) fn artifact_fields_parser<'src>(
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
                    span: info,
                });
            }
            Ok((fields, span))
        })
        .boxed()
}

pub(super) fn forge_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, SpannedStmt> {
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
                    Stmt::VarAssign {
                        name,
                        value: value_ast,
                        var_type: ty,
                        is_morph,
                        span: info,
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
pub(super) enum RawEngraveParam {
    Receiver {
        is_morph: bool,
        span: SimpleSpan<usize>,
    },
    Param(EngraveParam),
}

/// Internal representation for engrave target classification.
/// Used during engrave parsing to determine if the definition is a standalone function
/// or an artifact method, enabling proper parameter validation and AST construction.
pub(super) enum EngraveTarget {
    Function {
        name: String,
    },
    Method {
        artifact: String,
        method: String,
        span: SimpleSpan<usize>,
    },
}

pub(super) fn method_receiver_parser<'src>() -> BoxedParser<'src, RawEngraveParam> {
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

pub(super) fn core_keyword_span<'src>() -> BoxedParser<'src, SimpleSpan<usize>> {
    just(Token::Core).map_with(|_, extra| extra.span()).boxed()
}

pub(super) fn engrave_parser<'src>(
    ctx: ParserContext,
    block: BoxedParser<'src, SpannedStmt>,
) -> BoxedParser<'src, SpannedStmt> {
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
                            Stmt::Engrave {
                                name,
                                params,
                                return_type,
                                body: Box::new(body_ast),
                                method_target: None,
                                span: info,
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
                                    params.push(EngraveParam {
                                        name: "core".to_string(),
                                        param_type: Type::Artifact(target_meta.artifact.clone()),
                                        is_morph,
                                        span: info,
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
                            Stmt::Engrave {
                                name: method,
                                params,
                                return_type,
                                body: Box::new(body_ast),
                                method_target: Some(target_meta),
                                span: info,
                            },
                            span,
                        ))
                    }
                }
            },
        )
        .boxed()
}

pub(super) fn reveal_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, SpannedStmt> {
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
                .map(|(expr, _)| expr)
                .unwrap_or_else(|| Expr::Abyss(info));
            (Stmt::Reveal(value, info), span)
        })
        .boxed()
}

pub(super) fn orbit_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
    block: BoxedParser<'src, SpannedStmt>,
) -> BoxedParser<'src, SpannedStmt> {
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
                Stmt::Orbit {
                    params: params_opt.unwrap_or_default(),
                    body: Box::new(body_ast),
                    span: info,
                },
                span,
            )
        })
        .boxed()
}

pub(super) fn orbit_flow_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, SpannedStmt> {
    let ctx_resume = ctx.clone();
    let ident: BoxedParser<'src, (String, SimpleSpan<usize>)> = select! {
        Token::Identifier(name) => name
    }
    .map_with(|name, extra| (name, extra.span()))
    .boxed();

    let revolve = just(Token::Revolve)
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
                (Stmt::Revolve(maybe_ident.map(|(name, _)| name), info), span)
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
                (Stmt::Eject(maybe_ident.map(|(name, _)| name), info), span)
            },
        );

    revolve.or(eject).boxed()
}

pub(super) fn assignment_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, SpannedStmt> {
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
                    Stmt::Assignment {
                        name,
                        value: value_ast,
                        op: assignment_op_from_token(token),
                        span: info,
                    },
                    span,
                )
            },
        )
        .boxed()
}

pub(super) fn field_assignment_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, SpannedStmt> {
    let ctx_for_map = ctx.clone();
    expression
        .clone()
        .then_ignore(just(Token::Assign))
        .then(expression)
        .try_map(
            move |((target_ast, target_span), (value_ast, value_span)), _extra| {
                if let Expr::FieldAccess { target, field, .. } = target_ast {
                    let span = SimpleSpan::new(target_span.start(), value_span.end());
                    let info = ctx_for_map.info(span);
                    Ok((
                        Stmt::FieldAssignment {
                            target: *target,
                            field,
                            value: value_ast,
                            span: info,
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

pub(super) fn assignment_op_from_token(token: Token) -> AssignmentOp {
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

pub(super) fn index_assignment_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, SpannedStmt> {
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
                    Stmt::IndexAssignment {
                        target: target_ast,
                        index: index_ast,
                        value: value_ast,
                        span: info,
                    },
                    span,
                )
            },
        )
        .boxed()
}

pub(super) fn indexed_target_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
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

pub(super) fn orbit_param_parser<'src>(
    ctx: ParserContext,
    expression: BoxedParser<'src, SpannedExpr>,
) -> BoxedParser<'src, OrbitParam> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then_ignore(just(Token::Assign))
        .then(range_expr_parser(ctx.clone(), expression))
        .map(
            move |((name, name_span), (start_ast, end_ast, op, range_span))| {
                let span = merge_span(name_span, range_span);
                let info = ctx_for_map.info(span);
                OrbitParam {
                    name,
                    start: start_ast,
                    end: end_ast,
                    op,
                    span: info,
                }
            },
        )
        .boxed()
}

pub(super) fn engrave_param_parser<'src>(ctx: ParserContext) -> BoxedParser<'src, EngraveParam> {
    let ctx_for_map = ctx.clone();
    select! { Token::Identifier(name) => name }
        .map_with(|name, extra| (name, extra.span()))
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map(move |((name, name_span), (ty, ty_span))| {
            let span = merge_span(name_span, ty_span);
            let info = ctx_for_map.info(span);
            EngraveParam {
                name,
                param_type: ty,
                is_morph: false,
                span: info,
            }
        })
        .boxed()
}

pub(super) fn type_parser<'src>() -> BoxedParser<'src, (Type, SimpleSpan<usize>)> {
    choice((
        select! { Token::Type(ty) => ty }.map_with(|ty, extra| (ty, extra.span())),
        select! { Token::Identifier(name) => name }
            .map_with(|name, extra| (Type::Artifact(name), extra.span())),
    ))
    .boxed()
}

pub(super) fn type_keyword_name(ty: &Type) -> String {
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
        Type::Fate => "fate",
        Type::Augury => "augury",
        Type::Artifact(name) => name,
    }
    .to_string()
}
