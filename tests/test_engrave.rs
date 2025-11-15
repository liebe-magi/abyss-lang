mod test_base;

use abyss_lang::eval::EvalResult;
use test_base::{Value, test_base};

#[test]
fn test_simple_function() {
    let input = r#"
    engrave add(a: arcana, b: arcana) -> arcana {
        reveal a + b;
    };
    forge result: arcana = add(2, 3);
    result;
    "#;

    let result_rust = 2 + 3;

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(result_abyss)) = results[2] {
                assert_eq!(result_rust, result_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_function_with_strings() {
    let input = r#"
    engrave greet(name: rune) -> rune {
        reveal "Hello, " + name;
    };
    forge message: rune = greet("Abyss");
    message;
    "#;

    let message_rust = "Hello, Abyss".to_string();

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Rune(message_abyss)) = &results[2] {
                assert_eq!(message_rust.as_str(), message_abyss.as_ref());
            } else {
                panic!("Expected Rune result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_function_with_floats() {
    let input = r#"
    engrave multiply(a: aether, b: aether) -> aether {
        reveal a * b;
    };
    forge result: aether = multiply(2.5, 4.0);
    result;
    "#;

    let result_rust = 2.5 * 4.0;

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Aether(result_abyss)) = results[2] {
                assert_eq!(result_rust, result_abyss);
            } else {
                panic!("Expected Aether result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_function_with_boon_hex() {
    let input = r#"
    engrave is_even(num: arcana) -> omen {
        oracle(num % 2 == 0) {
            (boon) => reveal boon;
            (hex) => reveal hex;
        };
    };
    forge result: omen = is_even(4);
    result;
    "#;

    let result_rust = true;

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Omen(result_abyss)) = results[2] {
                assert_eq!(result_rust, result_abyss);
            } else {
                panic!("Expected Omen result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_nested_function_calls() {
    let input = r#"
    engrave add(a: arcana, b: arcana) -> arcana {
        reveal a + b;
    };

    engrave add_three_numbers(x: arcana, y: arcana, z: arcana) -> arcana {
        reveal add(add(x, y), z);
    };

    forge result: arcana = add_three_numbers(1, 2, 3);
    result;
    "#;

    let result_rust = 1 + 2 + 3;

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(result_abyss)) = results[3] {
                assert_eq!(result_rust, result_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_function_with_no_arguments() {
    let input = r#"
    engrave return_ten() -> arcana {
        reveal 10;
    };

    forge result: arcana = return_ten();
    result;
    "#;

    let result_rust = 10;

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(result_abyss)) = results[2] {
                assert_eq!(result_rust, result_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_function_with_recursive_calls() {
    let input = r#"
    engrave factorial(n: arcana) -> arcana {
        oracle(n <= 1) {
            (boon) => reveal 1;
            (hex) => reveal n * factorial(n - 1);
        };
    };

    forge result: arcana = factorial(5);
    result;
    "#;

    let result_rust = 120;

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(result_abyss)) = results[2] {
                assert_eq!(result_rust, result_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}
