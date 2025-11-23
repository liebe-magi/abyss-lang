mod test_base;

use abyss_interpreter::eval::{EvalError, EvalResult};
use test_base::{Value, test_base};

#[test]
fn artifact_instantiation_and_access() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
forge hero: Player = Player { name: "Ardyn", health: 100 };
hero.health;
"#;

    let results = test_base(input).expect("artifact evaluation failed");
    assert_eq!(results.len(), 3);
    match &results[2] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 100),
        other => panic!("expected arcana result, found {:?}", other),
    }
}

#[test]
fn artifact_field_assignment_requires_morph() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
forge hero: Player = Player { name: "Ardyn", health: 100 };
hero.health = 75;
"#;

    match test_base(input) {
        Ok(_) => panic!("expected immutability error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(message, _)) => {
                assert!(
                    message.contains("immutable"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected invalid operation error, found {:?}", other),
        },
    }
}

#[test]
fn artifact_field_assignment_updates_value() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
forge morph hero: Player = Player { name: "Ardyn", health: 100 };
hero.health = 75;
hero.health;
"#;

    let results = test_base(input).expect("artifact mutation failed");
    assert_eq!(results.len(), 4);
    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 75),
        other => panic!("expected arcana result, found {:?}", other),
    }
}

#[test]
fn artifact_literal_requires_all_fields() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
forge hero: Player = Player { name: "Ardyn" };
"#;

    match test_base(input) {
        Ok(_) => panic!("expected missing field error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(message, _)) => {
                assert!(
                    message.contains("missing fields"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected invalid operation error, found {:?}", other),
        },
    }
}

#[test]
fn artifact_variable_type_mismatch_errors() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
artifact Enemy { name: rune; damage: arcana; };
forge hero: Player = Enemy { name: "Goblin", damage: 7 };
"#;

    match test_base(input) {
        Ok(_) => panic!("expected type mismatch error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::TypeError(message, _)) => {
                assert!(
                    message.contains("Expected artifact of type"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected type error, found {:?}", other),
        },
    }
}

#[test]
fn artifact_equality_compares_fields() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
forge hero: Player = Player { name: "Ardyn", health: 100 };
forge clone: Player = Player { name: "Ardyn", health: 100 };
hero == clone;
"#;

    let results = test_base(input).expect("artifact equality failed");
    assert_eq!(results.len(), 4);
    match &results[3] {
        EvalResult::Data(Value::Omen(value)) => assert!(*value),
        other => panic!("expected omen result, found {:?}", other),
    }
}

#[test]
fn artifact_parameter_type_check() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
artifact Enemy { name: rune; damage: arcana; };
engrave describe(target: Player) -> rune {
    reveal target.name;
};
forge foe: Enemy = Enemy { name: "Ooze", damage: 3 };
describe(foe);
"#;

    match test_base(input) {
        Ok(_) => panic!("expected parameter type error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::TypeError(message, _)) => {
                assert!(
                    message.contains("Expected artifact of type Player"),
                    "unexpected message: {}",
                    message
                );
            }
            other => panic!("expected type error, found {:?}", other),
        },
    }
}

#[test]
fn artifact_function_return_type() {
    let input = r#"
artifact Player { name: rune; health: arcana; };
engrave create_player(name: rune) -> Player {
    reveal Player { name: name, health: 100 };
};
create_player("Nova");
"#;

    let results = test_base(input).expect("artifact return failed");
    assert_eq!(results.len(), 3);
    match &results[2] {
        EvalResult::Artifact(handle) => {
            let borrowed = handle.borrow();
            assert_eq!(borrowed.type_name, "Player");
            match borrowed.fields.get("name") {
                Some(Value::Rune(value)) => assert_eq!(value.as_ref(), "Nova"),
                other => panic!("expected rune field, found {:?}", other),
            }
        }
        other => panic!("expected artifact result, found {:?}", other),
    }
}

#[test]
fn artifact_method_invocation_returns_value() {
    let input = r#"
artifact Player { level: arcana; };
engrave Player::get_level(core) -> arcana {
    reveal core.level;
};
forge hero: Player = Player { level: 10 };
hero.get_level();
"#;

    let results = test_base(input).expect("method invocation failed");
    assert_eq!(results.len(), 4);
    match &results[3] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 10),
        other => panic!("expected arcana result, found {:?}", other),
    }
}

#[test]
fn mutable_method_requires_morph_receiver() {
    let input = r#"
artifact Player { level: arcana; };
engrave Player::set_level(morph core, next: arcana) -> abyss {
    core.level = next;
};
forge hero: Player = Player { level: 5 };
hero.set_level(8);
"#;

    match test_base(input) {
        Ok(_) => panic!("expected immutable receiver error"),
        Err(err) => match err.downcast_ref::<EvalError>() {
            Some(EvalError::InvalidOperation(message, _)) => {
                assert!(
                    message.contains("immutable receiver"),
                    "unexpected error message: {}",
                    message
                );
            }
            other => panic!("expected invalid operation error, found {:?}", other),
        },
    }
}

#[test]
fn mutable_method_updates_morph_receiver() {
    let input = r#"
artifact Player { level: arcana; };
engrave Player::set_level(morph core, next: arcana) -> abyss {
    core.level = next;
};
forge morph hero: Player = Player { level: 5 };
hero.set_level(12);
hero.level;
"#;

    let results = test_base(input).expect("mutable method should succeed");
    assert_eq!(results.len(), 5);
    match &results[4] {
        EvalResult::Data(Value::Arcana(value)) => assert_eq!(*value, 12),
        other => panic!("expected arcana result, found {:?}", other),
    }
}
