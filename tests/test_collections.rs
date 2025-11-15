mod test_base;

use abyss_lang::eval::EvalResult;
use test_base::{Value, test_base};

#[test]
fn measure_handles_scroll_and_lexicon() {
    let script = r#"
        forge pack: scroll = [1, 2, 3];
        measure(pack);
        measure({"alpha": 1, "beta": 2});
    "#;

    let results = test_base(script).expect("execution failed");
    assert!(results.len() >= 3, "expected at least three statements");

    match &results[1] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 3),
        other => panic!("expected arcana from first measure, got {other:?}"),
    }

    match &results[2] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 2),
        other => panic!("expected arcana from second measure, got {other:?}"),
    }
}

#[test]
fn inscribe_and_retract_mutate_scroll() {
    let script = r#"
        forge morph pack: scroll = [1];
        inscribe(pack, 2);
        inscribe(pack, 3);
        measure(pack);
        retract(pack);
        measure(pack);
    "#;

    let results = test_base(script).expect("execution failed");
    assert!(results.len() >= 6, "expected at least six statements");

    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 3),
        other => panic!("expected arcana from first measure, got {other:?}"),
    }

    match &results[4] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 3),
        other => {
            panic!("expected arcana from value returned by retract (popped value), got {other:?}")
        }
    }

    match &results[5] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 2),
        other => panic!("expected arcana from second measure, got {other:?}"),
    }
}

#[test]
fn expunge_and_contents_update_lexicon() {
    let script = r#"
        forge morph ledger: lexicon = {"alpha": 1, "beta": 2};
        expunge(ledger, "alpha");
        contents(ledger);
        measure(ledger);
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
        other => panic!("expected scroll result from contents, got {other:?}"),
    }

    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 1),
        other => panic!("expected arcana from measure after expunge, got {other:?}"),
    }
}
