# abyss-core

The "Brain" of the AbySS language. This crate provides the fundamental building blocks for parsing and analyzing AbySS code.

## Responsibilities

- **AST (Abstract Syntax Tree)**: Defines the structure of the language (`ast.rs`).
- **Parser**: A `chumsky`-based parser that converts source code into AST (`parser/`).
- **Type System**: Defines the `Type` enum and type checking primitives.
- **SymbolTable**: Manages static scope information (variable names, types, mutability) for static analysis (`semantic.rs`).

## Usage

This crate is designed to be lightweight and platform-agnostic, making it suitable for:
- The main CLI interpreter (`abyss-lang`).
- Future LSP (Language Server Protocol) implementations.
- Future Wasm-based playgrounds (parsing only).

For more information, see the [main repository](https://github.com/liebe-magi/abyss-lang).
