## ADDED Requirements
### Requirement: Stdlib centralises builtin method dispatch
The runtime stdlib SHALL own a registry of builtin methods keyed first by runtime type (e.g., `Type::Scroll`, `Type::Lexicon`, `Type::Materia`) and then by method name so the evaluator can remain agnostic of stdlib behavior. The registry SHALL live inside `stdlib::methods` alongside Rust implementations for each supported receiver type, and SHALL expose a single dispatcher that accepts the receiver `Value`, method name, argument list, and evaluation context.

#### Scenario: Register method tables during startup
- **WHEN** `stdlib::create_global_environment` runs
- **THEN** it SHALL call `stdlib::methods::get_all_builtin_methods()` (or equivalent) to build a per-type method table
- **AND** it SHALL store the resulting registry in the global environment so every scope can reuse the dispatcher.

#### Scenario: Evaluator delegates method calls
- **GIVEN** user code invokes `bag.tally()` on a scroll
- **WHEN** the evaluator resolves the receiver value and observes its runtime type
- **THEN** it SHALL pass the receiver, method name (`"tally"`), and arguments into the stdlib dispatcher
- **AND** the dispatcher SHALL look up the scroll method table and execute the registered Rust implementation.

#### Scenario: Unknown method surfaces coherent error
- **GIVEN** user code calls `lexicon.unknown()`
- **WHEN** the dispatcher fails to find the requested method in the lexicon table
- **THEN** it SHALL raise a runtime error that identifies the receiver type and method name, leaving the evaluator logic unchanged.
