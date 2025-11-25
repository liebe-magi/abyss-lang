# Static Analysis

## ADDED Requirements

### Requirement: Static Analysis Phase
The system MUST perform static analysis on the AST before execution or compilation.

#### Scenario: Analysis before execution
Given a source file "example.abyss"
When I run "abyss run example.abyss"
Then the system parses the source
And the system runs the Semantic Analyzer
And if analysis fails, execution stops with errors
And if analysis succeeds, execution proceeds

### Requirement: Variable Validation
The Analyzer MUST verify that all referenced variables are defined in the current scope.

#### Scenario: Undefined variable
Given the code:
```abyss
engrave main {
    say(undefined_var);
}
```
When I run the analyzer
Then it reports an "Undefined Variable" error for "undefined_var"

### Requirement: Type Validation
The Analyzer MUST verify that operations are performed on compatible types.

#### Scenario: Type mismatch
Given the code:
```abyss
engrave main {
    let x = 10;
    let y = "string";
    let z = x + y;
}
```
When I run the analyzer
Then it reports a "Type Mismatch" error
