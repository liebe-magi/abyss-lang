mod common;

use abyss_interpreter::env::Value;
use abyss_interpreter::eval::{EvalError, EvalResult};
use common::test_base;

fn expect_artifact_name(result: &EvalResult, expected: &str) {
    match result {
        EvalResult::Data(Value::Artifact(handle)) => {
            assert_eq!(handle.borrow().type_name, expected);
        }
        other => panic!("expected {} artifact, got {:?}", expected, other),
    }
}

#[test]
fn fate_variants_construct_and_annotate() {
    let input = r#"
forge ok: fate = bless { value: 42 };
forge bad: fate = curse { reason: "misfired" };
ok;
bad;
"#;
    let results = test_base(input).expect("fate construction should evaluate");
    expect_artifact_name(&results[2], "bless");
    expect_artifact_name(&results[3], "curse");
}

#[test]
fn augury_variants_construct_and_annotate() {
    let input = r#"
forge found: augury = manifest { value: "sigil" };
forge missing: augury = naught {};
found;
missing;
"#;
    let results = test_base(input).expect("augury construction should evaluate");
    expect_artifact_name(&results[2], "manifest");
    expect_artifact_name(&results[3], "naught");
}

#[test]
fn fate_destructures_with_artifact_patterns() {
    let input = r#"
forge result: fate = bless { value: 7 };
oracle (result) {
    bless { value } => value;
    curse { reason } => 0;
};
"#;
    let results = test_base(input).expect("fate pattern match should evaluate");
    match &results[1] {
        EvalResult::Data(Value::Arcana(7)) => {}
        other => panic!("expected unwrapped 7, got {:?}", other),
    }
}

#[test]
fn fate_annotation_rejects_other_values() {
    let input = r#"forge r: fate = 42;"#;
    match test_base(input) {
        Ok(_) => panic!("expected type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::ExpectedType(ty, _)) => {
                assert_eq!(format!("{:?}", ty), "Fate");
            }
            other => panic!("expected ExpectedType, got {:?}", other),
        },
    }
}

#[test]
fn fate_annotation_rejects_augury_variants() {
    let input = r#"forge r: fate = naught {};"#;
    match test_base(input) {
        Ok(_) => panic!("expected type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::ExpectedType(_, _)) => {}
            other => panic!("expected ExpectedType, got {:?}", other),
        },
    }
}

#[test]
fn engrave_fate_return_enforced() {
    let ok = r#"
engrave parse_level(raw: arcana) -> fate {
    oracle {
        (raw >= 0) => reveal bless { value: raw };
        _ => reveal curse { reason: "negative" };
    };
};
parse_level(5);
parse_level(0 - 1);
"#;
    let results = test_base(ok).expect("fate-returning function should work");
    expect_artifact_name(&results[1], "bless");
    expect_artifact_name(&results[2], "curse");

    let bad = r#"
artifact Player { name: rune; };
engrave broken() -> fate {
    reveal Player { name: "Ardyn" };
};
broken();
"#;
    match test_base(bad) {
        Ok(_) => panic!("expected return-type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::TypeError(msg, _)) => {
                assert!(
                    msg.contains("expected fate, got artifact Player"),
                    "msg: {msg}"
                );
            }
            other => panic!("expected type error, got {:?}", other),
        },
    }
}

#[test]
fn morph_fate_reassignment_works() {
    let input = r#"
forge morph r: fate = bless { value: 1 };
r = curse { reason: "flipped" };
r;
"#;
    let results = test_base(input).expect("fate reassignment should work");
    expect_artifact_name(&results[2], "curse");
}

#[test]
fn reserved_variant_names_cannot_be_redefined() {
    let input = r#"artifact bless { value: arcana; };"#;
    match test_base(input) {
        Ok(_) => panic!("expected reserved-name error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(msg.contains("reserved"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        },
    }

    let inner = r#"
engrave shadow() -> abyss {
    artifact naught { ghost: arcana; };
};
shadow();
"#;
    match test_base(inner) {
        Ok(_) => panic!("expected reserved-name error in inner scope"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(msg.contains("reserved"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        },
    }
}
