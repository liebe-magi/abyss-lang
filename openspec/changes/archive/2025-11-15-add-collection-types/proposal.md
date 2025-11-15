## Why
AbySS scripts cannot currently store heterogeneous collections or dynamically typed values, which limits data modelling beyond scalars. Collection support is also a prerequisite for future struct-like aggregates and method syntax. We need first-class list and map types so authored scripts can handle grouped data without custom encodings.

## What Changes
- Add the `scroll`, `lexicon`, and `materia` type keywords to the lexer and `Type` enum so functions and variables can declare collection shapes or dynamic inputs.
- Extend the grammar and AST with list/map literal nodes plus indexing and indexed-assignment expressions, including parser validation for rune keys inside map literals.
- Teach the evaluator/environment new `Value`/`EvalResult` variants (`Scroll`, `Lexicon`) together with `Type::Materia` semantics so collections can be created, mutated, and type-checked at runtime.
- Introduce stdlib builtins (`measure`, `inscribe`, `retract`, `expunge`, `contents`) that provide ergonomic collection utilities through the existing `Callable::Builtin` plumbing.
- Update tests, formatter, and diagnostics to exercise the new syntax and runtime behaviors while keeping backwards compatibility with existing programs.

## Impact
- Affected specs: `parser-infrastructure`, `runtime-builtins`
- Affected code: `src/ast.rs`, `src/parser/*`, `src/env.rs`, `src/eval.rs`, `src/stdlib/*`, formatter and evaluator tests in `tests/`
