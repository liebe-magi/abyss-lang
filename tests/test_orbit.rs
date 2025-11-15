mod test_base;

use abyss_lang::eval::EvalResult;
use test_base::{Value, test_base};

#[test]
fn test_simple_orbit() {
    let input = r#"
    forge morph sum: arcana = 0;
    orbit(i = 0..10) {
        sum += i;
    };
    sum;
    "#;

    let mut sum_rust = 0;
    for i in 0..10 {
        sum_rust += i;
    }

    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(sum_abyss)) = results[2] {
                assert_eq!(sum_rust, sum_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_orbit_with_nested_orbit() {
    let input = r#"
    forge morph sum: arcana = 0;
    orbit(i = 0..3) {
        orbit(j = 0..3) {
            sum += i * j;
        };
    };
    sum;
    "#;

    // Rustでの同じアルゴリズム
    let mut sum_rust = 0;
    for i in 0..3 {
        for j in 0..3 {
            sum_rust += i * j;
        }
    }

    // AbySSでの評価
    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(sum_abyss)) = results[2] {
                assert_eq!(sum_rust, sum_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_orbit_with_resume() {
    let input = r#"
    forge morph sum: arcana = 0;
    orbit(i = 0..3, j = 0..3) {
        oracle(i == j) {
            (boon) => resume j; // jのループをスキップ
        };
        sum += i * j;
    };
    sum;
    "#;

    // Rustでの同じアルゴリズム
    let mut sum_rust = 0;
    for i in 0..3 {
        for j in 0..3 {
            if i == j {
                continue; // jのループをスキップ
            }
            sum_rust += i * j;
        }
    }

    // AbySSでの評価
    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(sum_abyss)) = results[2] {
                assert_eq!(sum_rust, sum_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_orbit_with_eject() {
    let input = r#"
    forge morph sum: arcana = 0;
    orbit(i = 0..5) {
        oracle(i == 3) {
            (boon) => eject; // ループ全体を抜ける
        };
        sum += i;
    };
    sum;
    "#;

    // Rustでの同じアルゴリズム
    let mut sum_rust = 0;
    for i in 0..5 {
        if i == 3 {
            break; // ループ全体を抜ける
        }
        sum_rust += i;
    }

    // AbySSでの評価
    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(sum_abyss)) = results[2] {
                assert_eq!(sum_rust, sum_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_orbit_with_eject_outer_loop() {
    let input = r#"
    forge morph sum: arcana = 0;
    orbit(i = 0..3) {
        orbit(j = 0..3) {
            oracle(i == j) {
                (boon) => eject i; // 外側のループを抜ける
            };
            sum += i * j;
        };
    };
    sum;
    "#;

    // Rustでの同じアルゴリズム
    let mut sum_rust = 0;
    'outer: for i in 0..3 {
        for j in 0..3 {
            if i == j {
                break 'outer; // 外側のループを抜ける
            }
            sum_rust += i * j;
        }
    }

    // AbySSでの評価
    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(sum_abyss)) = results[2] {
                assert_eq!(sum_rust, sum_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_orbit_with_resume_outer_loop() {
    let input = r#"
    forge morph sum: arcana = 0;
    orbit(i = 0..3) {
        oracle(i == 1) {
            (boon) => resume i; // 外側のループをスキップ
        };
        sum += i;
    };
    sum;
    "#;

    // Rustでの同じアルゴリズム
    let mut sum_rust = 0;
    for i in 0..3 {
        if i == 1 {
            continue; // 外側のループをスキップ
        }
        sum_rust += i;
    }

    // AbySSでの評価
    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(sum_abyss)) = results[2] {
                assert_eq!(sum_rust, sum_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_infinite_orbit_with_eject() {
    let input = r#"
    forge morph count: arcana = 0;
    orbit {
        count += 1;
        oracle(count == 10) {
            (boon) => eject; // 無限ループを10回で終了
        };
    };
    count;
    "#;

    // Rustでの同じアルゴリズム
    let mut count_rust = 0;
    loop {
        count_rust += 1;
        if count_rust == 10 {
            break; // 10回でループを終了
        }
    }

    // AbySSでの評価
    match test_base(input) {
        Ok(results) => {
            if let EvalResult::Data(Value::Arcana(count_abyss)) = results[2] {
                assert_eq!(count_rust, count_abyss);
            } else {
                panic!("Expected Arcana result");
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}
