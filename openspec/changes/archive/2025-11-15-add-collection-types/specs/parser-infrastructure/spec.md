## ADDED Requirements
### Requirement: Collection Type Keywords
The parser SHALL treat `scroll`, `lexicon`, and `materia` as reserved type keywords, lex them distinctly from identifiers, and emit the matching `Type` variants when they appear in `forge`, `engrave`, or `orbit` declarations.

#### Scenario: Parse scroll declaration
- **GIVEN** source `forge items: scroll = [];`
- **WHEN** the parser processes the declaration
- **THEN** it SHALL emit a `Type::Scroll` entry for the variable and reject using `scroll` as an identifier in the same position.

#### Scenario: Parse materia parameter
- **GIVEN** source `engrave print_any(val: materia) { unveil(val); }`
- **WHEN** the parser builds the function signature
- **THEN** it SHALL emit an `AST::EngraveParam` whose `param_type` is `Type::Materia`.

### Requirement: Collection Literals
The parser SHALL support list (`scroll`) and map (`lexicon`) literals using `[...]` and `{...}` syntax and emit `AST::ListLiteral` / `AST::MapLiteral` nodes that preserve element order and literal spans.

#### Scenario: Parse scroll literal
- **GIVEN** source `forge mixed: scroll = [1, "hi", boon];`
- **WHEN** the parser encounters the bracketed expression
- **THEN** it SHALL produce an `AST::ListLiteral` containing three child expressions in order.

#### Scenario: Parse lexicon literal with rune keys
- **GIVEN** source `forge data: lexicon = {"id": 1, "name": "abyss"};`
- **WHEN** the parser processes the braces
- **THEN** it SHALL produce an `AST::MapLiteral` and enforce that each key token is a `rune` literal.

### Requirement: Collection Indexing Syntax
The parser SHALL emit dedicated AST nodes for `expr[index]` access and `expr[index] = value` assignment so the evaluator can differentiate between reads and writes.

#### Scenario: Parse scroll index access
- **GIVEN** source `forge val: arcana = items[2];`
- **WHEN** the parser handles the square-bracket expression
- **THEN** it SHALL emit an `AST::IndexAccess` whose left child is the `items` reference and whose right child is the literal `2`.

#### Scenario: Parse indexed assignment
- **GIVEN** source `items[i] = rune_value;`
- **WHEN** the parser consumes the assignment tokens
- **THEN** it SHALL emit an `AST::IndexAssignment` capturing the collection expression, index expression, and assigned value while preserving the assignment span for diagnostics.
