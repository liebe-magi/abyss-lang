mod common;

use abyss_interpreter::env::Value;
use abyss_interpreter::eval::{EvalError, EvalResult};
use common::test_base;

#[test]
fn test_parse_arcana_simple() {
    let input = "123;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 123),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_parse_arcana_leading_zeros() {
    let input = "00123;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 123),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_parse_arcana_with_whitespace() {
    let input = " 123;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 123),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_addition_simple() {
    let input = "1+2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 1 + 2),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_addition_larger_numbers() {
    let input = "123 + 456;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 123 + 456),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_addition_multiple_terms() {
    let input = "123 + 456 + 789;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 123 + 456 + 789),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_subtraction_simple() {
    let input = "10 - 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 - 3),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_subtraction_multiple_terms() {
    let input = "10 - 3 - 2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 - 3 - 2),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_multiplication_simple() {
    let input = "6 * 7;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 6 * 7),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_multiplication_multiple_terms() {
    let input = "6 * 7 * 2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 6 * 7 * 2),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_division_simple() {
    let input = "20 / 5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 20 / 5),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_division_multiple_terms() {
    let input = "20 / 5 / 2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 20 / 5 / 2),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_combined_operations_simple() {
    let input = "10 + 20 * 3 - 5 / 5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 + 20 * 3 - 5 / 5),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_combined_operations_with_parentheses_1() {
    let input = "(10 + 20) * 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, (10 + 20) * 3),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_combined_operations_with_parentheses_2() {
    let input = "10 * (20 + 3) / 5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 * (20 + 3) / 5),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_combined_operations_complex_1() {
    let input = "(10 + 20) / (5 - 3);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, (10 + 20) / (5 - 3)),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_combined_operations_complex_2() {
    let input = "(1 + 2) / 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, (1 + 2) / 3),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_combined_operations_complex_3() {
    let input = "10 / (2 + 3);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 / (2 + 3)),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_assignment() {
    let input = "forge x: arcana = 42;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            assert!(matches!(&results[0], EvalResult::Data(Value::Abyss)));
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_variable_usage() {
    let input = "forge x: arcana = 10; x + 5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 2);
            match &results[1] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 + 5),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_multiline_variable_usage() {
    let input = "
        forge x: arcana = 10;
        forge y: arcana = 3;
        3 * x + 2 * y;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 3 * 10 + 2 * 3),
                _ => panic!("Expected a number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_exponentiation_simple() {
    let input = "2 ^ 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 2_i64.pow(3)),
                _ => panic!("Expected an arcana (integer) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_exponentiation_zero_exponent() {
    let input = "5 ^ 0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 5_i64.pow(0)), // 結果は常に1
                _ => panic!("Expected an arcana (integer) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_exponentiation_with_parentheses() {
    let input = "(2 + 1) ^ 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, (2_i64 + 1_i64).pow(3)),
                _ => panic!("Expected an arcana (integer) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_exponentiation_nested() {
    let input = "3 ^ 4 ^ 2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 3_i64.pow(4).pow(2)),
                _ => panic!("Expected an arcana (integer) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_exponentiation_negative_exponent_error() {
    let input = "4 ^ -2;";
    match test_base(input) {
        Err(e) => match e.downcast_ref::<EvalError>() {
            Some(EvalError::NegativeExponent(_line_info)) => {}
            _ => panic!("Expected a negative exponent error"),
        },
        Ok(_) => panic!("Expected an error for negative exponent with arcana"),
    }
}

#[test]
fn test_parse_aether_simple() {
    let input = "123.45;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 123.45),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_parse_aether_leading_zeros() {
    let input = "00123.45;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 123.45),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_parse_aether_with_whitespace() {
    let input = " 123.45;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 123.45),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_addition_simple() {
    let input = "1.1 + 2.2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 1.1 + 2.2),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_addition_larger_numbers() {
    let input = "123.45 + 456.78;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 123.45 + 456.78),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_addition_multiple_terms() {
    let input = "123.45 + 456.78 + 789.01;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 123.45 + 456.78 + 789.01),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_subtraction_simple() {
    let input = "10.5 - 3.2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 - 3.2),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_subtraction_multiple_terms() {
    let input = "10.5 - 3.2 - 2.1;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 - 3.2 - 2.1),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_multiplication_simple() {
    let input = "6.5 * 7.2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 6.5 * 7.2),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_multiplication_multiple_terms() {
    let input = "6.5 * 7.2 * 2.1;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 6.5 * 7.2 * 2.1),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_division_simple() {
    let input = "20.5 / 5.1;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 20.5 / 5.1),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_division_multiple_terms() {
    let input = "20.5 / 5.1 / 2.2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 20.5 / 5.1 / 2.2),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_combined_operations_simple() {
    let input = "10.5 + 20.2 * 3.1 - 5.5 / 5.0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 + 20.2 * 3.1 - 5.5 / 5.0),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_combined_operations_with_parentheses_1() {
    let input = "(10.5 + 20.2) * 3.1;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, (10.5 + 20.2) * 3.1),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_combined_operations_with_parentheses_2() {
    let input = "10.5 * (20.2 + 3.3) / 5.5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 * (20.2 + 3.3) / 5.5),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_combined_operations_complex_1() {
    let input = "(10.5 + 20.2) / (5.5 - 3.1);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, (10.5 + 20.2) / (5.5 - 3.1)),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_combined_operations_complex_2() {
    let input = "(1.5 + 2.5) / 3.0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, (1.5 + 2.5) / 3.0),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_combined_operations_complex_3() {
    let input = "10.5 / (2.5 + 3.5);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 / (2.5 + 3.5)),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_assignment() {
    let input = "forge x: aether = 42.5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            assert!(matches!(&results[0], EvalResult::Data(Value::Abyss)));
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_variable_usage() {
    let input = "forge x: aether = 10.5; x + 5.5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 2);
            match &results[1] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 + 5.5),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_multiline_variable_usage() {
    let input = "
        forge x: aether = 10.5;
        forge y: aether = 3.5;
        3.0 * x + 2.0 * y;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 3.0 * 10.5 + 2.0 * 3.5),
                _ => panic!("Expected a floating point number result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_exponentiation_simple() {
    let input = "2.5 ** 3.0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 2.5_f64.powf(3.0)),
                _ => panic!("Expected an aether (floating point) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_exponentiation_zero_exponent() {
    let input = "5.0 ** 0.0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 5.0_f64.powf(0.0)), // 結果は常に1.0
                _ => panic!("Expected an aether (floating point) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_exponentiation_with_parentheses() {
    let input = "(1.5 + 2.5) ** 2.0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, (1.5_f64 + 2.5_f64).powf(2.0)),
                _ => panic!("Expected an aether (floating point) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_exponentiation_negative_exponent() {
    let input = "4.2 ** -1.0;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert!((*n - 1.0 / 4.2).abs() < 1e-9), // 負の指数もサポート
                _ => panic!("Expected an aether (floating point) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_exponentiation_fractional_exponent() {
    let input = "9.0 ** 0.5;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 9.0_f64.powf(0.5)), // 平方根
                _ => panic!("Expected an aether (floating point) result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_modulus() {
    let input = "10 % 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 % 3),
                _ => panic!("Expected an Arcana result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_modulus_negative() {
    let input = "-10 % 3;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, -10 % 3),
                _ => panic!("Expected an Arcana result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_modulus_assignment() {
    let input = "
        forge morph x: arcana = 10;
        x %= 3;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 % 3),
                _ => panic!("Expected an Arcana result after modulus assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_modulus() {
    let input = "10.5 % 3.2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert!((*n - (10.5 % 3.2)).abs() < 1e-9),
                _ => panic!("Expected an Aether result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_modulus_negative() {
    let input = "-10.5 % 3.2;";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Data(Value::Aether(n)) => assert!((*n - (-10.5 % 3.2)).abs() < 1e-9),
                _ => panic!("Expected an Aether result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_modulus_assignment() {
    let input = "
        forge morph x: aether = 10.5;
        x %= 3.2;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert!((*n - (10.5 % 3.2)).abs() < 1e-9),
                _ => panic!("Expected an Aether result after modulus assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_add_assign() {
    let input = "
        forge morph x: arcana = 10;
        x += 5;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 + 5),
                _ => panic!("Expected an Arcana result after add assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_sub_assign() {
    let input = "
        forge morph x: arcana = 10;
        x -= 3;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 - 3),
                _ => panic!("Expected an Arcana result after sub assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_mul_assign() {
    let input = "
        forge morph x: arcana = 10;
        x *= 2;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 * 2),
                _ => panic!("Expected an Arcana result after mul assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_div_assign() {
    let input = "
        forge morph x: arcana = 10;
        x /= 2;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 / 2),
                _ => panic!("Expected an Arcana result after div assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_mod_assign() {
    let input = "
        forge morph x: arcana = 10;
        x %= 3;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 10 % 3),
                _ => panic!("Expected an Arcana result after mod assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arcana_pow_assign() {
    let input = "
        forge morph x: arcana = 2;
        x ^= 3;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Arcana(n)) => assert_eq!(*n, 2_i64.pow(3)),
                _ => panic!("Expected an Arcana result after pow assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_add_assign() {
    let input = "
        forge morph x: aether = 10.5;
        x += 5.5;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 + 5.5),
                _ => panic!("Expected an Aether result after add assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_sub_assign() {
    let input = "
        forge morph x: aether = 10.5;
        x -= 3.0;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 - 3.0),
                _ => panic!("Expected an Aether result after sub assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_mul_assign() {
    let input = "
        forge morph x: aether = 10.5;
        x *= 2.0;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 * 2.0),
                _ => panic!("Expected an Aether result after mul assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_div_assign() {
    let input = "
        forge morph x: aether = 10.5;
        x /= 2.0;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 / 2.0),
                _ => panic!("Expected an Aether result after div assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_mod_assign() {
    let input = "
        forge morph x: aether = 10.5;
        x %= 3.0;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 10.5 % 3.0),
                _ => panic!("Expected an Aether result after mod assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_aether_pow_assign() {
    let input = "
        forge morph x: aether = 2.0;
        x **= 3.0;
        x;
    ";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 3);
            match &results[2] {
                EvalResult::Data(Value::Aether(n)) => assert_eq!(*n, 2.0_f64.powf(3.0)),
                _ => panic!("Expected an Aether result after pow assignment"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}
