## ADDED Requirements
### Requirement: Core Keyword and Method Receiver Tokens
The parser SHALL treat `core` as a reserved keyword when it appears as the first parameter of an `engrave` definition for artifact methods, lexing it via a dedicated `Token::Core`. The lexer SHALL also emit `Token::DoubleColon` for `::` so type-qualified method names can be parsed without multi-character lookahead hacks.

#### Scenario: Tokenize core receiver
- **GIVEN** source `engrave Player::heal(core, amount: arcana) -> abyss { ... }`
- **WHEN** the lexer processes the first parameter
- **THEN** it SHALL emit `Token::Core` so the parser can enforce receiver semantics instead of treating `core` as a generic identifier.

#### Scenario: Tokenize morph core receiver
- **GIVEN** source `engrave Player::set_level(morph core, value: arcana) -> abyss { ... }`
- **WHEN** the lexer consumes the `morph core` sequence
- **THEN** it SHALL emit the existing `Token::Morph` followed by `Token::Core`, preserving span information so the parser can flag invalid placements.

#### Scenario: Tokenize double colon
- **GIVEN** source `engrave Player::get_level(core) -> arcana { ... }`
- **WHEN** the lexer encounters `::`
- **THEN** it SHALL emit `Token::DoubleColon` ensuring the parser distinguishes `Player::get_level` from other identifier sequences.

### Requirement: Artifact Method Definition Syntax
The parser SHALL extend `AST::Engrave` parsing to accept `TypeName::method` signatures whose first parameter is `core` or `morph core`, record the artifact type and receiver mutability on the AST node, and reject definitions that omit or rename the receiver.

#### Scenario: Parse immutable artifact method
- **GIVEN** source `engrave Player::get_level(core) -> arcana { reveal core.level; }`
- **WHEN** the parser processes the declaration
- **THEN** it SHALL emit an `AST::Engrave` tagged as a method, storing the artifact name `Player`, the method identifier `get_level`, and a receiver marked immutable.

#### Scenario: Parse mutable artifact method
- **GIVEN** source `engrave Player::set_level(morph core, next: arcana) -> abyss { core.level = next; }`
- **WHEN** the parser handles the signature
- **THEN** it SHALL emit an `AST::Engrave` that records the receiver as mutable (`morph core`) so evaluation can enforce write access.

#### Scenario: Reject missing core receiver
- **GIVEN** source `engrave Player::heal(target: Player, amount: arcana) -> abyss { ... }`
- **WHEN** the parser validates the parameter list
- **THEN** it SHALL raise a diagnostic indicating artifact methods must declare `core` (or `morph core`) as the first argument, preventing accidental free functions from being tagged as methods.

### Requirement: Artifact Method Invocation Syntax
The parser SHALL recognize `expression.identifier(args...)` as a method call whenever parentheses follow the identifier, emitting `AST::MethodCall` nodes that capture the receiver expression, method name, argument list, and spans distinct from `AST::FieldAccess`.

#### Scenario: Parse simple method call
- **GIVEN** source `p.set_level(11);`
- **WHEN** the parser reaches the dot-expression with parentheses
- **THEN** it SHALL emit an `AST::MethodCall` whose receiver is the identifier `p`, method name `set_level`, and arguments contain the literal `11`.

#### Scenario: Parse chained method calls
- **GIVEN** source `party.leader().promote("captain");`
- **WHEN** the parser processes both dot-call segments
- **THEN** it SHALL emit nested `AST::MethodCall` nodes such that the second call (`promote`) receives the first call (`party.leader()`) as its receiver, preserving evaluation order.

#### Scenario: Disambiguate field access vs call
- **GIVEN** source `forge lvl: arcana = hero.level; hero.get_level();`
- **WHEN** the parser encounters the dot followed by an identifier
- **THEN** it SHALL emit `AST::FieldAccess` for `hero.level` because no parentheses follow, and `AST::MethodCall` for `hero.get_level()` because parentheses are present, ensuring both constructs co-exist.
