## ADDED Requirements
### Requirement: Oracle statement evaluation semantics
The evaluator SHALL execute `oracle` statements in one of two modes based on the AST marker emitted by the parser, and it SHALL branch on that mode marker to select the semantics.

#### Scenario: Evaluate if-else mode (no parentheses)
- **GIVEN** an `oracle { ... }` AST without parentheses after the keyword
- **WHEN** the evaluator processes the statement
- **THEN** it SHALL evaluate each branch guard expression from top to bottom, execute the first branch whose guard yields `boon`, and exit the statement after running that block.

#### Scenario: Evaluate match mode (with parentheses)
- **GIVEN** an `oracle (<expr>) { ... }` AST whose parentheses wrap the scrutinee expression
- **WHEN** the evaluator processes the statement
- **THEN** it SHALL evaluate the scrutinee expression exactly once, cache the resulting value, and compare it against each branch pattern according to the pattern semantics until a matching branch executes.

**Reason**: Removing the inline mutation syntax keeps the `oracle` evaluation modes clearly separated.
**Reason**: Eliminating the ambiguous inline binding form is necessary; `forge` statements and if-else mode `oracle` serve as the replacement.
The evaluator SHALL treat any legacy AST node that encodes inline scrutinee bindings as invalid and MUST report an error rather than executing it.

#### Scenario: Removed behavior
- **GIVEN** an AST for `oracle (z = y * 2) { ... }`
- **WHEN** the evaluator inspects the node
- **THEN** it SHALL reject the node (or raise an error) because inline bindings inside the parentheses are no longer allowed.
