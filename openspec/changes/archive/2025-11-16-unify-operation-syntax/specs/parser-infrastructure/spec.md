## ADDED Requirements
### Requirement: Glyph Type Keyword
The parser SHALL reserve the `glyph` keyword for type annotations and emit a dedicated `Type::Glyph` whenever `glyph` appears in signatures, variable declarations, or other typed constructs so functions can accept "type-as-value" parameters.

#### Scenario: Parse glyph parameter
- **GIVEN** source `engrave transcribe(target: glyph) -> rune { ... }`
- **WHEN** the parser processes the signature
- **THEN** it SHALL emit a parameter typed as `Type::Glyph`
- **AND** SHALL reject using `glyph` as an identifier in the same position.

### Requirement: Conversions Use Method Call Syntax
The parser SHALL represent conversions exclusively through dot-call method syntax (`expression.trans(arg, ...)`) leveraging the existing `AST::MethodCall` node, and SHALL reject the legacy `trans(value as type)` special form so only one conversion grammar remains.

#### Scenario: Parse method-based trans call
- **GIVEN** source `"123".trans(arcana);`
- **WHEN** the parser reaches the dot expression with parentheses
- **THEN** it SHALL emit an `AST::MethodCall` whose receiver is the string literal, method name is `trans`, and arguments list contains a single `AST::Var` referencing the identifier `arcana`.

#### Scenario: Reject legacy trans syntax
- **GIVEN** source `trans("123" as arcana)`
- **WHEN** the parser attempts to recognise the form
- **THEN** it SHALL raise a diagnostic indicating the `trans(value as type)` syntax is no longer supported and SHALL NOT emit the old AST variant.
