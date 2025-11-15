## ADDED Requirements
### Requirement: Artifact Type Definition Syntax
The parser SHALL recognize the `artifact` keyword and support struct-like type definitions with named fields and type annotations, emitting `AST::ArtifactDef` nodes that capture the artifact name, field declarations, and source spans.

#### Scenario: Parse artifact definition
- **GIVEN** source `artifact Player { name: rune; health: arcana; }`
- **WHEN** the parser processes the artifact statement
- **THEN** it SHALL emit an `AST::ArtifactDef` node with name "Player" and two fields: "name" of type `Type::Rune` and "health" of type `Type::Arcana`
- **AND** the artifact name SHALL follow identifier rules (start with letter, alphanumeric + underscore)
- **AND** field names SHALL be unique within the artifact definition.

#### Scenario: Parse nested artifact fields
- **GIVEN** source `artifact Stats { max_hp: arcana; current_hp: arcana; }
artifact Character { name: rune; stats: Stats; }`
- **WHEN** the parser processes both definitions
- **THEN** it SHALL emit two `AST::ArtifactDef` nodes where the second references the first as a field type
- **AND** the parser SHALL validate that referenced artifact types exist during semantic analysis.

#### Scenario: Reject duplicate field names
- **GIVEN** source `artifact Item { name: rune; name: arcana; }`
- **WHEN** the parser encounters the duplicate field
- **THEN** it SHALL emit a diagnostic error indicating duplicate field "name" with spans for both occurrences.

### Requirement: Artifact Instantiation Syntax
The parser SHALL support artifact literal syntax using the artifact type name followed by field-value pairs in braces, emitting `AST::ArtifactLiteral` nodes that preserve field order and value expressions.

#### Scenario: Parse artifact literal
- **GIVEN** source `forge hero: Player = Player { name: "Ardyn", health: 100 };`
- **WHEN** the parser processes the artifact literal
- **THEN** it SHALL emit an `AST::ArtifactLiteral` with type name "Player" and two field assignments
- **AND** each field assignment SHALL pair a field name identifier with an expression node.

#### Scenario: Parse artifact with expression values
- **GIVEN** source `Player { name: summon("Name: "), health: 50 + 50 }`
- **WHEN** the parser constructs the literal
- **THEN** it SHALL preserve the full expression ASTs for each field value
- **AND** evaluation SHALL occur when the literal is instantiated at runtime.

#### Scenario: Require all fields in literal
- **GIVEN** an artifact definition with three fields
- **WHEN** a literal provides only two fields
- **THEN** the evaluator SHALL raise a runtime error listing the missing field
- **AND** the parser SHALL permit partial literals syntactically but defer validation to evaluation time.

### Requirement: Field Access Syntax
The parser SHALL emit `AST::FieldAccess` nodes for dot notation expressions (`instance.field`) so the evaluator can retrieve field values from artifact instances.

#### Scenario: Parse field access
- **GIVEN** source `forge hp: arcana = hero.health;`
- **WHEN** the parser encounters the dot operator
- **THEN** it SHALL emit an `AST::FieldAccess` whose left child is the identifier "hero" and right child is the field name "health"
- **AND** the field name SHALL be lexed as an identifier token.

#### Scenario: Parse chained field access
- **GIVEN** source `forge current: arcana = character.stats.current_hp;`
- **WHEN** the parser processes the expression
- **THEN** it SHALL emit nested `AST::FieldAccess` nodes: outer access of "current_hp" from the result of inner access of "stats" from "character"
- **AND** evaluation SHALL proceed left-to-right through the chain.

#### Scenario: Field access in expressions
- **GIVEN** source `forge total: arcana = hero.health + ally.health;`
- **WHEN** the parser builds the binary operation
- **THEN** it SHALL emit two `AST::FieldAccess` nodes as the left and right operands of the addition
- **AND** field access SHALL bind tighter than binary operators.

### Requirement: Field Assignment Syntax
The parser SHALL emit `AST::FieldAssignment` nodes for field mutation expressions (`instance.field = value`) with proper precedence and span tracking for diagnostics.

#### Scenario: Parse field assignment
- **GIVEN** source `hero.health = 50;`
- **WHEN** the parser consumes the assignment tokens
- **THEN** it SHALL emit an `AST::FieldAssignment` capturing the target artifact expression, field name, and assigned value expression
- **AND** the target SHALL be any expression that evaluates to an artifact instance.

#### Scenario: Parse chained field assignment
- **GIVEN** source `character.stats.current_hp = 75;`
- **WHEN** the parser handles the nested access
- **THEN** it SHALL emit an `AST::FieldAssignment` where the target is an `AST::FieldAccess` ("character.stats") and the field is "current_hp"
- **AND** evaluation SHALL verify mutability at each level of the chain.

#### Scenario: Reject field assignment on immutable instance
- **GIVEN** source `forge hero: Player = Player { name: "Test", health: 100 };
hero.health = 50;`
- **WHEN** the evaluator attempts the assignment
- **THEN** it SHALL raise an error indicating "hero" is not declared with `morph` and cannot be mutated
- **AND** the parser SHALL accept the syntax but defer mutability checking to evaluation.

### Requirement: Artifact Type Annotations
The parser SHALL recognize artifact type names in variable declarations, function parameters, and return types, emitting `Type::Artifact(String)` variants that store the artifact name for type checking.

#### Scenario: Declare artifact-typed variable
- **GIVEN** source `forge player: Player = Player { name: "Hero", health: 100 };`
- **WHEN** the parser processes the declaration
- **THEN** it SHALL emit a `Type::Artifact("Player")` for the variable's type annotation
- **AND** the evaluator SHALL verify the assigned value matches the artifact type.

#### Scenario: Artifact parameter type
- **GIVEN** source `engrave heal(target: Player, amount: arcana) -> abyss { target.health += amount; }`
- **WHEN** the parser builds the function signature
- **THEN** it SHALL emit a parameter with `Type::Artifact("Player")` and enforce type matching when the function is called.

#### Scenario: Artifact return type
- **GIVEN** source `engrave create_player(name: rune) -> Player { reveal Player { name: name, health: 100 }; }`
- **WHEN** the parser processes the return type
- **THEN** it SHALL emit `Type::Artifact("Player")` as the function's return type
- **AND** the evaluator SHALL validate the revealed value matches this type.
