## Why

Interpreter values currently copy every collection on assignment (`Value::Scroll(Vec<Value>)`, `Value::Lexicon(HashMap<_, _>)`, etc.). Large scripts suffer quadratic slowdowns, users cannot rely on reference-style semantics (mutating `b` after `forge b = a;` fails to affect `a`), and the lack of shared heap objects blocks any future GC work. We need shared ownership with interior mutability to align runtime behaviour with user expectations and prepare for GC-backed memory management.

## What Changes

- Re-model runtime data so heap-backed variants of `Value` use `Rc`/`RefCell` handles (`Rune`, `Scroll`, `Lexicon`) while primitives stay `Copy`.
- Collapse `EvalResult` into a small wrapper that differentiates runtime data (`EvalResult::Data(Value)`) from control-flow signals (`Revealed`, `Resume`, `Eject`).
- Update the evaluator modules (`src/eval/statements.rs`, `src/eval/expressions.rs`, and `src/eval/values.rs`) to construct/propagate `Value` handles for literals, identifiers, assignments, arithmetic/string operations, and index expressions, ensuring mutations borrow through `RefCell` instead of cloning vectors/maps.
- Rewrite stdlib collection builtins (`inscribe`, `retract`, `expunge`, `contents`, `measure`) to operate on the new shared handles by borrowing mutably or immutably as needed.
- Adjust REPL / IO formatting helpers to unwrap `EvalResult::Data` before presenting values, and update the test suite to assert against the consolidated result shape.

## Impact

- Affected specs: `runtime-builtins` (collection semantics, evaluator/runtime value model).
- Affected code: `src/env.rs`, `src/eval/mod.rs`, `src/eval/result.rs`, `src/eval/values.rs`, `src/eval/collections.rs`, `src/eval/expressions.rs`, `src/eval/statements.rs`, `src/stdlib/collections.rs`, `src/stdlib/io.rs`, `src/main.rs`, and regression tests covering `EvalResult` matching and collection mutation.
