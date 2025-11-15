mod test_base;

use abyss_lang::eval::EvalResult;
use test_base::{Value, test_base};

#[test]
fn test_logical_not() {
    let input = "!boon;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(false)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_and_both_true() {
    let input = "boon && boon;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_and_one_false() {
    let input = "boon && hex;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(false)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_or_both_false() {
    let input = "hex || hex;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(false)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_or_one_true() {
    let input = "boon || hex;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_combination() {
    let input = "boon && (hex || boon);";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_with_arithmetic() {
    let input = "(5 + 3) == 8 && boon;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_logical_with_comparison() {
    let input = "5 > 3 && 10 <= 10 || hex;";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_nested_logical_operations() {
    let input = "!(boon && hex) || (5 + 5 == 10 && boon);";
    match test_base(input) {
        Ok(results) => assert!(matches!(results[0], EvalResult::Data(Value::Omen(true)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}
