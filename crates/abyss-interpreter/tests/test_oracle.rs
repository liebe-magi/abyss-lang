mod test_base;

use abyss_interpreter::eval::EvalResult;
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
    forge y: arcana = x ^ 2;
    oracle {
        (y > 100) => "y is greater than 100";
        (y == 100) => "y is equal to 100";
        _ => "y is less than 100";
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[2], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "y is greater than 100")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_with_string_comparison() {
    let input = r#"
    forge a: rune = "abyss";
    oracle {
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
    forge a: arcana = 1;
    forge b: arcana = 3;
    oracle {
        (a == 1 && b == 2) => reveal("a is 1 and b is 2");
        (a != 1 && b == 2) => reveal("a is not 1 and b is 2");
        (a == 1 && b != 2) => reveal("a is 1 and b is not 2");
        _ => reveal("a is not 1 and b is not 2");
    };
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(
                matches!(results[2], EvalResult::Data(Value::Rune(ref s)) if s.as_ref() == "a is 1 and b is not 2")
            )
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_ward_passes_when_guard_is_boon() {
    // The first arm matches ("active") and the ward expression is true,
    // so it should fire instead of falling through to the bare ("active") arm.
    let input = r#"
    forge mode: rune = "active";
    forge count: arcana = 5;
    forge result: rune = oracle (mode) {
        ("active") ward count > 0 => reveal("ready");
        ("active") => reveal("idle");
        _ => reveal("inactive");
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "ready")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_ward_falls_through_when_guard_is_hex() {
    // The first arm matches ("active") but the ward fails, so the next
    // ("active") arm without a ward should fire.
    let input = r#"
    forge mode: rune = "active";
    forge count: arcana = 0;
    forge result: rune = oracle (mode) {
        ("active") ward count > 0 => reveal("ready");
        ("active") => reveal("idle");
        _ => reveal("inactive");
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "idle"))
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_ward_with_non_omen_errors() {
    // A ward expression that does not yield an omen surfaces a runtime error.
    let input = r#"
    forge n: arcana = 1;
    oracle (n) {
        (1) ward 42 => "never";
        _ => "never either";
    };
    "#;
    match test_base(input) {
        Ok(results) => panic!("expected ward type error, got {:?}", results),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("Oracle ward must evaluate to an omen"),
                "unexpected error: {}",
                msg
            )
        }
    }
}

#[test]
fn test_oracle_if_else_ward_acts_as_extra_condition() {
    // In if-else mode, ward composes with the existing boolean pattern as
    // an extra conjunctive condition. Here the pattern (x > 0) is true and
    // the ward is true, so the arm fires.
    let input = r#"
    forge x: arcana = 7;
    forge y: arcana = 3;
    forge result: rune = oracle {
        (x > 0) ward y > 0 => reveal("both positive");
        (x > 0) => reveal("only x positive");
        _ => reveal("other");
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "both positive")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_binds_bare_identifier_pattern() {
    // A bare identifier pattern in match mode binds the scrutinee to that
    // name in the arm's scope. The body can then read the bound value.
    let input = r#"
    forge n: arcana = 7;
    forge result: arcana = oracle (n) {
        (x) => reveal(x * 2);
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[2], EvalResult::Data(Value::Arcana(14)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_binding_visible_in_ward() {
    // The bound name is visible to the ward expression of the same arm.
    // First arm: ward succeeds, so it fires. Second arm is the fallback.
    let input = r#"
    forge n: arcana = 7;
    forge result: rune = oracle (n) {
        (x) ward x > 0 => reveal("positive");
        (x) => reveal("non-positive");
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[2], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "positive")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_multiple_bindings_across_scrutinees() {
    // Multi-scrutinee patterns bind every bare-identifier element to the
    // matching scrutinee value.
    let input = r#"
    forge a: arcana = 3;
    forge b: arcana = 4;
    forge sum: arcana = oracle (a, b) {
        (x, y) => reveal(x + y);
    };
    sum;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[3], EvalResult::Data(Value::Arcana(7)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_binding_mixed_with_literals_and_wildcards() {
    // Literals still match by value, wildcards still skip, and identifiers
    // still bind — composing freely inside the same pattern tuple.
    let input = r#"
    forge a: arcana = 1;
    forge b: arcana = 99;
    forge result: rune = oracle (a, b) {
        (1, x) => reveal("first is one");
        (_, x) => reveal("first is not one");
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "first is one")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_match_binding_does_not_leak_to_outer_scope() {
    // The binding is confined to the per-branch scope, so the outer script
    // cannot see it after the oracle finishes.
    let input = r#"
    forge n: arcana = 5;
    oracle (n) {
        (x) => reveal(x);
    };
    x;
    "#;
    match test_base(input) {
        Ok(results) => panic!(
            "expected the post-oracle reference to `x` to fail, got {:?}",
            results
        ),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("UndefinedVariable") && msg.contains('x'),
                "unexpected error: {}",
                msg
            )
        }
    }
}

#[test]
fn test_oracle_match_binding_does_not_leak_between_arms() {
    // First arm binds x = the scrutinee, but its ward fails so the arm is
    // skipped. The second arm then re-binds x to the same scrutinee in its
    // own fresh scope; if the first arm's binding had leaked it would show
    // up here, but each arm runs in its own scope so the second arm's body
    // sees the value via its own binding cleanly.
    let input = r#"
    forge n: arcana = 5;
    forge picked: arcana = oracle (n) {
        (x) ward x > 100 => reveal(0);
        (x) => reveal(x);
    };
    picked;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[2], EvalResult::Data(Value::Arcana(5)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_empty_matches_empty_scroll() {
    let input = r#"
    forge xs: scroll = [];
    forge result: rune = oracle (xs) {
        [] => reveal("empty");
        [..] => reveal("non-empty");
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[2], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "empty")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_exact_length_binds_each_element() {
    let input = r#"
    forge xs: scroll = [10, 20];
    forge result: arcana = oracle (xs) {
        [a, b] => reveal(a + b);
        _ => reveal(0);
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[2], EvalResult::Data(Value::Arcana(30)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_head_tail_binds_rest_as_subscroll() {
    // The named rest should bind a fresh sub-scroll containing every
    // unmatched trailing element. Verified here by counting it with
    // `tally` and reading its first element back.
    let input = r#"
    forge xs: scroll = [10, 20, 30, 40];
    forge head_val: arcana = oracle (xs) {
        [head, ..rest] => reveal(head);
    };
    forge rest_len: arcana = oracle (xs) {
        [_, ..rest] => reveal(rest.tally());
    };
    head_val;
    rest_len;
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(matches!(results[3], EvalResult::Data(Value::Arcana(10))));
            assert!(matches!(results[4], EvalResult::Data(Value::Arcana(3))));
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_anonymous_rest_drops_tail() {
    // `[head, ..]` keeps the head binding but ignores the trailing
    // values entirely.
    let input = r#"
    forge xs: scroll = [7, 8, 9];
    forge result: arcana = oracle (xs) {
        [head, ..] => reveal(head);
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[2], EvalResult::Data(Value::Arcana(7)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_falls_through_on_length_mismatch() {
    // The first arm requires exactly two elements; the input has three,
    // so the arm should not fire and the trailing-rest arm should match.
    let input = r#"
    forge xs: scroll = [1, 2, 3];
    forge label: rune = oracle (xs) {
        [a, b] => reveal("two");
        [head, ..rest] => reveal("more");
        _ => reveal("other");
    };
    label;
    "#;
    match test_base(input) {
        Ok(results) => {
            assert!(matches!(&results[2], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "more"))
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_literal_elements_compare_by_value() {
    // A literal element is still compared by value; a binding next to
    // it captures the matching neighbour.
    let input = r#"
    forge xs: scroll = [42, 100];
    forge label: rune = oracle (xs) {
        [42, x] => reveal("starts with 42, second=" + x.trans(rune));
        [_, x] => reveal("other, second=" + x.trans(rune));
    };
    label;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[2], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "starts with 42, second=100")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_binding_visible_in_ward() {
    // A binding from a scroll pattern is visible to the same arm's ward.
    let input = r#"
    forge xs: scroll = [3, 4, 5];
    forge label: rune = oracle (xs) {
        [head, ..rest] ward head > 10 => reveal("big");
        [head, ..rest] => reveal("small head: " + head.trans(rune));
    };
    label;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[2], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "small head: 3")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_scroll_pattern_against_non_scroll_scrutinee_errors() {
    // Type mismatch (scroll pattern against an arcana scrutinee) raises
    // a runtime error rather than silently falling through, matching
    // the existing tuple-pattern type-mismatch behaviour.
    let input = r#"
    forge n: arcana = 5;
    oracle (n) {
        [a, b] => reveal("never");
        _ => reveal("never either");
    };
    "#;
    match test_base(input) {
        Ok(results) => panic!("expected scroll/scalar mismatch, got {:?}", results),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("Scroll pattern requires a scroll scrutinee"),
                "unexpected error: {}",
                msg
            )
        }
    }
}

#[test]
fn test_oracle_artifact_pattern_shorthand_binds_each_field() {
    // `Player { name, health }` shorthand binds both fields by their
    // declared names.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    forge hero: Player = Player { name: "Ardyn", health: 100 };
    forge result: arcana = oracle (hero) {
        Player { name, health } => reveal(health);
    };
    result;
    "#;
    match test_base(input) {
        Ok(results) => assert!(matches!(results[3], EvalResult::Data(Value::Arcana(100)))),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_artifact_pattern_partial_field_list() {
    // Listing only some fields is allowed — the pattern is non-exhaustive
    // by default, so `Player { name }` matches without mentioning `health`.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    forge hero: Player = Player { name: "Ardyn", health: 100 };
    forge greeting: rune = oracle (hero) {
        Player { name } => reveal("hello, " + name);
    };
    greeting;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "hello, Ardyn")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_artifact_pattern_literal_field_value() {
    // Explicit `field_name: <expr>` compares the field by value rather
    // than binding it; the binding next to it still captures.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    forge hero: Player = Player { name: "Ardyn", health: 100 };
    forge label: rune = oracle (hero) {
        Player { name: "Ardyn", health } => reveal("chosen, hp=" + health.trans(rune));
        Player { name } => reveal("someone: " + name);
    };
    label;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "chosen, hp=100")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_artifact_pattern_dispatches_on_type() {
    // Wrong artifact type falls through so sibling arms can match.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    artifact Enemy { name: rune; damage: arcana; };
    forge foe: Enemy = Enemy { name: "Goblin", damage: 7 };
    forge label: rune = oracle (foe) {
        Player { name } => reveal("hero " + name);
        Enemy { name, damage } => reveal("foe " + name + " (" + damage.trans(rune) + ")");
    };
    label;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[4], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "foe Goblin (7)")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_artifact_pattern_binding_visible_in_ward() {
    // Bindings introduced by an artifact pattern are visible to the same
    // arm's ward expression.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    forge weak: Player = Player { name: "Frail", health: 20 };
    forge label: rune = oracle (weak) {
        Player { name, health } ward health < 50 => reveal("run, " + name + "!");
        Player { name } => reveal("hello, " + name);
    };
    label;
    "#;
    match test_base(input) {
        Ok(results) => assert!(
            matches!(&results[3], EvalResult::Data(Value::Rune(s)) if s.as_ref() == "run, Frail!")
        ),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_oracle_artifact_pattern_unknown_field_errors_with_hint() {
    // A misspelled field surfaces the existing PR4-B did-you-mean hint via
    // `missing_field_error`.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    forge hero: Player = Player { name: "Ardyn", health: 100 };
    oracle (hero) {
        Player { helt } => reveal("never");
    };
    "#;
    match test_base(input) {
        Ok(results) => panic!("expected unknown-field error, got {:?}", results),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("Field 'helt'") && msg.contains("did you mean: health"),
                "unexpected error: {}",
                msg
            )
        }
    }
}

#[test]
fn test_oracle_artifact_pattern_against_non_artifact_errors() {
    // A non-artifact scrutinee is a clear programmer mistake and surfaces
    // a runtime error rather than silently falling through.
    let input = r#"
    artifact Player { name: rune; health: arcana; };
    forge n: arcana = 5;
    oracle (n) {
        Player { name } => reveal("never");
        _ => reveal("never either");
    };
    "#;
    match test_base(input) {
        Ok(results) => panic!("expected non-artifact scrutinee error, got {:?}", results),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("Artifact pattern requires an artifact scrutinee"),
                "unexpected error: {}",
                msg
            )
        }
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
