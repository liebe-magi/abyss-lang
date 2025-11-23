# abyss-interpreter

The "Engine" of the AbySS language. This crate handles the execution of AbySS programs.

## Responsibilities

- **Runtime Environment**: Manages the dynamic scope, variable storage, and function definitions (`env.rs`, `RuntimeEnv`).
- **Values**: Defines the runtime representation of data, including `Value` enum (scalars, objects, arrays).
- **Evaluator**: Traverses the AST and executes operations (`eval/`).
- **Standard Library**: Implements built-in functions (`summon`, `unveil`) and methods for types like `Scroll` (arrays) and `Lexicon` (maps) (`stdlib/`).

## Usage

This crate depends on `abyss-core` for AST definitions. It is used by:
- The main CLI interpreter (`abyss-lang`).
- Future Wasm-based playgrounds (execution engine).

For more information, see the [main repository](https://github.com/liebe-magi/abyss-lang).
