# Exhaustiveness Checking

## ADDED Requirements

### Requirement: Oracle Exhaustiveness
The `oracle` statement MUST cover all possible values of the scrutinee type.

#### Scenario: Non-exhaustive Omen
Given the code:
```abyss
let x = boon;
oracle x {
    boon => say("Yes");
}
```
When I run the analyzer
Then it reports a "Non-Exhaustive Match" error
And the error message mentions missing "hex"

### Requirement: Primitive Type Catch-All
For primitive types (`Arcana`, `Aether`, `Rune`), the `oracle` statement MUST include a catch-all pattern (`_` or variable binding).

#### Scenario: Missing catch-all for Arcana
Given the code:
```abyss
let x = 42;
oracle x {
    42 => say("Answer");
}
```
When I run the analyzer
Then it reports a "Non-Exhaustive Match" error
And the error message mentions requirement for catch-all

### Requirement: Spectrum Variant Coverage
For `Spectrum` types, the `oracle` statement MUST cover all defined variants OR include a catch-all pattern.

#### Scenario: Missing variant
Given a spectrum "Color" with variants "Red", "Green", "Blue"
And code:
```abyss
let c = Color.Red;
oracle c {
    Color.Red => say("Red");
    Color.Green => say("Green");
}
```
When I run the analyzer
Then it reports a "Non-Exhaustive Match" error
And the error message mentions missing "Blue"
