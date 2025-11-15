## 1. Implementation
- [x] 1.1 Scaffold `src/eval/` with `mod.rs`, `result.rs`, `values.rs`, `collections.rs`, `expressions.rs`, and `statements.rs`, wiring the module tree without behaviour changes.
- [x] 1.2 Move `EvalResult`, `EvalError`, and conversion helpers into the new modules; ensure public re-exports keep the `evaluate` API unchanged.
- [x] 1.3 Relocate expression/statement evaluation logic into their respective files and add concise inline docs where necessary for clarity.
- [x] 1.4 Update `lib.rs`, `main.rs`, stdlib modules, and tests to import from the reorganised modules and ensure references compile.
- [x] 1.5 Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` to prove the refactor is behaviour-neutral.
