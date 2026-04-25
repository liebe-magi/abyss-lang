# Expand Test Coverage

## Goal
Increase patch coverage to >70% and project coverage to >80% by adding missing test cases for `abyss-core` and `abyss-interpreter`.

## Context
The current PR has a patch coverage of ~40%, which is below the required threshold. The missing coverage is primarily in error handling paths and the `SymbolTable` implementation.

## Strategy
1.  **SymbolTable**: Add comprehensive unit tests for `crates/abyss-core/src/semantic.rs`.
2.  **Evaluator Errors**: Add unit tests for `crates/abyss-interpreter/src/eval/expressions.rs` targeting specific error conditions (negative exponents, type mismatches, etc.).
3.  **Stdlib Validation**: Add tests for argument validation and receiver mutability checks in stdlib methods (`lexicon`, `scroll`, `materia`, `io`).
4.  **Environment**: Add tests for edge cases in `RuntimeEnv` (e.g., duplicate artifact definitions).

## Risks
-   None. This is a test-only change.
