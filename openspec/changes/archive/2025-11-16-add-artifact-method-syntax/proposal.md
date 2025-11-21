## Why
Artifacts currently encapsulate data only. To let structs manage their own behavior and pave the way for methodized standard library utilities, we must add first-class method definitions and invocations tied to artifact types.

## What Changes
- Extend the lexer and parser with the `core` keyword, `::` qualifier, and dot-call parsing that emits dedicated AST nodes for artifact methods.
- Associate `engrave Type::method(core ...)` definitions with their artifact schemas so the environment can look up immutable and mutable methods.
- Introduce evaluator support for resolving and invoking methods, implicitly threading the receiver value and enforcing `morph core` mutability rules at runtime.

## Impact
- Affected specs: parser-infrastructure, evaluator-infrastructure
- Affected code: `parser/tokens.rs`, `parser/grammar.rs`, AST definitions, environment registration for artifacts, evaluator call dispatch, standard library scaffolding for future refactors.
