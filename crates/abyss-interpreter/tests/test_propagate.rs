mod common;

use abyss_interpreter::env::Value;
use abyss_interpreter::eval::{EvalError, EvalResult};
use common::test_base;

#[test]
fn question_unwraps_bless_and_manifest() {
    let input = r#"
engrave give() -> fate {
    reveal bless { value: 21 };
};
engrave peek() -> augury {
    reveal manifest { value: 2 };
};
engrave run() -> arcana {
    reveal give()? * peek()?;
};
run();
"#;
    let results = test_base(input).expect("? should unwrap success variants");
    match results.last().unwrap() {
        EvalResult::Data(Value::Arcana(42)) => {}
        other => panic!("expected 42, got {:?}", other),
    }
}

#[test]
fn question_propagates_curse_to_function_return() {
    let input = r#"
engrave fail() -> fate {
    reveal curse { reason: "misfired" };
};
engrave run() -> fate {
    forge x: arcana = fail()?;
    reveal bless { value: x };
};
oracle (run()) {
    bless { value } => "ok";
    curse { reason } => reason;
};
"#;
    let results = test_base(input).expect("propagated curse should be matchable");
    match results.last().unwrap() {
        EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "misfired"),
        other => panic!("expected reason rune, got {:?}", other),
    }
}

#[test]
fn question_propagates_out_of_nested_orbit_and_oracle() {
    let input = r#"
engrave find_gap(limit: arcana) -> augury {
    reveal naught {};
};
engrave scan() -> augury {
    orbit (i = 0..3) {
        oracle {
            (i == 1) => {
                forge hit: arcana = find_gap(i)?;
                unveil(hit);
            }
            _ => unveil(i);
        };
    };
    reveal manifest { value: 99 };
};
oracle (scan()) {
    manifest { value } => "found";
    naught {} => "nothing";
};
"#;
    let results = test_base(input).expect("propagation should unwind orbit and oracle");
    match results.last().unwrap() {
        EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "nothing"),
        other => panic!("expected naught branch, got {:?}", other),
    }
}

#[test]
fn cross_union_propagation_fails_return_check() {
    let input = r#"
engrave nothing() -> augury {
    reveal naught {};
};
engrave run() -> fate {
    forge x: arcana = nothing()?;
    reveal bless { value: x };
};
run();
"#;
    match test_base(input) {
        Ok(_) => panic!("expected return-type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::TypeError(msg, _)) => {
                assert!(
                    msg.contains("expected fate, got artifact naught"),
                    "msg: {msg}"
                );
            }
            other => panic!("expected type error, got {:?}", other),
        },
    }
}

#[test]
fn uncaught_curse_reaches_top_level_with_message() {
    let input = r#"
engrave fail() -> fate {
    reveal curse { reason: "the ritual collapsed" };
};
fail()?;
"#;
    match test_base(input) {
        Ok(_) => panic!("expected uncaught propagation"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(err @ EvalError::Propagation(_, span)) => {
                assert!(span.is_some(), "propagation should carry the ? span");
                assert_eq!(err.to_string(), "Uncaught curse: the ritual collapsed");
            }
            other => panic!("expected propagation, got {:?}", other),
        },
    }
}

#[test]
fn question_on_non_fate_value_errors() {
    let input = r#"forge x: arcana = 42?;"#;
    match test_base(input) {
        Ok(_) => panic!("expected type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(
                    msg.contains("? requires a fate or augury value, found arcana"),
                    "msg: {msg}"
                );
            }
            other => panic!("expected invalid operation, got {:?}", other),
        },
    }
}
