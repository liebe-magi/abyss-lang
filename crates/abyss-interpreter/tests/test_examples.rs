mod test_base;

use std::fs;
use std::path::PathBuf;
use test_base::test_base;

fn load_example(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join(name);
    fs::read_to_string(path).expect("failed to read example script")
}

#[test]
fn hello_example_executes() {
    let source = load_example("hello.aby");
    test_base(&source).expect("hello example failed to execute");
}

#[test]
fn fibonacci_example_executes() {
    let source = load_example("fibonacci.aby");
    test_base(&source).expect("fibonacci example failed to execute");
}

#[test]
fn artifact_example_executes() {
    let source = load_example("artifact.aby");
    test_base(&source).expect("artifact example failed to execute");
}

#[test]
fn pattern_example_executes() {
    let source = load_example("pattern.aby");
    test_base(&source).expect("pattern example failed to execute");
}
