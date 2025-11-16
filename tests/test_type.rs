mod test_base;

use abyss_lang::eval::{EvalError, EvalResult};
use test_base::{Value, test_base};

#[test]
fn test_cast_arcana_to_aether() {
    let input = "42.trans(aether);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 42.0),
                _ => panic!("Expected an Aether result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_aether_to_arcana() {
    let input = "3.14.trans(arcana);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 3),
                _ => panic!("Expected an Arcana result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_rune_to_aether() {
    let input = "\"3.14\".trans(aether);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => {
                    let expected = "3.14"
                        .parse::<f64>()
                        .expect("literal conversion should succeed");
                    assert_eq!(*n, expected);
                }
                _ => panic!("Expected an Aether result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_rune_to_arcana() {
    let input = "\"123\".trans(arcana);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 123),
                _ => panic!("Expected an Arcana result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_arcana_to_rune() {
    let input = "123.trans(rune);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "123"),
                _ => panic!("Expected a Rune result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_aether_to_rune() {
    let input = "3.14.trans(rune);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Rune(s)) => assert_eq!(s.as_ref(), "3.14"),
                _ => panic!("Expected a Rune result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn trans_requires_glyph_argument() {
    let input = "42.trans(42);";
    match test_base(input) {
        Ok(_) => panic!("expected glyph validation error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(message, _)) => {
                assert!(
                    message.contains("glyph value"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected invalid operation error, found {other:?}"),
        },
    }
}
