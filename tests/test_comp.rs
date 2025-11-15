mod test_base;

use abyss_lang::eval::{EvalError, EvalResult};
use test_base::{Value, test_base};

#[test]
fn test_arcana_comparison_equal() {
    let input = "5 == 5;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_comparison_not_equal() {
    let input = "5 != 3;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_comparison_less_than() {
    let input = "3 < 5;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_comparison_greater_than() {
    let input = "10 > 7;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_comparison_less_than_or_equal() {
    let input = "5 <= 5;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_comparison_greater_than_or_equal() {
    let input = "8 >= 7;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_comparison_equal() {
    let input = "3.14 == 3.14;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_comparison_not_equal() {
    let input = "3.14 != 2.71;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_comparison_less_than() {
    let input = "2.71 < 3.14;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_comparison_greater_than() {
    let input = "6.28 > 3.14;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_comparison_less_than_or_equal() {
    let input = "3.14 <= 3.14;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_comparison_greater_than_or_equal() {
    let input = "3.14 >= 2.71;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_rune_comparison_equal() {
    let input = "\"hello\" == \"hello\";";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_rune_comparison_not_equal() {
    let input = "\"hello\" != \"world\";";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_rune_comparison_less_than_should_error() {
    let input = "\"apple\" < \"banana\";";
    match test_base(input) {
        Err(e) => match e.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(_, _)) => {}
            _ => panic!("Expected an invalid operation error for < with Rune"),
        },
        Ok(_) => panic!("Expected an error for < operation with Rune"),
    }
}

#[test]
fn test_rune_comparison_greater_than_should_error() {
    let input = "\"banana\" > \"apple\";";
    match test_base(input) {
        Err(e) => match e.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(_, _)) => {}
            _ => panic!("Expected an invalid operation error for > with Rune"),
        },
        Ok(_) => panic!("Expected an error for > operation with Rune"),
    }
}

#[test]
fn test_rune_comparison_less_than_or_equal_should_error() {
    let input = "\"apple\" <= \"apple\";";
    match test_base(input) {
        Err(e) => match e.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(_, _)) => {}
            _ => panic!("Expected an invalid operation error for <= with Rune"),
        },
        Ok(_) => panic!("Expected an error for <= operation with Rune"),
    }
}

#[test]
fn test_rune_comparison_greater_than_or_equal_should_error() {
    let input = "\"banana\" >= \"apple\";";
    match test_base(input) {
        Err(e) => match e.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(_, _)) => {}
            _ => panic!("Expected an invalid operation error for >= with Rune"),
        },
        Ok(_) => panic!("Expected an error for >= operation with Rune"),
    }
}
