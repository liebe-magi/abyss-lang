# Design: Static Analysis & Exhaustiveness Checking

## 1. Architecture Overview

We will completely migrate the responsibility of static analysis from `abyss-interpreter` (runtime) to `abyss-core` (definition/analysis time).

### Current State

*   **Type Definitions:** `SpectrumSchema` and `ArtifactSchema` are located in `env.rs` of `abyss-interpreter`.
*   **Checking:** `eval/statements.rs` performs checks at runtime using `RuntimeEnv`.
*   **Issues:** Tools like LSP or compilers cannot access type information without executing the code.

### Target State

*   **Type Definitions:** Move schema definitions to `abyss-core`.
*   **Analyzer:** Create a new `SemanticAnalyzer` in `abyss-core`.
*   **Flow:** `Source -> Parser -> AST -> **Analyzer (Check!)** -> Interpreter / Compiler`

## 2. Data Structure Reorganization (`abyss-core`)

### 2.1. Moving Schema Definitions

Move type definition information from `interpreter` to `core`, eliminating the dependency on `Value` (runtime value).

*   **Move Targets:** `ArtifactSchema`, `SpectrumSchema`
*   **Destination:** `crates/abyss-core/src/types.rs` (or `schema.rs`)

```rust
// crates/abyss-core/src/types.rs

#[derive(Debug, Clone, PartialEq)]
pub struct SpectrumSchema {
    pub name: String,
    pub variants: HashMap<String, Vec<Type>>, // Variant Name -> List of Argument Types
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactSchema {
    pub name: String,
    pub fields: HashMap<String, Type>,
}
```

### 2.2. Extending `SymbolTable`

Extend the current `SymbolTable` to hold definition information for user-defined types (Spectrum/Artifact).

```rust
// crates/abyss-core/src/semantic.rs

pub enum SymbolKind {
    Variable(Type, bool), // (Type, Mutable?)
    Spectrum(SpectrumSchema),
    Artifact(ArtifactSchema),
    Function(FunctionSignature),
}

pub struct SymbolTable {
    // Manage scope for type definitions and function definitions in addition to variables
    scopes: Vec<HashMap<String, SymbolKind>>,
}
```

## 3. Implementing Static Analyzer (`abyss-core/src/analysis.rs`)

Implement an `Analyzer` that traverses the AST, builds the symbol table, and performs verification.

### `analyze` Function Flow

1.  **Scope Enter:** Create a new scope.
2.  **Declaration Pass (Collecting Declarations):**
    *   When definitions like `spectrum`, `artifact`, `engrave` are found, register them in `SymbolTable`.
    *   Report an error if there are duplicate definitions.
3.  **Statement Pass (Verifying Statements):**
    *   Verify the contents of statements like `oracle`, `assign`.
    *   Is the variable defined? (`SymbolTable` lookup)
    *   Do the types match? (`Type` mismatch check)
    *   **Is exhaustiveness satisfied? (`Exhaustiveness Check`)**
4.  **Scope Exit:** Destroy the scope.

```rust
// crates/abyss-core/src/analysis.rs

pub struct Analyzer {
    symbols: SymbolTable,
    errors: Vec<AnalysisError>,
}

impl Analyzer {
    pub fn analyze(&mut self, ast: &AST) {
        match ast {
            AST::SpectrumDef { name, variants, .. } => {
                // Register definition in symbol table
                self.register_spectrum(name, variants);
            },
            AST::Oracle { conditionals, branches, is_match, .. } => {
                // 1. Infer types of the expression
                let scrutinee_types = self.infer_types(conditionals);
                
                // 2. Exhaustiveness check
                if *is_match {
                    self.check_exhaustiveness(&scrutinee_types, branches);
                }
                
                // 3. Recursively analyze branch contents
                // ...
            },
            // ...
        }
    }
}
```

## 4. Detailed Specification of Exhaustiveness Checking

This is the core part of this proposal. We define the logic for the `check_exhaustiveness` method.

### Algorithm Overview

Define a set of required patterns based on the type of the scrutinee, and check if they exist in the branches.

### 4.1. Primitive Type Check (New Requirement)

For primitive types, enumerating all patterns is impossible (or impractical), so we require a **Catch-All using a Wildcard (`_`) or Variable Binding (`x`)**.

| Type (`Type`) | Exhaustiveness Condition (Must satisfy one) |
| :--- | :--- |
| **`Omen` (bool)** | ① Both `boon` and `hex` exist.<br>② `_` (Wildcard) or Variable Binding exists. |
| **`Arcana` (int)** | ① `_` or Variable Binding exists.<br>*(Future: Covering the full range `min..=max` might be allowed, but v1 requires wildcard)* |
| **`Aether` (float)** | ① `_` or Variable Binding exists. |
| **`Rune` (string)** | ① `_` or Variable Binding exists. |

### 4.2. Spectrum Type Check (Transfer of Existing Logic)

Retrieve `SpectrumSchema` from `SymbolTable` and check if all defined variants are covered.

*   **Conditions:**
    1.  `_` or Variable Binding exists (Immediate OK).
    2.  OR, all defined `VariantName`s exist in the patterns.

### 4.3. Implementation Image

```rust
fn check_exhaustiveness(&mut self, type_: &Type, branches: &[AST]) {
    // 1. Check for wildcard/variable binding
    let has_catch_all = branches.iter().any(|b| is_catch_all_pattern(&b.pattern));
    if has_catch_all {
        return; // OK: Fully exhaustive
    }

    match type_ {
        Type::Omen => {
            // Check for both boon and hex
            let has_boon = branches.iter().any(|b| is_literal(b, true));
            let has_hex = branches.iter().any(|b| is_literal(b, false));
            if !has_boon || !has_hex {
                self.errors.push(AnalysisError::NonExhaustiveMatch("Omen must cover boon and hex or use _"));
            }
        },
        Type::Arcana | Type::Aether | Type::Rune => {
            // NG if no Catch-all
            self.errors.push(AnalysisError::NonExhaustiveMatch("Primitive type must use _ or variable binding"));
        },
        Type::Spectrum(name) => {
            // Get all defined variants
            let schema = self.symbols.lookup_spectrum(name).unwrap();
            let all_variants: HashSet<_> = schema.variants.keys().collect();
            
            // Get variants present in patterns
            let covered: HashSet<_> = branches.iter().flat_map(|b| get_variants(b)).collect();
            
            // Error if there is a difference
            let missing: Vec<_> = all_variants.difference(&covered).collect();
            if !missing.is_empty() {
                self.errors.push(AnalysisError::NonExhaustiveMatch(format!("Missing variants: {:?}", missing)));
            }
        },
        _ => {}, // Other types are currently not checked or error
    }
}
```

## 5. Roadmap

We recommend implementing in the following order:

1.  **Schema Migration:** Move `ArtifactSchema`, `SpectrumSchema` to `core`. Modify `interpreter` to reference them.
2.  **SymbolTable Expansion:** Extend `SymbolTable` in `core` to store type definitions and function signatures.
3.  **Analyzer Scaffold:** Create `Analyzer` struct in `abyss-core` and build the framework for traversing AST.
4.  **Exhaustiveness Logic:** Implement the logic described in "4." into `Analyzer`.
5.  **Integration:** Remove runtime checks in `interpreter` (`eval/statements.rs`) and change to pass through `Analyzer` before execution (or connect via CLI).

With this design, LSP can simply call `Analyzer` in `abyss-core` to display type errors and exhaustiveness errors in the editor in real-time.
