## Context
AbySS currently supports primitive types and collections but lacks user-defined composite types. The artifact feature introduces struct-like data structures following the magical naming convention. This is the foundation for future object-oriented features like methods.

## Goals / Non-Goals
**Goals:**
- Enable definition of custom types with named, typed fields
- Support instantiation with field-value syntax
- Allow field access and mutation via dot notation
- Integrate with existing type system and environment
- Maintain backwards compatibility

**Non-Goals:**
- Method definitions on artifacts (deferred to future work)
- Inheritance or trait-like mechanisms
- Generic artifact types
- Artifact constructors beyond literal syntax
- Private/public field visibility

## Decisions

### Decision: Artifact Type Storage
**Choice:** Store artifact schemas globally in the environment as a separate symbol table alongside functions and variables.

**Rationale:** Artifact types need to be accessible for type checking during instantiation and assignment. Global storage allows types to be referenced before definition (forward references) and simplifies type resolution.

**Alternatives considered:**
- Scope-based storage: Would complicate type resolution and prevent forward references.
- Merged with variable namespace: Would cause naming conflicts between types and variables.

### Decision: Artifact Value Representation
**Choice:** Store artifacts as `Value::Artifact(String, HashMap<String, Value>)` with the type name and field map.

**Rationale:** Type name enables runtime type checking. HashMap allows efficient field access. Storing fields as Values supports nested artifacts and collections naturally.

**Alternatives considered:**
- Index-based field storage: Would require schema lookup for every access, less intuitive.
- Flattened storage: Would complicate nested artifact handling.

### Decision: Field Mutability Semantics
**Choice:** Artifact field mutation requires the artifact instance variable to be declared with `morph`. Individual fields don't have independent mutability.

**Rationale:** Aligns with existing AbySS mutability model where `morph` controls write access at the variable level. Keeps the language simple by avoiding per-field mutability annotations.

**Alternatives considered:**
- Per-field `morph` declarations: More flexible but adds syntax complexity.
- Always mutable fields: Breaks consistency with immutable-by-default philosophy.

### Decision: Nested Artifact Mutability
**Choice:** When an artifact contains another artifact as a field, mutations propagate if the outermost instance is declared `morph`. Nested artifacts are stored by value (copied), not by reference.

**Rationale:** Value semantics are simpler to reason about and avoid reference lifecycle management. Consistent with how collections work today.

**Alternatives considered:**
- Reference semantics: Would require borrow checking or GC, significant complexity increase.
- Copy-on-write: Optimization that can be added later without breaking semantics.

### Decision: Artifact Type Names in Type System
**Choice:** Extend `Type` enum with `Type::Artifact(String)` variant that stores the artifact name.

**Rationale:** Allows artifact types to participate in type checking for variables, parameters, and returns. Name-based lookup is straightforward.

**Alternatives considered:**
- Separate artifact type enum: Would fragment type system, complicate matching.
- Structural typing: Too complex for initial implementation, no clear syntax.

### Decision: Field Order in Literals
**Choice:** Artifact literals allow fields in any order. The parser accepts any ordering; the evaluator validates against the schema's complete field set.

**Rationale:** Provides flexibility for developers and mirrors how JSON/dictionary literals work. Field order in the definition is preserved for display purposes only.

**Alternatives considered:**
- Mandatory declaration order: Too restrictive, especially for artifacts with many fields.
- Positional field syntax: Less readable, harder to extend.

### Decision: Artifact Equality
**Choice:** Two artifacts are equal if they have the same type name and all field values are equal (recursive comparison).

**Rationale:** Natural semantics for value types. Enables use in conditionals and oracle patterns.

**Alternatives considered:**
- Reference equality: Would require reference semantics (deferred).
- No equality: Would limit artifact usability unnecessarily.

## Risks / Trade-offs

**Risk:** Parser complexity increases with field access chaining and nested artifacts.
**Mitigation:** Leverage existing expression precedence rules. Add comprehensive parser tests for edge cases.

**Risk:** Type system becomes more complex with nominal typing (name-based) for artifacts.
**Mitigation:** Keep structural validation simple. Document type rules clearly. Defer generics and advanced type features.

**Risk:** Performance impact from HashMap field lookups.
**Mitigation:** Most scripts won't have performance-critical artifact access. Profile later if needed. Consider optimization (e.g., field indexing) as future work.

**Risk:** Nested artifact mutations may confuse users due to value semantics.
**Mitigation:** Document behavior clearly with examples. Consider adding warnings or lints for common mistakes in future.

## Migration Plan
No breaking changes to existing syntax or semantics. Artifact is a new feature that doesn't affect existing scripts. Users can adopt incrementally.

## Open Questions
1. **Should artifact definitions be scoped or always global?**
   - Recommendation: Start with global (simplest). Can add scoped artifacts later if needed.

2. **Should artifacts support default field values?**
   - Recommendation: No for initial implementation. Can add in future if demand exists.

3. **Should the formatter respect user-specified field order in literals?**
   - Recommendation: Yes, preserve source order as it may carry semantic meaning to the developer.

4. **How should recursive artifact types be handled (e.g., linked lists)?**
   - Recommendation: Not supported in initial version as it requires reference types. Document as limitation.
