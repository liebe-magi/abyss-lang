# Design: Spectrum Enum

## Architecture
The `spectrum` feature touches the AST, Parser, and Evaluator.

### AST Changes
- New `AST::SpectrumDef` to represent the definition.
- New `AST::SpectrumVariant` or similar to represent the usage/instantiation.
- Updates to `Type` enum to include `Spectrum(String)`.
- Updates to `Oracle` related AST nodes to support pattern matching patterns.

### Parser Changes
- New keywords: `spectrum`.
- Parse `spectrum Name { ... }` block.
- Parse `Name::Variant` syntax in expressions.
- Parse patterns in `oracle` branches.

### Evaluator Changes
- Store spectrum definitions in the environment (or a separate type registry).
- Evaluate `Name::Variant` to a runtime value (likely a new `Value::Spectrum` variant).
- Implement pattern matching logic in `eval_oracle`.

## Data Representation
A runtime `Value::Spectrum` will likely need to store:
- The name of the spectrum type.
- The name of the variant.
- The associated data (if any) as a list of values.

## Compatibility
- This is a purely additive change.
- Existing code should not be affected, unless `spectrum` is used as a variable name (which will now be a keyword).

## Open Questions
- **Generics**: The proposal mentions generics are not yet available. We will stick to concrete types for now.
- **Namespace**: `::` is introduced as a namespace separator. This might be useful for future module systems.
