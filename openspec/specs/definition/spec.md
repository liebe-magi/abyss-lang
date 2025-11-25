# definition Specification

## Purpose
TBD - created by archiving change define-spectrum-enum. Update Purpose after archive.
## Requirements
### Requirement: Define Spectrum with Unit Variants
The `spectrum` keyword MUST allow defining a type with multiple named unit variants.

#### Scenario: Defining a Color spectrum
```abyss
spectrum Color {
    Red,
    Blue,
    Green,
}
```

### Requirement: Define Spectrum with Tuple Variants
The `spectrum` keyword MUST allow defining variants that hold data (tuple variants).

#### Scenario: Defining a MoveCommand spectrum
```abyss
spectrum MoveCommand {
    Up(arcana),
    To(arcana, arcana),
    Stop,
}
```

