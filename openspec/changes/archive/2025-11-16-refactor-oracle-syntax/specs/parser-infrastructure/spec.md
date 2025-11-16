## ADDED Requirements
### Requirement: Oracle statements support `if-else` and `match`
The parser SHALL recognize the `oracle` keyword and treat the token following it as the mode selector.
1.  If the token after `oracle` is `{` (no parentheses), the parser SHALL treat the construct as `if-else` mode and parse each branch head as a guard expression evaluated from top to bottom.
2.  If the token after `oracle` is `(`, the parser SHALL treat the construct as `match` mode, parsing the parenthesized expression as the scrutinee and each branch head as a pattern list.

#### Scenario: Parse if-else mode (no parentheses)
- **GIVEN** `oracle { (x > 0) => ...; _ => ...; }`
- **WHEN** the parser processes the statement
- **THEN** it SHALL treat the construct as `if-else` mode and parse `(x > 0)` as a guard expression.

#### Scenario: Parse match mode (with parentheses)
- **GIVEN** `oracle (x) { (1) => ...; _ => ...; }`
- **WHEN** the parser processes the statement
- **THEN** it SHALL treat `x` as the scrutinee expression and `(1)` as a branch pattern.

### Requirement: Oracle parentheses reject variable binding
The parser SHALL emit a diagnostic error whenever it encounters an assignment expression (e.g., `a = ...`) inside the parentheses immediately following `oracle`.

**Reason**: The inline binding form looked like match mode but behaved like if-else mode, which was confusing; `forge` statements and the if-else mode `oracle` replace it.

#### Scenario: Removed syntax
- **GIVEN** `oracle (z = y * 2) { (z > 50) => ...; }`
- **WHEN** the parser processes the statement
- **THEN** it SHALL raise a syntax error because variable binding inside the parentheses is no longer permitted.
