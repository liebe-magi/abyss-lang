use abyss_core::ast::{
    ArtifactField, ArtifactMethodTarget, AssignmentOp, ConditionalAssignment, EngraveParam, Expr,
    OracleBranch, OrbitParam, Pattern, Stmt, Type,
};
use abyss_core::format::{format_expr, format_pattern, format_program, format_stmt};
use abyss_core::parser::{collect_comments, parse};

fn arcana(value: i64) -> Box<Expr> {
    Box::new(Expr::Arcana(value, None))
}

fn rune(value: &str) -> Expr {
    Expr::Rune(value.to_string(), None)
}

fn omen(value: bool) -> Box<Expr> {
    Box::new(Expr::Omen(value, None))
}

fn var(name: &str) -> Expr {
    Expr::Var(name.to_string(), None)
}

fn reveal(value: Expr) -> Stmt {
    Stmt::Reveal(value, None)
}

#[test]
fn format_basic_expressions_and_assignments() {
    let expr_stmt = Stmt::Expr(
        Expr::Mul(
            Box::new(Expr::Add(arcana(1), arcana(2), None)),
            arcana(3),
            None,
        ),
        None,
    );
    assert_eq!(format_stmt(&expr_stmt, 1), "    (1 + 2) * 3;");

    let logical = Expr::LogicalNot(omen(false), None);
    assert_eq!(format_expr(&logical, 0), "!hex");

    let var_assign = Stmt::VarAssign {
        name: "sigil".into(),
        value: rune("alpha"),
        var_type: Type::Rune,
        is_morph: true,
        span: None,
    };
    assert_eq!(
        format_stmt(&var_assign, 0),
        "forge morph sigil: rune = \"alpha\";"
    );

    let assign = Stmt::Assignment {
        name: "sigil".into(),
        value: var("base"),
        op: AssignmentOp::PowAetherAssign,
        span: None,
    };
    assert_eq!(format_stmt(&assign, 0), "sigil **= base;");

    assert_eq!(format_expr(&Expr::Arcana(7, None), 0), "7");
    assert_eq!(format_expr(&Expr::Aether(7.0, None), 0), "7.0");
    assert_eq!(format_expr(&Expr::Aether(7.25, None), 0), "7.25");
    assert_eq!(format_expr(&Expr::Rune("echo".into(), None), 0), "\"echo\"");
    assert_eq!(format_expr(&Expr::Omen(true, None), 0), "boon");
    assert_eq!(format_expr(&Expr::Abyss(None), 0), "abyss");

    let reveal_core = reveal(var("sigil"));
    assert_eq!(format_stmt(&reveal_core, 0), "reveal sigil;");

    let reveal_abyss = reveal(Expr::Abyss(None));
    assert_eq!(format_stmt(&reveal_abyss, 0), "reveal;");

    let field_access = Expr::FieldAccess {
        target: Box::new(var("relic")),
        field: "core".into(),
        span: None,
    };
    assert_eq!(format_expr(&field_access, 0), "relic.core");

    let field_assignment = Stmt::FieldAssignment {
        target: var("relic"),
        field: "core".into(),
        value: *arcana(5),
        span: None,
    };
    assert_eq!(format_stmt(&field_assignment, 0), "relic.core = 5;");

    let index_assignment = Stmt::IndexAssignment {
        target: var("sigils"),
        index: *arcana(2),
        value: rune("beta"),
        span: None,
    };
    assert_eq!(format_stmt(&index_assignment, 0), "sigils[2] = \"beta\";");

    let method_call = Expr::MethodCall {
        receiver: Box::new(var("relic")),
        method: "ignite".into(),
        args: vec![rune("spark")],
        span: None,
    };
    assert_eq!(format_expr(&method_call, 0), "relic.ignite(\"spark\")");
}

#[test]
fn format_collections_and_literals() {
    let list = Expr::ListLiteral {
        elements: vec![
            Expr::Arcana(1, None),
            Expr::Arcana(2, None),
            Expr::Arcana(3, None),
        ],
        span: None,
    };
    assert_eq!(format_expr(&list, 0), "[1, 2, 3]");

    let map = Expr::MapLiteral {
        entries: vec![
            ("name".into(), Expr::Rune("abyss".into(), None)),
            ("value".into(), Expr::Arcana(3, None)),
        ],
        span: None,
    };
    assert_eq!(format_expr(&map, 0), "{\"name\": \"abyss\", \"value\": 3}");

    let artifact_literal = Expr::ArtifactLiteral {
        type_name: "Relic".into(),
        fields: vec![
            ("sigil".into(), Expr::Rune("alpha".into(), None)),
            ("power".into(), Expr::Arcana(9, None)),
        ],
        span: None,
    };
    assert_eq!(
        format_expr(&artifact_literal, 0),
        "Relic { sigil: \"alpha\", power: 9 }"
    );

    let artifact_empty = Expr::ArtifactLiteral {
        type_name: "Relic".into(),
        fields: vec![],
        span: None,
    };
    assert_eq!(format_expr(&artifact_empty, 0), "Relic {}");

    let artifact_def = Stmt::ArtifactDef {
        name: "Relic".into(),
        fields: vec![
            ArtifactField {
                name: "sigil".into(),
                field_type: Type::Rune,
                span: None,
            },
            ArtifactField {
                name: "power".into(),
                field_type: Type::Arcana,
                span: None,
            },
        ],
        span: None,
    };
    let expected = concat!(
        "artifact Relic {\n",
        "    sigil: rune;\n",
        "    power: arcana;\n",
        "};"
    );
    assert_eq!(format_stmt(&artifact_def, 0), expected);

    let list_access = Expr::IndexAccess {
        target: Box::new(var("sigils")),
        index: arcana(0),
        span: None,
    };
    assert_eq!(format_expr(&list_access, 0), "sigils[0]");

    let func_call = Expr::FuncCall {
        name: "summon".into(),
        args: vec![Expr::Rune("echo".into(), None), Expr::Arcana(1, None)],
        span: None,
    };
    assert_eq!(format_expr(&func_call, 0), "summon(\"echo\", 1)");
}

#[test]
fn format_control_flow_and_functions() {
    let block = Stmt::Block(vec![reveal(var("sigil"))], None);
    assert_eq!(format_stmt(&block, 0), "{\n    reveal sigil;\n};");

    let oracle = Expr::Oracle {
        is_match: false,
        conditionals: Vec::new(),
        branches: vec![
            OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, None))],
                guard: None,
                body: reveal(var("spark")),
                span: None,
            },
            OracleBranch {
                pattern: vec![Pattern::DontCare(None)],
                guard: None,
                body: reveal(rune("wild")),
                span: None,
            },
            OracleBranch {
                pattern: vec![],
                guard: None,
                body: reveal(Expr::Abyss(None)),
                span: None,
            },
        ],
        span: None,
    };
    let oracle_expected = concat!(
        "oracle {\n",
        "    (1) => reveal spark;\n",
        "    (_) => reveal \"wild\";\n",
        "    _ => reveal;\n",
        "}"
    );
    assert_eq!(format_expr(&oracle, 0), oracle_expected);

    let oracle_with_ward = Expr::Oracle {
        is_match: true,
        conditionals: vec![ConditionalAssignment {
            variable: "__match_0".into(),
            expression: Box::new(var("count")),
            span: None,
        }],
        branches: vec![
            OracleBranch {
                pattern: vec![Pattern::Expr(Expr::Arcana(1, None))],
                guard: Some(Expr::GreaterThan(Box::new(var("count")), arcana(0), None)),
                body: reveal(rune("ready")),
                span: None,
            },
            OracleBranch {
                pattern: vec![Pattern::DontCare(None)],
                guard: None,
                body: reveal(rune("idle")),
                span: None,
            },
        ],
        span: None,
    };
    let oracle_with_ward_expected = concat!(
        "oracle (count) {\n",
        "    (1) ward count > 0 => reveal \"ready\";\n",
        "    (_) => reveal \"idle\";\n",
        "}"
    );
    assert_eq!(format_expr(&oracle_with_ward, 0), oracle_with_ward_expected);

    let oracle_with_scroll_pattern = Expr::Oracle {
        is_match: true,
        conditionals: vec![ConditionalAssignment {
            variable: "__match_0".into(),
            expression: Box::new(var("xs")),
            span: None,
        }],
        branches: vec![
            OracleBranch {
                pattern: vec![Pattern::Scroll {
                    elements: vec![],
                    span: None,
                }],
                guard: None,
                body: reveal(rune("empty")),
                span: None,
            },
            OracleBranch {
                pattern: vec![Pattern::Scroll {
                    elements: vec![
                        Pattern::Expr(var("head")),
                        Pattern::Rest {
                            name: Some("rest".into()),
                            span: None,
                        },
                    ],
                    span: None,
                }],
                guard: None,
                body: reveal(var("head")),
                span: None,
            },
        ],
        span: None,
    };
    let oracle_with_scroll_pattern_expected = concat!(
        "oracle (xs) {\n",
        "    [] => reveal \"empty\";\n",
        "    [head, ..rest] => reveal head;\n",
        "}"
    );
    assert_eq!(
        format_expr(&oracle_with_scroll_pattern, 0),
        oracle_with_scroll_pattern_expected
    );

    let oracle_with_artifact_pattern = Expr::Oracle {
        is_match: true,
        conditionals: vec![ConditionalAssignment {
            variable: "__match_0".into(),
            expression: Box::new(var("hero")),
            span: None,
        }],
        branches: vec![
            // Shorthand `Player { name }` (one binding, sub-pattern is
            // `Var(name)` matching the field name).
            OracleBranch {
                pattern: vec![Pattern::Artifact {
                    type_name: "Player".into(),
                    fields: vec![("name".into(), Pattern::Expr(var("name")))],
                    span: None,
                }],
                guard: None,
                body: reveal(var("name")),
                span: None,
            },
            // Explicit field with literal compare and a trailing binding.
            OracleBranch {
                pattern: vec![Pattern::Artifact {
                    type_name: "Player".into(),
                    fields: vec![
                        ("name".into(), Pattern::Expr(rune("Ardyn"))),
                        ("health".into(), Pattern::Expr(var("health"))),
                    ],
                    span: None,
                }],
                guard: None,
                body: reveal(var("health")),
                span: None,
            },
        ],
        span: None,
    };
    let oracle_with_artifact_pattern_expected = concat!(
        "oracle (hero) {\n",
        "    Player { name } => reveal name;\n",
        "    Player { name: \"Ardyn\", health } => reveal health;\n",
        "}"
    );
    assert_eq!(
        format_expr(&oracle_with_artifact_pattern, 0),
        oracle_with_artifact_pattern_expected
    );

    let oracle_with_lexicon_pattern = Expr::Oracle {
        is_match: true,
        conditionals: vec![ConditionalAssignment {
            variable: "__match_0".into(),
            expression: Box::new(var("config")),
            span: None,
        }],
        branches: vec![
            // Empty `{}` — matches any lexicon.
            OracleBranch {
                pattern: vec![Pattern::Lexicon {
                    entries: vec![],
                    span: None,
                }],
                guard: None,
                body: reveal(rune("a lexicon")),
                span: None,
            },
            // Two-key shape with a binding and a literal compare.
            OracleBranch {
                pattern: vec![Pattern::Lexicon {
                    entries: vec![
                        ("name".into(), Pattern::Expr(var("n"))),
                        ("port".into(), Pattern::Expr(Expr::Arcana(8080, None))),
                    ],
                    span: None,
                }],
                guard: None,
                body: reveal(var("n")),
                span: None,
            },
        ],
        span: None,
    };
    let oracle_with_lexicon_pattern_expected = concat!(
        "oracle (config) {\n",
        "    {} => reveal \"a lexicon\";\n",
        "    { \"name\": n, \"port\": 8080 } => reveal n;\n",
        "}"
    );
    assert_eq!(
        format_expr(&oracle_with_lexicon_pattern, 0),
        oracle_with_lexicon_pattern_expected
    );

    let block_stmt = Stmt::Block(vec![reveal(var("sigil"))], None);
    let orbit = Stmt::Orbit {
        params: vec![OrbitParam {
            name: "i".into(),
            start: *arcana(0),
            end: *arcana(2),
            op: "..".into(),
            span: None,
        }],
        body: Box::new(block_stmt.clone()),
        span: None,
    };
    let orbit_expected = concat!("orbit (i = 0..2)", "{\n    reveal sigil;\n};");
    assert_eq!(format_stmt(&orbit, 0), orbit_expected);

    let resume_named = Stmt::Revolve(Some("outer".into()), None);
    assert_eq!(format_stmt(&resume_named, 0), "revolve outer;");
    let resume_default = Stmt::Revolve(None, None);
    assert_eq!(format_stmt(&resume_default, 0), "revolve;");

    let eject_named = Stmt::Eject(Some("inner".into()), None);
    assert_eq!(format_stmt(&eject_named, 0), "eject inner;");
    let eject_default = Stmt::Eject(None, None);
    assert_eq!(format_stmt(&eject_default, 0), "eject;");

    let method = Stmt::Engrave {
        name: "ignite".into(),
        params: vec![
            EngraveParam {
                name: "core".into(),
                param_type: Type::Artifact("Pyre".into()),
                is_morph: true,
                span: None,
            },
            EngraveParam {
                name: "ember".into(),
                param_type: Type::Arcana,
                is_morph: false,
                span: None,
            },
        ],
        return_type: Type::Arcana,
        body: Box::new(block_stmt.clone()),
        method_target: Some(ArtifactMethodTarget {
            artifact: "Pyre".into(),
            requires_morph: true,
        }),
        span: None,
    };
    let method_expected = concat!(
        "engrave Pyre::ignite(morph core, ember: arcana) -> arcana ",
        "{\n    reveal sigil;\n};"
    );
    assert_eq!(format_stmt(&method, 0), method_expected);

    let function = Stmt::Engrave {
        name: "summon".into(),
        params: vec![EngraveParam {
            name: "target".into(),
            param_type: Type::Scroll,
            is_morph: false,
            span: None,
        }],
        return_type: Type::Abyss,
        body: Box::new(block_stmt),
        method_target: None,
        span: None,
    };
    let function_expected = concat!(
        "engrave summon(target: scroll) ",
        "{\n    reveal sigil;\n};"
    );
    assert_eq!(format_stmt(&function, 0), function_expected);
}

#[test]
fn format_pattern_shapes() {
    assert_eq!(format_pattern(&Pattern::DontCare(None), 0), "_");
    assert_eq!(
        format_pattern(
            &Pattern::Rest {
                name: None,
                span: None
            },
            0
        ),
        ".."
    );
    assert_eq!(
        format_pattern(
            &Pattern::Rest {
                name: Some("tail".into()),
                span: None
            },
            0
        ),
        "..tail"
    );
    assert_eq!(
        format_pattern(
            &Pattern::Artifact {
                type_name: "Tag".into(),
                fields: vec![],
                span: None
            },
            0
        ),
        "Tag {}"
    );
}

#[test]
fn format_type_keyword_variants() {
    let variants = vec![
        (Type::Arcana, "arcana".to_string()),
        (Type::Aether, "aether".to_string()),
        (Type::Rune, "rune".to_string()),
        (Type::Omen, "omen".to_string()),
        (Type::Abyss, "abyss".to_string()),
        (Type::Scroll, "scroll".to_string()),
        (Type::Lexicon, "lexicon".to_string()),
        (Type::Materia, "materia".to_string()),
        (Type::Glyph, "glyph".to_string()),
        (Type::Artifact("Relic".into()), "Relic".to_string()),
    ];

    for (index, (ty, expected)) in variants.into_iter().enumerate() {
        let is_morph = index % 2 == 1;
        let stmt = Stmt::VarAssign {
            name: format!("param{}", index),
            value: *arcana(1),
            var_type: ty,
            is_morph,
            span: None,
        };
        let prefix = if is_morph { "morph " } else { "" };
        let expected_text = format!("forge {}param{}: {} = 1;", prefix, index, expected);
        assert_eq!(format_stmt(&stmt, 0), expected_text);
    }
}

fn format_source(source: &str) -> String {
    let outcome = parse(source);
    assert!(
        outcome.diagnostics.is_empty(),
        "parser emitted diagnostics: {:?}",
        outcome.diagnostics
    );
    format_program(source, &outcome.ast, &collect_comments(source))
}

#[test]
fn format_program_preserves_comments() {
    let source = concat!(
        "// pick a sigil\n",
        "forge x:arcana=1; // initial value\n",
        "/* block comment\n   spanning lines */\n",
        "engrave double(n:arcana)->arcana {\n",
        "// twice the input\n",
        "reveal n*2; // fast path\n",
        "};\n",
        "unveil(double(x));\n",
        "// done\n",
    );
    let expected = concat!(
        "// pick a sigil\n",
        "forge x: arcana = 1; // initial value\n",
        "/* block comment\n",
        "   spanning lines */\n",
        "engrave double(n: arcana) -> arcana {\n",
        "    // twice the input\n",
        "    reveal n * 2; // fast path\n",
        "};\n",
        "unveil(double(x));\n",
        "// done\n",
    );
    assert_eq!(format_source(source), expected);
}

#[test]
fn format_program_with_comments_is_idempotent() {
    let source = concat!(
        "// leading\n",
        "forge x: arcana = 1; // trailing\n",
        "orbit (i = 0..2){\n",
        "    // loop body note\n",
        "    unveil(i);\n",
        "};\n",
        "// end\n",
    );
    let once = format_source(source);
    let twice = format_source(&once);
    assert_eq!(once, twice, "formatting must be idempotent");
}

#[test]
fn format_program_without_comments_matches_per_statement_output() {
    let source = "forge x: arcana = 1;\nunveil(x);\n";
    let outcome = parse(source);
    assert!(outcome.diagnostics.is_empty());
    let joined: String = outcome
        .ast
        .iter()
        .map(|stmt| format!("{}\n", format_stmt(stmt, 0)))
        .collect();
    assert_eq!(format_program(source, &outcome.ast, &[]), joined);
}
