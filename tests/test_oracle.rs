mod test_base;

use abyss_lang::eval::EvalResult;
use test_base::{Value, test_base};

#[test]
fn test_oracle_simple_positive() {
    let input = r#"
    forge x: arcana = 1;
    oracle {
        (x > 0) => "x is positive";
        (x < 0) => "x is negative";
        _ => reveal("x is zero");
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[1], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "x is positive")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_simple_zero() {
    let input = r#"
    forge x: arcana = 0;
    oracle {
        (x > 0) => "x is positive";
        (x < 0) => "x is negative";
        _ => "x is zero";
    };
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(results[1], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "x is zero")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_omen_hex() {
    let input = r#"
    forge x: arcana = -1;
    oracle (x > 0) {
        (boon) => reveal("x is positive");
        (hex) => reveal("x is negative or zero");
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[1], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "x is negative or zero")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_computation() {
    let input = r#"
    forge x: arcana = 11;
    oracle (y = x ^ 2) {
        (y > 100) => "y is greater than 100";
        (y == 100) => "y is equal to 100";
        _ => "y is less than 100";
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[1], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "y is greater than 100")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_string_comparison() {
    let input = r#"
    forge a: rune = "abyss";
    oracle (a = "abyss") {
        (a == "abyss") => "a is abyss";
        _ => "a is not abyss";
    };
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(results[1], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "a is abyss")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_multiple_conditions_1() {
    let input = r#"
    forge a: arcana = 3;
    forge b: arcana = 2;
    oracle (a, b) {
        (1, 2) => reveal("a is 1 and b is 2");
        (_, 2) => reveal("a is not 1 and b is 2");
        (1, _) => reveal("a is 1 and b is not 2");
        _ => reveal("a is not 1 and b is not 2");
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[2], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "a is not 1 and b is 2")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_multiple_conditions_2() {
    let input = r#"
    oracle (a = 1, b = 3) {
        (a == 1 && b == 2) => reveal("a is 1 and b is 2");
        (a != 1 && b == 2) => reveal("a is not 1 and b is 2");
        (a == 1 && b != 2) => reveal("a is 1 and b is not 2");
        _ => reveal("a is not 1 and b is not 2");
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[0], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "a is 1 and b is not 2")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_block_and_reveal() {
    let input = r#"
    forge x: arcana = -10;
    forge y: arcana = oracle (x > 0) {
        (boon) => reveal(x);
        (hex) => {
            forge z: arcana = x + 5;
            oracle (z > 0) {
                (boon) => reveal(x + 5);
                (hex) => reveal(x - 5);
            };
        }
    };
    y;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[2], EvalResult::Data(Value::Arcana(-15)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}
