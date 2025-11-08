## 1. Implementation
- [x] 1.1 Update `Cargo.toml` and regenerate `Cargo.lock` with `chumsky = "0.11"` and `ariadne = "0.6"`.
- [x] 1.2 Refactor `src/parser/diagnostics.rs` to the 0.11/0.6 error APIs (lifetimes, `Simple` pattern matching, `ariadne::Report` spans).
- [x] 1.3 Update `src/parser/{mod,grammar,helpers,tokens}.rs` to use `chumsky::input::Stream`, explicit lifetimes, and the replacement combinators for `map_with_span`, `filter`, and recovery helpers.
- [x] 1.4 Adjust lexer/number parsing utilities so they consume the new slice outputs and continue producing `SpannedToken` values.
- [x] 1.5 Fix or extend tests (including REPL/CLI fixtures) so diagnostics and parser behaviour remain stable under the new dependencies.

## 2. Validation
- [x] 2.1 Run `cargo fmt`.
- [x] 2.2 Run `cargo clippy --all-targets --all-features` and address new warnings.
- [x] 2.3 Run `cargo test` to confirm parser/evaluator regressions are caught.
- [x] 2.4 Execute `pre-commit run --all-files` to ensure all hooks succeed with the updated toolchain.

## 3. Documentation
- [x] 3.1 Finalise the spec delta in `openspec/changes/update-chumsky-ariadne-compat/specs/parser-infrastructure/spec.md`.
- [x] 3.2 Capture summary of migration considerations (lifetimes, diagnostics spans) in PR description or changelog if maintained.
