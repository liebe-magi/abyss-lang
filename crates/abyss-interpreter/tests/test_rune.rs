mod common;

use abyss_interpreter::env::Value;
use abyss_interpreter::eval::{EvalError, EvalResult};
use common::test_base;

#[test]
fn test_parse_rune() {
    let input = "\"Hello, Abyss!\";";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "Hello, Abyss!"),
                _ => panic!("Expected a string result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_evaluate_rune_assign() {
    let input = r#"forge message: rune = "Hello World from Abyss!"; message;"#;
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 2);
            match &results[1] {
                EvalResult::Data(Value::Rune(s)) => {
                    assert_eq!(s.as_ref(), "Hello World from Abyss!")
                }
                _ => panic!("Expected a string result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_rune_concatenation() {
    let input = r#"forge part1: rune = "Hello, "; forge part2: rune = "Abyss"; part1 + part2;"#;
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "Hello, Abyss"),
                _ => panic!("Expected a concatenated string result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_rune_concatenation_multiline() {
    let input = r#"
        forge part1: rune = "Hello, ";
        forge part2: rune = "Abyss";
        part1 + part2 + "!";
    "#;
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "Hello, Abyss!"),
                _ => panic!("Expected a concatenated string result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_unveil_rune_1() {
    let input = r#"unveil("Hello, Abyss!");"#;
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            assert!(matches!(&results[0], EvalResult::Data(Value::Abyss)));
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_unveil_rune_2() {
    let input = r#"
        forge part1: rune = "Hello, ";
        forge part2: rune = "Abyss";
        unveil(part1 + part2 + "!");
    "#;
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            assert!(matches!(&results[0], EvalResult::Data(Value::Abyss)));
            assert!(matches!(&results[1], EvalResult::Data(Value::Abyss)));
            assert!(matches!(&results[2], EvalResult::Data(Value::Abyss)));
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_unveil_rune_3() {
    let input = r#"
        unveil("1 + 3 = ", 1 + 3);
    "#;
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            assert!(matches!(&results[0], EvalResult::Data(Value::Abyss)));
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_trans_in_string_concatenation() {
    let input = r#"
    forge x: rune = "answer: " + 42.transmute(rune);
    x;
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[1], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "answer: 42")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_trans_in_arithmetic_expression() {
    let input = r#"
    forge y: arcana = "42".transmute(arcana) + 8;
    y;
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(matches!(results[1], EvalResult::Data(Value::Arcana(50))))
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_trans_with_assignment_operator() {
    let input = r#"
    forge morph z: rune = "answer: ";
    z += 42.transmute(rune);
    z;
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[2], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "answer: 42")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_unveil_no_args_error() {
    let input = r#"unveil();"#;
    match test_base(input) {
        Err(e) => match e.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(
                    msg.contains("unveil() requires at least 1 argument"),
                    "Expected error message about requiring at least 1 argument, got: {}",
                    msg
                );
            }
            _ => panic!("Expected an InvalidOperation error"),
        },
        Ok(_) => panic!("Expected an error for unveil() with no arguments"),
    }
}

#[test]
fn rune_methods_transform_and_query() {
    let input = r#"
forge spell: rune = "  Dark Incantation  ";
forge clean: rune = spell.trim();
clean.upper();
clean.lower();
clean.tally();
clean.contains("Incant");
clean.replace("Dark", "Bright");
"#;
    let results = test_base(input).expect("rune methods should evaluate");
    assert_eq!(results.len(), 7);
    match &results[2] {
        EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "DARK INCANTATION"),
        other => panic!("expected upper() rune, got {:?}", other),
    }
    match &results[3] {
        EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "dark incantation"),
        other => panic!("expected lower() rune, got {:?}", other),
    }
    match &results[4] {
        EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 16),
        other => panic!("expected tally() arcana, got {:?}", other),
    }
    match &results[5] {
        EvalResult::Data(Value::Omen(true)) => {}
        other => panic!("expected contains() boon, got {:?}", other),
    }
    match &results[6] {
        EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "Bright Incantation"),
        other => panic!("expected replace() rune, got {:?}", other),
    }
}

#[test]
fn rune_split_yields_scroll_of_runes() {
    let input = r#"
forge csv: rune = "boon,hex,abyss";
forge parts: scroll = csv.split(",");
parts.tally();
parts[1];
"#;
    let results = test_base(input).expect("split should evaluate");
    assert_eq!(results.len(), 4);
    match &results[2] {
        EvalResult::Data(Value::Arcana(3)) => {}
        other => panic!("expected 3 parts, got {:?}", other),
    }
    match &results[3] {
        EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "hex"),
        other => panic!("expected rune part, got {:?}", other),
    }
}

#[test]
fn rune_tally_counts_unicode_characters() {
    let input = r#""呪文詠唱".tally();"#;
    let results = test_base(input).expect("tally should evaluate");
    match &results[0] {
        EvalResult::Data(Value::Arcana(4)) => {}
        other => panic!("expected 4 characters, got {:?}", other),
    }
}

#[test]
fn rune_split_rejects_empty_separator() {
    let input = r#""abc".split("");"#;
    match test_base(input) {
        Ok(_) => panic!("expected split error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(msg, _)) => {
                assert!(msg.contains("non-empty separator"), "msg: {msg}");
            }
            other => panic!("expected invalid operation, got {:?}", other),
        },
    }
}
