# Design: Static Analysis & Exhaustiveness Checking

## Architecture

The responsibility for static analysis and type checking will be moved from `abyss-interpreter` (runtime) to `abyss-core` (definition/analysis time).

### Current State
- **Type Definitions:** `SpectrumSchema` and `ArtifactSchema` reside in `abyss-interpreter/src/env.rs`.
- **Checking:** `eval/statements.rs` performs checks at runtime using `RuntimeEnv`.
- **Issue:** External tools (LSP, Compiler) cannot access type information without executing code.

### Target State
- **Type Definitions:** Schema definitions moved to `abyss-core`.
- **Analyzer:** New `SemanticAnalyzer` component in `abyss-core`.
- **Flow:** `Source -> Parser -> AST -> Analyzer (Check!) -> Interpreter / Compiler`

## Data Structure Reorganization (`abyss-core`)

### Schema Migration
Move type definition information from `interpreter` to `core`, removing dependencies on `Value` (runtime values).

- **Target:** `crates/abyss-core/src/types.rs`
- **Structs:** `SpectrumSchema`, `ArtifactSchema`

### SymbolTable Expansion
Extend `SymbolTable` to hold user-defined type definitions (Spectrum/Artifact).

```rust
pub enum SymbolKind {
    Variable(Type, bool),
    Spectrum(SpectrumSchema),
    Artifact(ArtifactSchema),
    Function(FunctionSignature),
}
```

## Static Analyzer (`abyss-core/src/analysis.rs`)

Implement an `Analyzer` that traverses the AST, builds the symbol table, and validates code.

### Analysis Flow
1.  **Scope Enter:** Create new scope.
2.  **Declaration Pass:** Register `spectrum`, `artifact`, `engrave` definitions. Check for duplicates.
3.  **Statement Pass:** Validate `oracle`, `assign` statements.
    - Check variable existence.
    - Check type compatibility.
    - **Check exhaustiveness.**
4.  **Scope Exit:** Destroy scope.

## Exhaustiveness Check Logic

The `check_exhaustiveness` method ensures `oracle` statements cover all possible cases.

### Primitive Types
Must use a wildcard (`_`) or variable binding (`x`) as a catch-all.
- **Omen (bool):** `boon` + `hex` OR catch-all.
- **Arcana (int), Aether (float), Rune (string):** Catch-all required.

### Spectrum Types
Must cover all defined variants OR use a catch-all.
- **Logic:**
    1. Check for catch-all. If present, OK.
    2. Fetch `SpectrumSchema` from `SymbolTable`.
    3. Compare defined variants with patterns in branches.
    4. Report missing variants.
