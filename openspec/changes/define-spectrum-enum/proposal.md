# Define Spectrum Enum

## Summary
Introduce `spectrum`, a user-defined enumerated type (sum type) that supports both unit variants and tuple variants. This feature allows developers to model states and complex data structures more expressively. It includes syntax for definition, instantiation via `::`, and pattern matching integration with the `oracle` statement.

## Motivation
Currently, `abyss-lang` lacks a robust way to represent a value that can be one of several distinct types or states (sum types). Developers often resort to using loose constants or string literals, which is error-prone and lacks type safety. `spectrum` provides a first-class mechanism for this, aligning with modern language features like Rust's `enum` or Swift's `enum`.

## Proposed Solution
1.  **Definition**: New `spectrum` keyword to define enums with unit and tuple variants.
2.  **Instantiation**: Use `SpectrumName::VariantName` syntax to create instances.
3.  **Matching**: Extend `oracle` to support destructuring and matching against spectrum variants.

## Alternatives Considered
- **Classes/Structs**: Could simulate enums with class hierarchies, but `abyss-lang` focuses on a simpler, more data-oriented approach.
- **String Constants**: Too loose and untyped.
