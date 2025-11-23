# Test Coverage Requirements

## ADDED Requirements

### Requirement: SymbolTable Functionality
The `SymbolTable` MUST support all defined operations including scope management and symbol definition/lookup.
#### Scenario: SymbolTable Operations
-   **Given** a new `SymbolTable`
-   **When** scopes are pushed and popped
-   **Then** symbols should be resolved correctly according to scoping rules.
-   **And** `lookup_mut` should allow mutable access to symbol info.

### Requirement: Evaluator Error Handling
The evaluator MUST gracefully handle invalid operations and return appropriate `EvalError`s.
#### Scenario: Evaluator Error Handling
-   **Given** an invalid expression (e.g., `PowArcana` with negative exponent)
-   **When** it is evaluated
-   **Then** it should return a specific `EvalError`.

### Requirement: Stdlib Validation
Standard library methods MUST validate their arguments and receiver state.
#### Scenario: Stdlib Argument Validation
-   **Given** a stdlib method call with incorrect arguments (count or type)
-   **When** it is evaluated
-   **Then** it should return an `InvalidOperation` or `TypeError`.
-   **And** methods requiring mutable receivers must fail if called on immutable variables.
