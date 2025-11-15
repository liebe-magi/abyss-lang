## ADDED Requirements
### Requirement: Evaluator modules enforce concern boundaries
The interpreter SHALL organise the evaluator under `src/eval/` with separate Rust modules for result/diagnostic types, shared value helpers, collection indexing utilities, expression evaluation, and statement/control-flow execution, ensuring each file exposes a focused API and the public `eval` module simply re-exports the entry points (`evaluate`, `display_error_with_source`, `EvalResult`, `EvalError`).

#### Scenario: Expressions module handles pure computations
- **WHEN** arithmetic or comparison AST nodes are evaluated
- **THEN** the logic SHALL reside inside `eval::expressions` (or submodules nested beneath it)
- **AND** the statements layer SHALL invoke these helpers instead of embedding the code directly in `mod.rs`.

#### Scenario: Statements module drives control flow
- **WHEN** the interpreter executes statements such as `forge`, `morph`, `oracle`, `orbit`, and `engrave`
- **THEN** the implementation SHALL live in `eval::statements` (or nested submodules)
- **AND** it SHALL call shared helpers from the other evaluator modules rather than duplicating indexing/type conversion logic.

#### Scenario: Shared helpers centralise runtime types
- **WHEN** other crates access `EvalResult`, `EvalError`, or helper functions like value conversion and collection indexing
- **THEN** those types SHALL be declared in `eval::result` / `eval::values` / `eval::collections` and re-exported via `eval::mod`, keeping the rest of the codebase free from duplicate definitions or deep knowledge of the module internals.
