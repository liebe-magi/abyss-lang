mod common;

use abyss_interpreter::env::Value;
use abyss_interpreter::eval::{EvalError, EvalResult};
use common::test_base;

#[test]
fn tally_handles_scroll_and_lexicon() {
    let script = r#"
        forge pack: scroll = [1, 2, 3];
        forge ledger: lexicon = {"alpha": 1, "beta": 2};
        pack.tally();
        ledger.tally();
    "#;

    let results = test_base(script).expect("execution failed");
    assert!(results.len() >= 4, "expected at least four statements");

    match &results[2] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 3),
        other => panic!("expected arcana from pack.tally(), got {other:?}"),
    }

    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 2),
        other => panic!("expected arcana from ledger.tally(), got {other:?}"),
    }
}

#[test]
fn scribe_and_extract_mutate_scroll() {
    let script = r#"
        forge morph pack: scroll = [1];
        pack.scribe(2);
        pack.scribe(3);
        pack.tally();
        pack.extract();
        pack.tally();
    "#;

    let results = test_base(script).expect("execution failed");
    assert!(results.len() >= 6, "expected at least six statements");

    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 3),
        other => panic!("expected arcana from first tally, got {other:?}"),
    }

    match &results[4] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 3),
        other => panic!("expected arcana from extract result, got {other:?}"),
    }

    match &results[5] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 2),
        other => panic!("expected arcana from second tally, got {other:?}"),
    }
}

#[test]
fn lexicon_methods_update_contents() {
    let script = r#"
        forge morph ledger: lexicon = {"alpha": 1, "beta": 2};
        ledger.expunge("alpha");
        ledger.glossary();
        ledger.tally();
    "#;

    let results = test_base(script).expect("execution failed");
    assert!(results.len() >= 4, "expected at least four statements");

    match &results[2] {
        EvalResult::Data(Value::Scroll(items)) => {
            let borrowed = items.borrow();
            assert_eq!(borrowed.len(), 1, "expected one rune key after expunge");
            match &borrowed[0] {
                Value::Rune(key) => assert_eq!(key.as_ref(), "beta"),
                other => panic!("expected rune key, got {other:?}"),
            }
        }
        other => panic!("expected scroll result from glossary, got {other:?}"),
    }

    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 1),
        other => panic!("expected arcana from tally after expunge, got {other:?}"),
    }
}

#[test]
fn define_handles_insert_and_update() {
    let script = r#"
        forge morph ledger: lexicon = {"alpha": 1};
        ledger.define("beta", 2);
        ledger.define("alpha", 3);
        ledger.glossary();
        ledger.tally();
    "#;

    let results = test_base(script).expect("execution failed");
    assert!(results.len() >= 5, "expected at least five statements");

    match &results[3] {
        EvalResult::Data(Value::Scroll(items)) => {
            let borrowed = items.borrow();
            assert_eq!(
                borrowed.len(),
                2,
                "expected two rune keys after define calls"
            );
            let mut seen = borrowed
                .iter()
                .map(|value| match value {
                    Value::Rune(text) => text.as_ref().clone(),
                    other => panic!("expected rune key, got {other:?}"),
                })
                .collect::<std::collections::HashSet<_>>();
            assert!(seen.remove("alpha"));
            assert!(seen.remove("beta"));
            assert!(seen.is_empty());
        }
        other => panic!("expected scroll result from glossary, got {other:?}"),
    }

    match &results[4] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 2),
        other => panic!("expected arcana from tally after define, got {other:?}"),
    }
}

#[test]
fn scribe_requires_morph_scroll() {
    let script = r#"
        forge pack: scroll = [1];
        pack.scribe(2);
    "#;

    match test_base(script) {
        Ok(_) => panic!("expected immutability error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(message, _)) => {
                assert!(
                    message.contains("scroll::scribe"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected invalid operation error, found {other:?}"),
        },
    }
}

#[test]
fn expunge_requires_morph_lexicon() {
    let script = r#"
        forge ledger: lexicon = {"alpha": 1};
        ledger.expunge("alpha");
    "#;

    match test_base(script) {
        Ok(_) => panic!("expected immutability error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(message, _)) => {
                assert!(
                    message.contains("lexicon::expunge"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected invalid operation error, found {other:?}"),
        },
    }
}

#[test]
fn define_requires_rune_key() {
    let script = r#"
        forge morph ledger: lexicon = {"alpha": 1};
        ledger.define(1, 2);
    "#;

    match test_base(script) {
        Ok(_) => panic!("expected key type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::TypeError(message, _)) => {
                assert!(
                    message.contains("key must be a rune"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected type error, found {other:?}"),
        },
    }
}

#[test]
fn scroll_sort_unique_sum_min_max() {
    let input = r#"
forge xs: scroll = [3, 1, 2, 3, 1];
xs.sort();
xs.unique();
xs.sum();
xs.min();
xs.max();
forge names: scroll = ["hex", "boon", "abyss"];
names.sort();
"#;
    let results = test_base(input).expect("scroll methods should evaluate");
    assert_eq!(results.len(), 8);

    let as_arcana_vec = |result: &EvalResult| -> Vec<i64> {
        match result {
            EvalResult::Data(Value::Scroll(items)) => items
                .borrow()
                .iter()
                .map(|v| match v {
                    Value::Arcana(n) => *n,
                    other => panic!("expected arcana element, got {:?}", other),
                })
                .collect(),
            other => panic!("expected scroll, got {:?}", other),
        }
    };

    assert_eq!(as_arcana_vec(&results[1]), vec![1, 1, 2, 3, 3]);
    assert_eq!(as_arcana_vec(&results[2]), vec![3, 1, 2]);
    assert!(matches!(&results[3], EvalResult::Data(Value::Arcana(10))));
    assert!(matches!(&results[4], EvalResult::Data(Value::Arcana(1))));
    assert!(matches!(&results[5], EvalResult::Data(Value::Arcana(3))));
    match &results[7] {
        EvalResult::Data(Value::Scroll(items)) => {
            let names: Vec<String> = items
                .borrow()
                .iter()
                .map(|v| match v {
                    Value::Rune(s) => s.as_ref().clone(),
                    other => panic!("expected rune element, got {:?}", other),
                })
                .collect();
            assert_eq!(names, vec!["abyss", "boon", "hex"]);
        }
        other => panic!("expected scroll, got {:?}", other),
    }
}

#[test]
fn scroll_sort_leaves_receiver_untouched() {
    let input = r#"
forge xs: scroll = [2, 1];
xs.sort();
xs[0];
"#;
    let results = test_base(input).expect("sort should not mutate");
    assert!(matches!(&results[2], EvalResult::Data(Value::Arcana(2))));
}

#[test]
fn scroll_aggregates_reject_empty_and_mixed() {
    let empty = r#"
forge xs: scroll = [];
xs.min();
"#;
    match test_base(empty) {
        Ok(_) => panic!("expected empty-scroll error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(msg.contains("empty scroll"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        },
    }

    let mixed = r#"
forge xs: scroll = [1, "two"];
xs.sum();
"#;
    match test_base(mixed) {
        Ok(_) => panic!("expected mixed-type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(msg.contains("share one type"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        },
    }
}
