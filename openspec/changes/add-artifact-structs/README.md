# Artifact (Struct) Feature Proposal

This OpenSpec change proposal introduces user-defined struct types (called `artifact` in AbySS's magical naming convention) to the language.

## Overview

The artifact feature enables developers to define custom composite data types with named, typed fields. This is a foundational feature for future enhancements like methods and more advanced object-oriented programming patterns.

## Key Features

### 1. Artifact Definition
Define custom types with the `artifact` keyword:
```abyss
artifact Player {
    name: rune;
    health: arcana;
};
```

### 2. Instantiation
Create artifact instances using field-value syntax:
```abyss
forge hero: Player = Player {
    name: "Ardyn",
    health: 100
};
```

### 3. Field Access
Access fields using dot notation:
```abyss
forge hp: arcana = hero.health;
unveil(hero.name);
```

### 4. Field Mutation
Modify fields in mutable artifact instances:
```abyss
forge morph hero: Player = Player { name: "Marcus", health: 90 };
hero.health = 50;
```

### 5. Nested Artifacts
Artifacts can contain other artifacts as fields:
```abyss
artifact Stats { max_hp: arcana; current_hp: arcana; };
artifact Character { name: rune; stats: Stats; };

forge player: Character = Character {
    name: "Theron",
    stats: Stats { max_hp: 200, current_hp: 180 }
};

unveil(player.stats.current_hp); // Chained access
```

### 6. Type System Integration
Artifacts work with the existing type system:
- Variable declarations: `forge hero: Player = ...;`
- Function parameters: `engrave heal(target: Player, amount: arcana) -> abyss { ... }`
- Return types: `engrave create_player() -> Player { ... }`

## Design Decisions

See `design.md` for detailed rationale on:
- Global vs scoped artifact type storage
- Value vs reference semantics
- Mutability model (instance-level, not per-field)
- Nominal typing (name-based type checking)

## Implementation Plan

See `tasks.md` for the complete implementation checklist organized by:
1. Parser and AST changes
2. Type system and environment extensions
3. Runtime and evaluation logic
4. Quality gates and testing

## Specification Deltas

- **parser-infrastructure**: Adds 5 requirements with 15 scenarios covering syntax and AST generation
- **runtime-builtins**: Adds 7 requirements with 26 scenarios covering runtime semantics and evaluation

## Example Usage

See `example.aby` for a comprehensive demonstration of artifact features including:
- Basic definitions and instantiation
- Field access and mutation
- Nested artifacts with chained access
- Function parameters and returns
- Conditional logic with artifacts
- Artifact equality

## Validation

To validate this proposal (once the openspec CLI is available):
```bash
openspec validate add-artifact-structs --strict
```

## Related Work

This proposal builds upon:
- Collection types (v0.2.0): scroll, lexicon, materia
- Parser infrastructure: chumsky-based parsing
- Runtime builtins: existing type system and evaluation

## Future Work

Features intentionally deferred for future proposals:
- Method definitions on artifacts
- Inheritance or trait mechanisms
- Generic artifact types (e.g., `artifact Box<T>`)
- Constructor functions beyond literal syntax
- Field visibility (private/public)
- Recursive artifact types (requires reference semantics)
- Default field values

## Status

**Stage**: Proposal (not yet approved or implemented)

**Approval Required**: Yes - this introduces significant new language features and syntax

**Breaking Changes**: None - this is a purely additive feature
