# evaluator-infrastructure Specification

## Purpose
TBD - created by archiving change refactor-evaluator-architecture. Update Purpose after archive.
## Requirements
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

### Requirement: Types Are First-Class Global Variables
The evaluator SHALL treat every built-in type name (`arcana`, `rune`, `scroll`, `lexicon`, etc.) and every defined artifact type as a glyph-valued global variable that lives in the environment, allowing user code to reference types via ordinary identifier lookup instead of bespoke AST literals.

#### Scenario: Evaluate primitive glyph binding
- **GIVEN** the stdlib initialisation registers a symbol `arcana` bound to `Value::Glyph(Type::Arcana)`
- **WHEN** a script executes `forge t: glyph = arcana;`
- **THEN** the evaluator SHALL look up the existing global `arcana` variable, clone its `Value::Glyph(Type::Arcana)` handle, and bind `t` to that value just like any other variable assignment.

#### Scenario: Register artifact glyph variable
- **GIVEN** the program defines `artifact Player { level: arcana; }`
- **WHEN** the evaluator processes the artifact definition
- **THEN** it SHALL register both the artifact schema and a global variable named `Player` whose value is `Value::Glyph(Type::Artifact("Player"))`, making later statements like `forge builder: glyph = Player;` use standard identifier lookup to copy the glyph value.

### Requirement: Trans Conversion Implemented as Method
The evaluator SHALL provide a builtin method named `trans` on every `materia` value that accepts a single glyph argument, reuses the existing conversion rules (rune→arcana, arcana→rune, etc.), and raises descriptive `EvalError`s when the glyph argument is missing, not a glyph, or represents an unsupported conversion.

#### Scenario: Convert rune to arcana via method
- **GIVEN** source `forge lvl: arcana = "123".trans(arcana);`
- **WHEN** the evaluator runs the method call
- **THEN** it SHALL resolve the builtin `trans`, ensure the glyph argument equals `Type::Arcana`, parse the rune to `arcana`, and bind `lvl` to `EvalResult::Data(Value::Arcana(123))`.

#### Scenario: Reject non-glyph parameter
- **GIVEN** source `forge bad: arcana = 10.trans("arcana");`
- **WHEN** the evaluator executes the call
- **THEN** `trans` SHALL raise an `EvalError` explaining that the first argument must be a glyph value and SHALL leave the receiver unchanged.

#### Scenario: Reject unsupported conversion
- **GIVEN** a rune value and call `"hi".trans(scroll);`
- **WHEN** the evaluator resolves the glyph parameter to `Type::Scroll`
- **THEN** it SHALL raise an `EvalError` indicating runes cannot be converted to `scroll`, mirroring the existing conversion matrix.

