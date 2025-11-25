# instantiation Specification

## Purpose
TBD - created by archiving change define-spectrum-enum. Update Purpose after archive.
## Requirements
### Requirement: Instantiate Unit Variant
The language MUST support using `SpectrumName::VariantName` to create an instance of a unit variant.

#### Scenario: Creating a Red color
```abyss
spectrum Color { Red, Blue }
forge c: Color = Color::Red;
```

### Requirement: Instantiate Tuple Variant
The language MUST support using `SpectrumName::VariantName(args...)` to create an instance of a tuple variant.

#### Scenario: Creating a MoveCommand
```abyss
spectrum MoveCommand { Up(arcana) }
forge cmd: MoveCommand = MoveCommand::Up(10);
```

