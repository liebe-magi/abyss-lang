## 1. Parser and AST
- [ ] 1.1 Add `artifact` keyword token and update lexer to reserve it alongside existing keywords.
- [ ] 1.2 Implement `AST::ArtifactDef` node with struct name, field list (name + type pairs), and span information.
- [ ] 1.3 Add grammar rules for artifact definition syntax: `artifact TypeName { field1: Type1; field2: Type2; }`.
- [ ] 1.4 Implement `AST::ArtifactLiteral` node for instantiation syntax: `TypeName { field1: value1, field2: value2 }`.
- [ ] 1.5 Add `AST::FieldAccess` node for dot notation: `instance.field_name`.
- [ ] 1.6 Add `AST::FieldAssignment` node for field mutation: `instance.field_name = value`.
- [ ] 1.7 Update formatter to pretty-print artifact definitions, literals, and field access expressions.
- [ ] 1.8 Add parser diagnostics for malformed artifact definitions and instantiations.

## 2. Type System and Environment
- [ ] 2.1 Extend environment to store artifact type schemas with field name-to-type mappings.
- [ ] 2.2 Add validation logic for artifact definitions (no duplicate fields, valid type annotations).
- [ ] 2.3 Implement type checking for artifact instantiation (all fields present, correct types).
- [ ] 2.4 Add support for using artifact types in variable declarations, function parameters, and return types.

## 3. Runtime and Evaluation
- [ ] 3.1 Extend `Value` and `EvalResult` with `Artifact` variant containing type name and field-value map.
- [ ] 3.2 Implement evaluation for `AST::ArtifactDef` to register the schema in the environment.
- [ ] 3.3 Implement evaluation for `AST::ArtifactLiteral` to validate fields and construct runtime artifact values.
- [ ] 3.4 Implement evaluation for `AST::FieldAccess` to retrieve field values from artifact instances.
- [ ] 3.5 Implement evaluation for `AST::FieldAssignment` with `morph` enforcement for mutable instances.
- [ ] 3.6 Add cloning, display, and equality helpers for artifact values.

## 4. Quality Gates
- [ ] 4.1 Add parser tests for artifact syntax including edge cases and error scenarios.
- [ ] 4.2 Add evaluator tests for artifact definitions, instantiation, field access, and mutation.
- [ ] 4.3 Add type system tests for artifact type checking and validation.
- [ ] 4.4 Create example `.aby` scripts demonstrating artifact usage.
- [ ] 4.5 Run `cargo fmt`, `cargo test`, and ensure all tests pass.
- [ ] 4.6 Validate proposal structure and completeness.
