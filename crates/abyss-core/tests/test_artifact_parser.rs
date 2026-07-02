use abyss_core::ast::{Expr, Stmt, Type};
use abyss_core::parser::parse;

fn parse_single(source: &str) -> Vec<Stmt> {
    let outcome = parse(source);
    assert!(
        outcome.diagnostics.is_empty(),
        "parser emitted diagnostics: {:?}",
        outcome.diagnostics
    );
    outcome.ast
}

#[test]
fn parses_artifact_definition_fields() {
    let ast = parse_single(
        r#"
artifact Player {
    name: rune;
    health: arcana;
};
"#,
    );

    assert_eq!(ast.len(), 1);
    match &ast[0] {
        Stmt::ArtifactDef { name, fields, .. } => {
            assert_eq!(name, "Player");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "name");
            assert_eq!(fields[0].field_type, Type::Rune);
            assert_eq!(fields[1].name, "health");
            assert_eq!(fields[1].field_type, Type::Arcana);
        }
        other => panic!("expected artifact definition, found {:?}", other),
    }
}

#[test]
fn parses_artifact_literal_in_assignment() {
    let ast = parse_single(
        r#"
artifact Player { name: rune; health: arcana; };
forge hero: Player = Player { name: "Ardyn", health: 100 };
"#,
    );

    assert_eq!(ast.len(), 2);
    match &ast[1] {
        Stmt::VarAssign { value, .. } => match value {
            Expr::ArtifactLiteral {
                type_name, fields, ..
            } => {
                assert_eq!(type_name, "Player");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "health");
            }
            other => panic!("expected artifact literal, found {:?}", other),
        },
        other => panic!("expected variable assignment, found {:?}", other),
    }
}

#[test]
fn parses_field_assignment_expression() {
    let ast = parse_single(
        r#"
artifact Player { name: rune; health: arcana; };
forge morph hero: Player = Player { name: "Ardyn", health: 100 };
hero.health = 75;
"#,
    );

    assert_eq!(ast.len(), 3);
    match &ast[2] {
        Stmt::FieldAssignment {
            target,
            field,
            value,
            ..
        } => {
            assert_eq!(field, "health");
            match target {
                Expr::Var(name, _) => assert_eq!(name, "hero"),
                other => panic!("expected base variable, found {:?}", other),
            }
            match value {
                Expr::Arcana(number, _) => assert_eq!(*number, 75),
                other => panic!("expected arcana literal, found {:?}", other),
            }
        }
        other => panic!("expected field assignment, found {:?}", other),
    }
}

#[test]
fn duplicate_artifact_fields_emit_diagnostic() {
    let outcome = parse(
        r#"
artifact Item {
    name: rune;
    name: arcana;
};
"#,
    );

    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diag| diag.label.contains("Duplicate field `name`")),
        "expected duplicate-field diagnostic, got {:?}",
        outcome.diagnostics
    );
}

#[test]
fn parses_artifact_method_definition() {
    let ast = parse_single(
        r#"
artifact Player { level: arcana; };
engrave Player::get_level(core) -> arcana {
    reveal core.level;
};
"#,
    );

    assert_eq!(ast.len(), 2);
    match &ast[1] {
        Stmt::Engrave {
            name,
            params,
            return_type,
            method_target,
            ..
        } => {
            assert_eq!(name, "get_level");
            assert_eq!(*return_type, Type::Arcana);
            let target = method_target
                .as_ref()
                .expect("expected method metadata on engrave statement");
            assert_eq!(target.artifact, "Player");
            assert!(!target.requires_morph);
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "core");
            assert_eq!(params[0].param_type, Type::Artifact("Player".into()));
            assert!(!params[0].is_morph);
        }
        other => panic!("expected engrave statement, found {:?}", other),
    }
}

#[test]
fn parses_method_call_expression() {
    let ast = parse_single(
        r#"
artifact Player { level: arcana; };
forge hero: Player = Player { level: 7 };
hero.get_level();
"#,
    );

    assert_eq!(ast.len(), 3);
    match &ast[2] {
        Stmt::Expr(
            Expr::MethodCall {
                receiver,
                method,
                args,
                ..
            },
            _,
        ) => {
            assert_eq!(method, "get_level");
            assert!(args.is_empty());
            match receiver.as_ref() {
                Expr::Var(name, _) => assert_eq!(name, "hero"),
                other => panic!("expected receiver variable, found {:?}", other),
            }
        }
        other => panic!("expected method-call statement, found {:?}", other),
    }
}
