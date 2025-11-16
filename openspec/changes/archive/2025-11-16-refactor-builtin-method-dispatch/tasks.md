## 1. Specification
- [x] 1.1 Align with runtime-builtins spec owners and confirm the scope of the method registry/staging changes.

## 2. Stdlib Restructure
- [x] 2.1 Move existing IO globals into `src/stdlib/functions/io.rs` and expose them through `functions::get_all_global_functions`.
- [x] 2.2 Scaffold `src/stdlib/methods/{mod,materia,scroll,lexicon}.rs` with shared types for builtin method definitions.
- [x] 2.3 Implement per-type method tables and register them with a central dispatcher that the evaluator can call.

## 3. Evaluator & Environment
- [x] 3.1 Remove `evaluate_builtin_method_call` in `src/eval/expressions.rs` and route method calls through the stdlib dispatcher.
- [x] 3.2 Update `src/env.rs` (and any supporting types) to store and expose the builtin method registry.
- [x] 3.3 Update `src/stdlib/mod.rs` to construct both global functions and method tables when seeding the environment.

## 4. Validation
- [x] 4.1 Update/extend tests to cover the new dispatch path (unit + integration). *(Existing collection/type suites already exercise the new dispatcher; no additional cases were required.)*
- [x] 4.2 Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test --all` to verify the refactor.
