use abyss_core::ast::{AST, ArtifactField, ArtifactMethodTarget, AssignmentOp, Type};
use abyss_core::format::format_ast;

fn arcana(value: i64) -> Box<AST> {
    Box::new(AST::Arcana(value, None))
}

fn rune(value: &str) -> Box<AST> {
    Box::new(AST::Rune(value.to_string(), None))
}

fn omen(value: bool) -> Box<AST> {
    Box::new(AST::Omen(value, None))
}

fn var(name: &str) -> Box<AST> {
    Box::new(AST::Var(name.to_string(), None))
}

fn abyss() -> Box<AST> {
    Box::new(AST::Abyss(None))
}

#[test]
fn format_basic_expressions_and_assignments() {
    let expr_stmt = AST::Statement(
        Box::new(AST::Mul(
            Box::new(AST::Add(arcana(1), arcana(2), None)),
            arcana(3),
            None,
        )),
        None,
    );
    assert_eq!(format_ast(&expr_stmt, 1), "    (1 + 2) * 3;");

    let logical = AST::LogicalNot(omen(false), None);
    assert_eq!(format_ast(&logical, 0), "!hex");

    let var_assign = AST::VarAssign {
        name: "sigil".into(),
        value: rune("alpha"),
        var_type: Type::Rune,
        is_morph: true,
        line_info: None,
    };
    assert_eq!(
        format_ast(&var_assign, 0),
        "forge morph sigil: rune = \"alpha\""
    );

    let assign = AST::Assignment {
        name: "sigil".into(),
        value: var("base"),
        op: AssignmentOp::PowAetherAssign,
        line_info: None,
    };
    assert_eq!(format_ast(&assign, 0), "sigil **= base");

    assert_eq!(format_ast(&AST::Arcana(7, None), 0), "7");
    assert_eq!(format_ast(&AST::Aether(7.0, None), 0), "7.0");
    assert_eq!(format_ast(&AST::Aether(7.25, None), 0), "7.25");
    assert_eq!(format_ast(&AST::Rune("echo".into(), None), 0), "\"echo\"");
    assert_eq!(format_ast(&AST::Omen(true, None), 0), "boon");
    assert_eq!(format_ast(&AST::Abyss(None), 0), "abyss");

    let reveal_core = AST::Reveal(var("sigil"), None);
    assert_eq!(format_ast(&reveal_core, 0), "reveal sigil");

    let reveal_abyss = AST::Reveal(abyss(), None);
    assert_eq!(format_ast(&reveal_abyss, 0), "reveal");

    let field_access = AST::FieldAccess {
        target: var("relic"),
        field: "core".into(),
        line_info: None,
    };
    assert_eq!(format_ast(&field_access, 0), "relic.core");

    let field_assignment = AST::FieldAssignment {
        target: var("relic"),
        field: "core".into(),
        value: arcana(5),
        line_info: None,
    };
    assert_eq!(format_ast(&field_assignment, 0), "relic.core = 5");

    let index_assignment = AST::IndexAssignment {
        target: var("sigils"),
        index: arcana(2),
        value: rune("beta"),
        line_info: None,
    };
    assert_eq!(format_ast(&index_assignment, 0), "sigils[2] = \"beta\"");

    let method_call = AST::MethodCall {
        receiver: var("relic"),
        method: "ignite".into(),
        args: vec![*rune("spark")],
        line_info: None,
    };
    assert_eq!(format_ast(&method_call, 0), "relic.ignite(\"spark\")");
}

#[test]
fn format_collections_and_literals() {
    let list = AST::ListLiteral {
        elements: vec![
            AST::Arcana(1, None),
            AST::Arcana(2, None),
            AST::Arcana(3, None),
        ],
        line_info: None,
    };
    assert_eq!(format_ast(&list, 0), "[1, 2, 3]");

    let map = AST::MapLiteral {
        entries: vec![
            ("name".into(), AST::Rune("abyss".into(), None)),
            ("value".into(), AST::Arcana(3, None)),
        ],
        line_info: None,
    };
    assert_eq!(format_ast(&map, 0), "{\"name\": \"abyss\", \"value\": 3}");

    let artifact_literal = AST::ArtifactLiteral {
        type_name: "Relic".into(),
        fields: vec![
            ("sigil".into(), AST::Rune("alpha".into(), None)),
            ("power".into(), AST::Arcana(9, None)),
        ],
        line_info: None,
    };
    assert_eq!(
        format_ast(&artifact_literal, 0),
        "Relic { sigil: \"alpha\", power: 9 }"
    );

    let artifact_empty = AST::ArtifactLiteral {
        type_name: "Relic".into(),
        fields: vec![],
        line_info: None,
    };
    assert_eq!(format_ast(&artifact_empty, 0), "Relic {}");

    let artifact_def = AST::ArtifactDef {
        name: "Relic".into(),
        fields: vec![
            ArtifactField {
                name: "sigil".into(),
                field_type: Type::Rune,
                line_info: None,
            },
            ArtifactField {
                name: "power".into(),
                field_type: Type::Arcana,
                line_info: None,
            },
        ],
        line_info: None,
    };
    let expected = concat!(
        "artifact Relic {\n",
        "    sigil: rune;\n",
        "    power: arcana;\n",
        "}"
    );
    assert_eq!(format_ast(&artifact_def, 0), expected);

    let list_access = AST::IndexAccess {
        target: var("sigils"),
        index: arcana(0),
        line_info: None,
    };
    assert_eq!(format_ast(&list_access, 0), "sigils[0]");

    let func_call = AST::FuncCall {
        name: "summon".into(),
        args: vec![AST::Rune("echo".into(), None), AST::Arcana(1, None)],
        line_info: None,
    };
    assert_eq!(format_ast(&func_call, 0), "summon(\"echo\", 1)");
}

#[test]
fn format_control_flow_and_functions() {
    let block = AST::Block(
        vec![AST::Statement(
            Box::new(AST::Reveal(var("sigil"), None)),
            None,
        )],
        None,
    );
    assert_eq!(format_ast(&block, 0), "{\n    reveal sigil;\n}");

    let oracle = AST::Oracle {
        is_match: false,
        conditionals: Vec::new(),
        branches: vec![
            AST::OracleBranch {
                pattern: vec![AST::Arcana(1, None)],
                body: Box::new(AST::Reveal(var("spark"), None)),
                line_info: None,
            },
            AST::Comment("// fallback".into(), None),
            AST::OracleBranch {
                pattern: vec![AST::OracleDontCareItem(None)],
                body: Box::new(AST::Reveal(Box::new(AST::Rune("wild".into(), None)), None)),
                line_info: None,
            },
            AST::OracleBranch {
                pattern: vec![],
                body: Box::new(AST::Reveal(abyss(), None)),
                line_info: None,
            },
        ],
        line_info: None,
    };
    let oracle_expected = concat!(
        "oracle {\n",
        "    (1) => reveal spark\n",
        "    // fallback\n",
        "    (_) => reveal \"wild\"\n",
        "    _ => reveal\n",
        "}"
    );
    assert_eq!(format_ast(&oracle, 0), oracle_expected);

    let orbit = AST::Orbit {
        params: vec![AST::OrbitParam {
            name: "i".into(),
            start: arcana(0),
            end: arcana(2),
            op: "..".into(),
            line_info: None,
        }],
        body: Box::new(block.clone()),
        line_info: None,
    };
    let orbit_expected = concat!("orbit (i = 0..2)", "{\n    reveal sigil;\n}");
    assert_eq!(format_ast(&orbit, 0), orbit_expected);

    let resume_named = AST::Resume(Some("outer".into()), None);
    assert_eq!(format_ast(&resume_named, 0), "resume outer");
    let resume_default = AST::Resume(None, None);
    assert_eq!(format_ast(&resume_default, 0), "resume");

    let eject_named = AST::Eject(Some("inner".into()), None);
    assert_eq!(format_ast(&eject_named, 0), "eject inner");
    let eject_default = AST::Eject(None, None);
    assert_eq!(format_ast(&eject_default, 0), "eject");

    let method = AST::Engrave {
        name: "ignite".into(),
        params: vec![
            AST::EngraveParam {
                name: "core".into(),
                param_type: Type::Artifact("Pyre".into()),
                is_morph: true,
                line_info: None,
            },
            AST::EngraveParam {
                name: "ember".into(),
                param_type: Type::Arcana,
                is_morph: false,
                line_info: None,
            },
        ],
        return_type: Type::Arcana,
        body: Box::new(block.clone()),
        method_target: Some(ArtifactMethodTarget {
            artifact: "Pyre".into(),
            requires_morph: true,
        }),
        line_info: None,
    };
    let method_expected = concat!(
        "engrave Pyre::ignite(morph core, ember: arcana) -> arcana ",
        "{\n    reveal sigil;\n}"
    );
    assert_eq!(format_ast(&method, 0), method_expected);

    let function = AST::Engrave {
        name: "summon".into(),
        params: vec![AST::EngraveParam {
            name: "target".into(),
            param_type: Type::Scroll,
            is_morph: false,
            line_info: None,
        }],
        return_type: Type::Abyss,
        body: Box::new(block),
        method_target: None,
        line_info: None,
    };
    let function_expected = concat!("engrave summon(target: scroll) ", "{\n    reveal sigil;\n}");
    assert_eq!(format_ast(&function, 0), function_expected);
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
        let param = AST::EngraveParam {
            name: format!("param{}", index),
            param_type: ty,
            is_morph: index % 2 == 1,
            line_info: None,
        };
        let prefix = if index % 2 == 1 { "morph " } else { "" };
        let expected_text = format!("{}param{}: {}", prefix, index, expected);
        assert_eq!(format_ast(&param, 0), expected_text);
    }
}
