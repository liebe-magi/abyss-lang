mod test_base;

use abyss_lang::eval::EvalResult;
use test_base::test_base;

#[test]
fn test_cast_arcana_to_aether() {
    let input = "trans(42 as aether);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Aether(n) => assert_eq!(*n, 42.0),
                _ => panic!("Expected an Aether result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_aether_to_arcana() {
    let input = "trans(3.14 as arcana);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Arcana(n) => assert_eq!(*n, 3),
                _ => panic!("Expected an Arcana result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_rune_to_aether() {
    let input = "trans(\"3.14\" as aether);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Aether(n) => {
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
    let input = "trans(\"123\" as arcana);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Arcana(n) => assert_eq!(*n, 123),
                _ => panic!("Expected an Arcana result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_arcana_to_rune() {
    let input = "trans(123 as rune);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Rune(s) => assert_eq!(s, "123"),
                _ => panic!("Expected a Rune result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_cast_aether_to_rune() {
    let input = "trans(3.14 as rune);";
    match test_base(input) {
        Ok(results) => {
            assert_eq!(results.len(), 1);
            match &results[0] {
                EvalResult::Rune(s) => assert_eq!(s, "3.14"),
                _ => panic!("Expected a Rune result"),
            }
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}
