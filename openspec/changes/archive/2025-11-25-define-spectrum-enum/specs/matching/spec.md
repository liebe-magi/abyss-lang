# Spec: Spectrum Matching

## ADDED Requirements

### Requirement: Match Unit Variant
The `oracle` statement MUST support matching against unit variants.

#### Scenario: Matching Color
```abyss
spectrum Color { Red, Blue }
forge c: Color = Color::Red;
oracle (c) {
    (Color::Red) => { unveil("It is Red"); }
    (Color::Blue) => { unveil("It is Blue"); }
}
```

### Requirement: Match Tuple Variant with Destructuring
The `oracle` statement MUST support matching against tuple variants and binding their values to variables.

#### Scenario: Matching MoveCommand
```abyss
spectrum MoveCommand { Up(arcana), Stop }
forge cmd: MoveCommand = MoveCommand::Up(10);
oracle (cmd) {
    (MoveCommand::Up(dist)) => { unveil("Moving up", dist); }
    (MoveCommand::Stop) => { unveil("Stopping"); }
}
```

### Requirement: Exhaustiveness Check
When matching a `spectrum`, the `oracle` statement MUST ensure all variants are covered or a wildcard `_` is present. If not, it MUST raise a compile-time error.

#### Scenario: Missing Variant Error
```abyss
spectrum Color { Red, Blue, Green }
forge c: Color = Color::Red;
// Error: Non-exhaustive patterns: `Green` not covered
oracle (c) {
    (Color::Red) => { ... }
    (Color::Blue) => { ... }
}
```
