## 1. Implementation
- [x] 1.1 Rework `Value` in `src/env.rs` and `EvalResult` in `src/eval/result.rs` so heap-backed data uses shared `Rc<RefCell<_>>` handles and collapse runtime data into `EvalResult::Data`.
- [x] 1.2 Update the evaluator modules (`src/eval/statements.rs`, `src/eval/expressions.rs`, and supporting helpers in `src/eval/values.rs`) to construct and propagate shared handles for literals, identifiers, assignments, arithmetic/string ops, and control flow without cloning collection contents.
- [x] 1.3 Refactor index access/assignment helpers in `src/eval/collections.rs` to borrow through `RefCell`, ensuring mutability checks honor shared references.
- [x] 1.4 Update stdlib collection builtins plus IO/repl display helpers to work with the new result shape.
- [x] 1.5 Refresh regression tests to assert against `EvalResult::Data` and verify aliasing semantics for scrolls/lexicons.
