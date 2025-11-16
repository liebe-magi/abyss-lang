use crate::ast::{AST, AssignmentOp, Type};

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
        Type::Artifact(name) => name.clone(),
    }
}

/// Formats an AST node into a readable string with appropriate indentation.
/// This function handles various types of AST nodes, applying formatting rules based on node type.
/// It also manages operator precedence to ensure correct placement of parentheses.
///
/// # Arguments
/// * `ast` - The AST node to format.
/// * `indent_level` - The level of indentation for the formatted output.
///
/// # Returns
/// A formatted string representation of the AST node.
pub fn format_ast(ast: &AST, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);

    // Determines the precedence level for an AST node to handle operator precedence.
    let precedence = |node: &AST| match node {
        AST::LogicalOr(_, _, _) => 10,
        AST::LogicalAnd(_, _, _) => 20,
        AST::Equal(_, _, _) | AST::NotEqual(_, _, _) => 30,
        AST::LessThan(_, _, _)
        | AST::LessThanOrEqual(_, _, _)
        | AST::GreaterThan(_, _, _)
        | AST::GreaterThanOrEqual(_, _, _) => 40,
        AST::Add(_, _, _) | AST::Sub(_, _, _) => 50,
        AST::Mul(_, _, _) | AST::Div(_, _, _) | AST::Mod(_, _, _) => 60,
        AST::PowArcana(_, _, _) | AST::PowAether(_, _, _) => 70,
        AST::LogicalNot(_, _) => 80,
        AST::IndexAccess { .. } | AST::FieldAccess { .. } => 90,
        _ => 100,
    };

    let current_precedence = precedence(ast);

    // Formats a sub-expression, adding parentheses if necessary based on precedence.
    let format_with_parentheses = |expr: &AST, parent_precedence: u8| -> String {
        let sub_precedence = precedence(expr);
        let code = format_ast(expr, indent_level);

        if sub_precedence < parent_precedence {
            format!("({})", code)
        } else {
            code
        }
    };

    match ast {
        AST::Statement(statement, _) => {
            format!("{}{};", indent, format_ast(statement, indent_level))
        }
        AST::Add(left, right, _)
        | AST::Sub(left, right, _)
        | AST::Mul(left, right, _)
        | AST::Div(left, right, _)
        | AST::Mod(left, right, _)
        | AST::PowArcana(left, right, _)
        | AST::PowAether(left, right, _)
        | AST::LogicalAnd(left, right, _)
        | AST::LogicalOr(left, right, _)
        | AST::Equal(left, right, _)
        | AST::NotEqual(left, right, _)
        | AST::LessThan(left, right, _)
        | AST::LessThanOrEqual(left, right, _)
        | AST::GreaterThan(left, right, _)
        | AST::GreaterThanOrEqual(left, right, _) => {
            let operator = match ast {
                AST::Add(_, _, _) => "+",
                AST::Sub(_, _, _) => "-",
                AST::Mul(_, _, _) => "*",
                AST::Div(_, _, _) => "/",
                AST::Mod(_, _, _) => "%",
                AST::PowArcana(_, _, _) => "^",
                AST::PowAether(_, _, _) => "**",
                AST::LogicalAnd(_, _, _) => "&&",
                AST::LogicalOr(_, _, _) => "||",
                AST::Equal(_, _, _) => "==",
                AST::NotEqual(_, _, _) => "!=",
                AST::LessThan(_, _, _) => "<",
                AST::LessThanOrEqual(_, _, _) => "<=",
                AST::GreaterThan(_, _, _) => ">",
                AST::GreaterThanOrEqual(_, _, _) => ">=",
                _ => unreachable!(),
            };
            format!(
                "{} {} {}",
                format_with_parentheses(left, current_precedence),
                operator,
                format_with_parentheses(right, current_precedence)
            )
        }
        AST::LogicalNot(expr, _) => {
            format!("!{}", format_with_parentheses(expr, current_precedence))
        }
        AST::VarAssign {
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
                format_ast(value, indent_level)
            )
        }
        AST::Assignment {
            name, value, op, ..
        } => match op {
            AssignmentOp::Assign => format!("{} = {}", name, format_ast(value, indent_level)),
            AssignmentOp::AddAssign => {
                format!("{} += {}", name, format_ast(value, indent_level))
            }
            AssignmentOp::SubAssign => {
                format!("{} -= {}", name, format_ast(value, indent_level))
            }
            AssignmentOp::MulAssign => {
                format!("{} *= {}", name, format_ast(value, indent_level))
            }
            AssignmentOp::DivAssign => {
                format!("{} /= {}", name, format_ast(value, indent_level))
            }
            AssignmentOp::ModAssign => {
                format!("{} %= {}", name, format_ast(value, indent_level))
            }
            AssignmentOp::PowArcanaAssign => {
                format!("{} ^= {}", name, format_ast(value, indent_level))
            }
            AssignmentOp::PowAetherAssign => {
                format!("{} **= {}", name, format_ast(value, indent_level))
            }
        },
        AST::Var(name, _) => name.clone(),
        AST::FieldAccess { target, field, .. } => {
            format!("{}.{}", format_ast(target, indent_level), field)
        }
        AST::Arcana(value, _) => format!("{}", value),
        AST::Aether(value, _) => {
            if value.fract() == 0.0 {
                format!("{:.1}", value)
            } else {
                format!("{}", value)
            }
        }
        AST::Rune(value, _) => format!("\"{}\"", value),
        AST::Omen(value, _) => match value {
            true => "boon".to_string(),
            false => "hex".to_string(),
        },
        AST::Abyss(_) => "abyss".to_string(),
        AST::Reveal(value, _) => {
            let val = format_ast(value, indent_level);
            let trimmed_val = val.trim();
            match trimmed_val {
                "abyss" => "reveal".to_string(),
                _ => format!("reveal {}", trimmed_val),
            }
        }
        AST::Block(statements, _) => {
            let mut result = format!("{}{{\n", indent);
            for statement in statements {
                result.push_str(&format!("{}\n", format_ast(statement, indent_level + 1)));
            }
            result.push_str(&format!("{}}}", indent));
            result
        }
        AST::Oracle {
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
                            format_ast(cond.expression.as_ref(), indent_level)
                        } else {
                            format!(
                                "{} = {}",
                                cond.variable,
                                format_ast(cond.expression.as_ref(), indent_level)
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                result.push_str(&format!(" ({})", conditions));
            }
            result.push_str(" {\n");
            for branch in branches {
                if let AST::Comment(text, _) = branch {
                    result.push_str(&format!("{}{}\n", "    ".repeat(indent_level + 1), text));
                    continue;
                }

                if let AST::OracleBranch { pattern, body, .. } = branch {
                    let pattern = pattern
                        .iter()
                        .map(|pat| format_ast(pat, indent_level + 1))
                        .collect::<Vec<_>>()
                        .join(", ");
                    result.push_str(&format!(
                        "{}{} => {}\n",
                        "    ".repeat(indent_level + 1),
                        if pattern.is_empty() {
                            "_".to_string()
                        } else {
                            format!("({})", pattern)
                        },
                        format_ast(body.as_ref(), indent_level + 1).trim()
                    ));
                }
            }
            result.push_str(&format!("{}}}", indent));
            result
        }
        AST::OracleDontCareItem(_) => "_".to_string(),
        AST::Orbit { params, body, .. } => {
            let mut result = "orbit".to_string();
            if !params.is_empty() {
                let params_str = params
                    .iter()
                    .map(|param| format_ast(param, indent_level))
                    .collect::<Vec<_>>()
                    .join(", ");
                result.push_str(&format!(" ({})", params_str));
            }
            result.push_str(format_ast(body.as_ref(), indent_level).trim());
            result
        }
        AST::OrbitParam {
            name,
            start,
            end,
            op,
            ..
        } => {
            let start_expr = format_ast(start, 0);
            let end_expr = format_ast(end, 0);
            format!("{} = {}{}{}", name, start_expr, op, end_expr)
        }
        AST::Resume(value, _) => match value {
            Some(idendifier) => format!("resume {}", idendifier),
            None => "resume".to_string(),
        },
        AST::Eject(value, _) => match value {
            Some(idendifier) => format!("eject {}", idendifier),
            None => "eject".to_string(),
        },
        AST::Engrave {
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
                param_strings.push(format_ast(param, indent_level));
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
                    format_ast(body, indent_level)
                ),
                Some(ret) => format!(
                    "engrave {}({}) -> {} {}",
                    qualified_name,
                    params_str,
                    ret,
                    format_ast(body, indent_level)
                ),
            }
        }
        AST::EngraveParam {
            name,
            param_type,
            is_morph,
            ..
        } => {
            let qualifier = if *is_morph { "morph " } else { "" };
            format!("{}{}: {}", qualifier, name, type_keyword(param_type))
        }
        AST::FuncCall { name, args, .. } => {
            let args_str = args
                .iter()
                .map(|arg| format_ast(arg, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, args_str)
        }
        AST::ListLiteral { elements, .. } => {
            let contents = elements
                .iter()
                .map(|elem| format_ast(elem, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", contents)
        }
        AST::MapLiteral { entries, .. } => {
            let contents = entries
                .iter()
                .map(|(key, value)| format!("\"{}\": {}", key, format_ast(value, indent_level)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", contents)
        }
        AST::ArtifactLiteral {
            type_name, fields, ..
        } => {
            if fields.is_empty() {
                format!("{} {{}}", type_name)
            } else {
                let contents = fields
                    .iter()
                    .map(|(field, value)| format!("{}: {}", field, format_ast(value, indent_level)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", type_name, contents)
            }
        }
        AST::IndexAccess { target, index, .. } => {
            format!(
                "{}[{}]",
                format_ast(target, indent_level),
                format_ast(index, indent_level)
            )
        }
        AST::IndexAssignment {
            target,
            index,
            value,
            ..
        } => format!(
            "{}[{}] = {}",
            format_ast(target, indent_level),
            format_ast(index, indent_level),
            format_ast(value, indent_level)
        ),
        AST::FieldAssignment {
            target,
            field,
            value,
            ..
        } => format!(
            "{}.{} = {}",
            format_ast(target, indent_level),
            field,
            format_ast(value, indent_level)
        ),
        AST::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            let args_str = args
                .iter()
                .map(|arg| format_ast(arg, indent_level))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}.{}({})",
                format_ast(receiver, indent_level),
                method,
                args_str
            )
        }
        AST::ArtifactDef { name, fields, .. } => {
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
        AST::Comment(text, _) => text.clone(),
        _ => format!("Not implemented: {:?}", ast),
    }
}
