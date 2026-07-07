use crate::ast::{AssignmentOp, Expr, Pattern, Stmt, Type};
use crate::parser::SourceComment;

/// Walks the comment list in source order while statements are being
/// formatted, so each comment is emitted exactly once, next to the
/// statement it preceded (or trailed) in the original source.
struct CommentCursor<'a> {
    comments: &'a [SourceComment],
    source: &'a str,
    next: usize,
}

impl<'a> CommentCursor<'a> {
    /// Emit (indented, one per line) every not-yet-consumed comment that
    /// starts before `pos`.
    fn flush_before(&mut self, pos: usize, indent_level: usize, out: &mut String) {
        let indent = "    ".repeat(indent_level);
        while self.next < self.comments.len() && self.comments[self.next].span.start() < pos {
            for line in self.comments[self.next].text.lines() {
                out.push_str(&indent);
                out.push_str(line.trim_end());
                out.push('\n');
            }
            self.next += 1;
        }
    }

    /// If the next comment sits on the same source line as `stmt_end`
    /// (no newline between them), consume it and return its text so the
    /// caller can append it to the formatted statement line.
    fn take_trailing(&mut self, stmt_end: usize) -> Option<String> {
        let comment = self.comments.get(self.next)?;
        let start = comment.span.start();
        if start < stmt_end || self.source.get(stmt_end..start)?.contains('\n') {
            return None;
        }
        self.next += 1;
        Some(comment.text.clone())
    }
}

/// Format a whole parsed program, re-emitting the comments collected by
/// [`crate::parser::collect_comments`] next to the statements they
/// accompanied: full-line comments stay above their statement (including
/// inside blocks), a comment trailing a statement on the same line is
/// re-attached to the formatted line, and comments after the last
/// statement land at the end.
///
/// Comments embedded somewhere the formatter cannot represent (inside an
/// expression, between oracle arms) are moved up to the closest preceding
/// statement boundary.
pub fn format_program(source: &str, stmts: &[Stmt], comments: &[SourceComment]) -> String {
    let mut cursor = CommentCursor {
        comments,
        source,
        next: 0,
    };
    let mut out = String::new();
    for stmt in stmts {
        if let Some(span) = stmt_span(stmt) {
            cursor.flush_before(span.start(), 0, &mut out);
        }
        out.push_str(&format_stmt_inner(stmt, 0, &mut Some(&mut cursor)));
        if let Some(span) = stmt_span(stmt)
            && let Some(trailing) = cursor.take_trailing(span.end() + 1)
        {
            out.push(' ');
            out.push_str(&trailing);
        }
        out.push('\n');
    }
    cursor.flush_before(usize::MAX, 0, &mut out);
    out
}

fn stmt_span(stmt: &Stmt) -> Option<crate::span::Span> {
    match stmt {
        Stmt::Expr(_, span)
        | Stmt::Reveal(_, span)
        | Stmt::Block(_, span)
        | Stmt::Comment(_, span)
        | Stmt::Revolve(_, span)
        | Stmt::Eject(_, span)
        | Stmt::VarAssign { span, .. }
        | Stmt::Assignment { span, .. }
        | Stmt::IndexAssignment { span, .. }
        | Stmt::FieldAssignment { span, .. }
        | Stmt::Orbit { span, .. }
        | Stmt::Engrave { span, .. }
        | Stmt::ArtifactDef { span, .. } => *span,
    }
}

fn type_keyword(var_type: &Type) -> String {
    match var_type {
        Type::Arcana => "arcana".to_string(),
        Type::Aether => "aether".to_string(),
        Type::Rune => "rune".to_string(),
        Type::Omen => "omen".to_string(),
        Type::Abyss => "abyss".to_string(),
        Type::Scroll => "scroll".to_string(),
        Type::Lexicon => "lexicon".to_string(),
        Type::Materia => "materia".to_string(),
        Type::Glyph => "glyph".to_string(),
        Type::Fate => "fate".to_string(),
        Type::Augury => "augury".to_string(),
        Type::Artifact(name) => name.clone(),
    }
}

/// Formats a top-level statement as a terminated line: indentation,
/// the statement body, and the trailing semicolon. This is the entry
/// point the CLI's `align` subcommand and the REPL echo use.
pub fn format_stmt(stmt: &Stmt, indent_level: usize) -> String {
    format_stmt_inner(stmt, indent_level, &mut None)
}

fn format_stmt_inner(
    stmt: &Stmt,
    indent_level: usize,
    comments: &mut Option<&mut CommentCursor>,
) -> String {
    let indent = "    ".repeat(indent_level);
    format!(
        "{}{};",
        indent,
        format_stmt_body_inner(stmt, indent_level, comments)
    )
}

/// Formats the body of a statement without indentation or the trailing
/// semicolon — the shape used inside oracle arm bodies and by
/// [`format_stmt`].
fn format_stmt_body(stmt: &Stmt, indent_level: usize) -> String {
    format_stmt_body_inner(stmt, indent_level, &mut None)
}

fn format_stmt_body_inner(
    stmt: &Stmt,
    indent_level: usize,
    comments: &mut Option<&mut CommentCursor>,
) -> String {
    let indent = "    ".repeat(indent_level);

    match stmt {
        Stmt::Expr(expr, _) => format_expr(expr, indent_level),
        Stmt::VarAssign {
            name,
            value,
            var_type,
            is_morph,
            ..
        } => {
            format!(
                "forge {}{}: {} = {}",
                if *is_morph { "morph " } else { "" },
                name,
                type_keyword(var_type),
                format_expr(value, indent_level)
            )
        }
        Stmt::Assignment {
            name, value, op, ..
        } => {
            let operator = match op {
                AssignmentOp::Assign => "=",
                AssignmentOp::AddAssign => "+=",
                AssignmentOp::SubAssign => "-=",
                AssignmentOp::MulAssign => "*=",
                AssignmentOp::DivAssign => "/=",
                AssignmentOp::ModAssign => "%=",
                AssignmentOp::PowArcanaAssign => "^=",
                AssignmentOp::PowAetherAssign => "**=",
            };
            format!("{} {} {}", name, operator, format_expr(value, indent_level))
        }
        Stmt::Reveal(value, _) => {
            let val = format_expr(value, indent_level);
            let trimmed_val = val.trim();
            match trimmed_val {
                "abyss" => "reveal".to_string(),
                _ => format!("reveal {}", trimmed_val),
            }
        }
        Stmt::Block(statements, block_span) => {
            let mut result = format!("{}{{\n", indent);
            for statement in statements {
                if let (Some(cursor), Some(span)) = (comments.as_deref_mut(), stmt_span(statement))
                {
                    cursor.flush_before(span.start(), indent_level + 1, &mut result);
                }
                result.push_str(&format_stmt_inner(statement, indent_level + 1, comments));
                if let (Some(cursor), Some(span)) = (comments.as_deref_mut(), stmt_span(statement))
                    && let Some(trailing) = cursor.take_trailing(span.end() + 1)
                {
                    result.push(' ');
                    result.push_str(&trailing);
                }
                result.push('\n');
            }
            if let (Some(cursor), Some(span)) = (comments.as_deref_mut(), block_span) {
                cursor.flush_before(span.end(), indent_level + 1, &mut result);
            }
            result.push_str(&format!("{}}}", indent));
            result
        }
        Stmt::Orbit { params, body, .. } => {
            let mut result = "orbit".to_string();
            if !params.is_empty() {
                let params_str = params
                    .iter()
                    .map(|param| {
                        let start_expr = format_expr(&param.start, 0);
                        let end_expr = format_expr(&param.end, 0);
                        format!("{} = {}{}{}", param.name, start_expr, param.op, end_expr)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                result.push_str(&format!(" ({})", params_str));
            }
            result.push_str(
                format_stmt_body_inner(body.as_ref(), indent_level, comments).trim_start(),
            );
            result
        }
        Stmt::Revolve(value, _) => match value {
            Some(identifier) => format!("revolve {}", identifier),
            None => "revolve".to_string(),
        },
        Stmt::Eject(value, _) => match value {
            Some(identifier) => format!("eject {}", identifier),
            None => "eject".to_string(),
        },
        Stmt::Engrave {
            name,
            params,
            return_type,
            body,
            method_target,
            ..
        } => {
            let return_type_str = match return_type {
                Type::Abyss => None,
                _ => Some(type_keyword(return_type)),
            };
            let mut param_strings = Vec::new();
            let mut iter = params.iter();
            if let Some(target) = method_target {
                let receiver = if target.requires_morph {
                    "morph core"
                } else {
                    "core"
                };
                param_strings.push(receiver.to_string());
                debug_assert!(
                    !params.is_empty(),
                    "Artifact method with method_target must have at least one parameter (the receiver)"
                );
                iter.next();
            }
            for param in iter {
                let qualifier = if param.is_morph { "morph " } else { "" };
                param_strings.push(format!(
                    "{}{}: {}",
                    qualifier,
                    param.name,
                    type_keyword(&param.param_type)
                ));
            }
            let params_str = param_strings.join(", ");
            let qualified_name = if let Some(target) = method_target {
                format!("{}::{}", target.artifact, name)
            } else {
                name.clone()
            };
            match return_type_str {
                None => format!(
                    "engrave {}({}) {}",
                    qualified_name,
                    params_str,
                    format_stmt_body_inner(body, indent_level, comments)
                ),
                Some(ret) => format!(
                    "engrave {}({}) -> {} {}",
                    qualified_name,
                    params_str,
                    ret,
                    format_stmt_body_inner(body, indent_level, comments)
                ),
            }
        }
        Stmt::IndexAssignment {
            target,
            index,
            value,
            ..
        } => format!(
            "{}[{}] = {}",
            format_expr(target, indent_level),
            format_expr(index, indent_level),
            format_expr(value, indent_level)
        ),
        Stmt::FieldAssignment {
            target,
            field,
            value,
            ..
        } => format!(
            "{}.{} = {}",
            format_expr(target, indent_level),
            field,
            format_expr(value, indent_level)
        ),
        Stmt::ArtifactDef { name, fields, .. } => {
            let mut result = format!("artifact {} {{\n", name);
            for field in fields {
                result.push_str(&format!(
                    "{}{}: {};\n",
                    "    ".repeat(indent_level + 1),
                    field.name,
                    type_keyword(&field.field_type)
                ));
            }
            result.push_str(&format!("{}}}", indent));
            result
        }
        Stmt::Comment(text, _) => text.clone(),
    }
}

// Determines the precedence level for an expression to decide where
// parentheses are required on round-trip.
fn precedence(node: &Expr) -> u8 {
    match node {
        Expr::LogicalOr(_, _, _) => 10,
        Expr::LogicalAnd(_, _, _) => 20,
        Expr::Equal(_, _, _) | Expr::NotEqual(_, _, _) => 30,
        Expr::LessThan(_, _, _)
        | Expr::LessThanOrEqual(_, _, _)
        | Expr::GreaterThan(_, _, _)
        | Expr::GreaterThanOrEqual(_, _, _) => 40,
        Expr::Add(_, _, _) | Expr::Sub(_, _, _) => 50,
        Expr::Mul(_, _, _) | Expr::Div(_, _, _) | Expr::Mod(_, _, _) => 60,
        Expr::PowArcana(_, _, _) | Expr::PowAether(_, _, _) => 70,
        Expr::LogicalNot(_, _) => 80,
        Expr::IndexAccess { .. } | Expr::FieldAccess { .. } | Expr::Propagate(_, _) => 90,
        _ => 100,
    }
}

/// Formats an expression into a readable string, adding parentheses
/// where operator precedence requires them for a faithful round-trip.
pub fn format_expr(expr: &Expr, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);
    let current_precedence = precedence(expr);

    // Formats a sub-expression, adding parentheses if necessary based on precedence.
    let format_with_parentheses = |sub: &Expr, parent_precedence: u8| -> String {
        let sub_precedence = precedence(sub);
        let code = format_expr(sub, indent_level);

        if sub_precedence < parent_precedence {
            format!("({})", code)
        } else {
            code
        }
    };

    match expr {
        Expr::Add(left, right, _)
        | Expr::Sub(left, right, _)
        | Expr::Mul(left, right, _)
        | Expr::Div(left, right, _)
        | Expr::Mod(left, right, _)
        | Expr::PowArcana(left, right, _)
        | Expr::PowAether(left, right, _)
        | Expr::LogicalAnd(left, right, _)
        | Expr::LogicalOr(left, right, _)
        | Expr::Equal(left, right, _)
        | Expr::NotEqual(left, right, _)
        | Expr::LessThan(left, right, _)
        | Expr::LessThanOrEqual(left, right, _)
        | Expr::GreaterThan(left, right, _)
        | Expr::GreaterThanOrEqual(left, right, _) => {
            let operator = match expr {
                Expr::Add(_, _, _) => "+",
                Expr::Sub(_, _, _) => "-",
                Expr::Mul(_, _, _) => "*",
                Expr::Div(_, _, _) => "/",
                Expr::Mod(_, _, _) => "%",
                Expr::PowArcana(_, _, _) => "^",
                Expr::PowAether(_, _, _) => "**",
                Expr::LogicalAnd(_, _, _) => "&&",
                Expr::LogicalOr(_, _, _) => "||",
                Expr::Equal(_, _, _) => "==",
                Expr::NotEqual(_, _, _) => "!=",
                Expr::LessThan(_, _, _) => "<",
                Expr::LessThanOrEqual(_, _, _) => "<=",
                Expr::GreaterThan(_, _, _) => ">",
                Expr::GreaterThanOrEqual(_, _, _) => ">=",
                _ => unreachable!(),
            };
            format!(
                "{} {} {}",
                format_with_parentheses(left, current_precedence),
                operator,
                format_with_parentheses(right, current_precedence)
            )
        }
        Expr::LogicalNot(inner, _) => {
            format!("!{}", format_with_parentheses(inner, current_precedence))
        }
        Expr::Propagate(inner, _) => {
            format!("{}?", format_with_parentheses(inner, current_precedence))
        }
        Expr::Var(name, _) => name.clone(),
        Expr::FieldAccess { target, field, .. } => {
            format!("{}.{}", format_expr(target, indent_level), field)
        }
        Expr::Arcana(value, _) => format!("{}", value),
        Expr::Aether(value, _) => {
            if value.fract() == 0.0 {
                format!("{:.1}", value)
            } else {
                format!("{}", value)
            }
        }
        Expr::Rune(value, _) => format!("\"{}\"", value),
        Expr::Omen(value, _) => match value {
            true => "boon".to_string(),
            false => "hex".to_string(),
        },
        Expr::Abyss(_) => "abyss".to_string(),
        Expr::Oracle {
            is_match,
            conditionals,
            branches,
            ..
        } => {
            let mut result = "oracle".to_string();
            if !conditionals.is_empty() {
                let conditions = conditionals
                    .iter()
                    .map(|cond| {
                        if *is_match {
                            format_expr(cond.expression.as_ref(), indent_level)
                        } else {
                            format!(
                                "{} = {}",
                                cond.variable,
                                format_expr(cond.expression.as_ref(), indent_level)
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                result.push_str(&format!(" ({})", conditions));
            }
            result.push_str(" {\n");
            for branch in branches {
                // Top-level scroll / artifact patterns keep their natural
                // form (`[…] =>`, `Player { name } =>`) rather than
                // getting re-wrapped in `(…)`. The single-element pattern
                // already self-formats in the right shape, so the outer
                // parens would be redundant noise on round-trip.
                let pattern_text = match branch.pattern.as_slice() {
                    [] => "_".to_string(),
                    [only @ Pattern::Scroll { .. }]
                    | [only @ Pattern::Artifact { .. }]
                    | [only @ Pattern::Lexicon { .. }] => format_pattern(only, indent_level + 1),
                    elems => {
                        let inner = elems
                            .iter()
                            .map(|pat| format_pattern(pat, indent_level + 1))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("({})", inner)
                    }
                };
                let guard_text = branch
                    .guard
                    .as_ref()
                    .map(|guard| format!(" ward {}", format_expr(guard, indent_level + 1)))
                    .unwrap_or_default();
                // Single-statement arm bodies are semicolon-terminated on
                // round-trip (`=> reveal x;`), while block bodies end at
                // their closing brace — mirroring what the parser accepts.
                let body_text = match &branch.body {
                    Stmt::Block(..) => format_stmt_body(&branch.body, indent_level + 1),
                    other => format_stmt(other, indent_level + 1),
                };
                result.push_str(&format!(
                    "{}{}{} => {}\n",
                    "    ".repeat(indent_level + 1),
                    pattern_text,
                    guard_text,
                    body_text.trim()
                ));
            }
            result.push_str(&format!("{}}}", indent));
            result
        }
        Expr::FuncCall { name, args, .. } => {
            let args_str = args
                .iter()
                .map(|arg| format_expr(arg, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, args_str)
        }
        Expr::ListLiteral { elements, .. } => {
            let contents = elements
                .iter()
                .map(|elem| format_expr(elem, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", contents)
        }
        Expr::MapLiteral { entries, .. } => {
            let contents = entries
                .iter()
                .map(|(key, value)| format!("\"{}\": {}", key, format_expr(value, indent_level)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", contents)
        }
        Expr::ArtifactLiteral {
            type_name, fields, ..
        } => {
            if fields.is_empty() {
                format!("{} {{}}", type_name)
            } else {
                let contents = fields
                    .iter()
                    .map(|(field, value)| {
                        format!("{}: {}", field, format_expr(value, indent_level))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", type_name, contents)
            }
        }
        Expr::IndexAccess { target, index, .. } => {
            format!(
                "{}[{}]",
                format_expr(target, indent_level),
                format_expr(index, indent_level)
            )
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            let args_str = args
                .iter()
                .map(|arg| format_expr(arg, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}.{}({})",
                format_expr(receiver, indent_level),
                method,
                args_str
            )
        }
    }
}

/// Formats an oracle match-arm pattern.
pub fn format_pattern(pattern: &Pattern, indent_level: usize) -> String {
    match pattern {
        Pattern::DontCare(_) => "_".to_string(),
        Pattern::Scroll { elements, .. } => {
            let inner = elements
                .iter()
                .map(|elem| format_pattern(elem, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", inner)
        }
        Pattern::Rest { name, .. } => match name {
            Some(name) => format!("..{}", name),
            None => "..".to_string(),
        },
        Pattern::Artifact {
            type_name, fields, ..
        } => {
            if fields.is_empty() {
                // `Type {}` matches by type alone (any artifact of this
                // type). Mirror `ArtifactLiteral`'s empty-fields formatting
                // rather than emitting `Type {  }` with double spaces.
                format!("{} {{}}", type_name)
            } else {
                let inner = fields
                    .iter()
                    .map(|(name, sub)| match sub {
                        // Shorthand `{ name }` keeps its compact form when the
                        // sub-pattern is just `Var(name)` with the same name.
                        Pattern::Expr(Expr::Var(var_name, _)) if var_name == name => name.clone(),
                        other => format!("{}: {}", name, format_pattern(other, indent_level)),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", type_name, inner)
            }
        }
        Pattern::Lexicon { entries, .. } => {
            if entries.is_empty() {
                // `{}` matches any lexicon (the "by shape" catch-all).
                "{}".to_string()
            } else {
                let inner = entries
                    .iter()
                    .map(|(key, sub)| format!("\"{}\": {}", key, format_pattern(sub, indent_level)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", inner)
            }
        }
        Pattern::Expr(expr) => format_expr(expr, indent_level),
    }
}
