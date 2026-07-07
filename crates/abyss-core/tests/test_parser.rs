use abyss_core::parser::{ParserDiagnostic, SimpleSpan, emit_diagnostics, parse};

#[test]
fn parse_reports_unexpected_token_with_help() {
    let outcome = parse(
        r#"
forge hero: arcana = ;
"#,
    );

    assert!(
        !outcome.diagnostics.is_empty(),
        "expected parser diagnostics, got none"
    );

    let diag = &outcome.diagnostics[0];
    assert_eq!(diag.title, "Spell error: Incantation failed");
    assert!(diag.label.starts_with("Unexpected token"));
    assert!(
        diag.help
            .as_ref()
            .is_some_and(|msg| msg.contains("Perhaps you meant one of:")),
        "expected helpful suggestion, got {:?}",
        diag.help
    );
}

#[test]
fn emit_diagnostics_prints_reports() {
    let diagnostics = vec![ParserDiagnostic {
        title: "Test error".into(),
        label: "Something went wrong".into(),
        span: SimpleSpan::new(0, 1),
        help: Some("Try adding a semicolon".into()),
    }];

    emit_diagnostics("<test>", "artifact Foo {};", &diagnostics)
        .expect("ariadne should print diagnostics successfully");
}

#[test]
fn incant_malformed_braces_emit_diagnostics() {
    for (source, needle) in [
        (r#"incant "unclosed {name";"#, "unclosed"),
        (r#"incant "empty {}";"#, "empty"),
        (r#"incant "stray } here";"#, "stray"),
        (r#"incant "bad {na-me}";"#, "identifier"),
    ] {
        let outcome = parse(source);
        assert!(
            outcome
                .diagnostics
                .iter()
                .any(|diag| diag.label.contains(needle)),
            "expected `{}` diagnostic for {}, got {:?}",
            needle,
            source,
            outcome.diagnostics
        );
    }
}
