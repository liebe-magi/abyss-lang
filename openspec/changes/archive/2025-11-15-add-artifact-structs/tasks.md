## 1. Parser and AST
- [x] 1.1 Add `artifact` keyword token and update lexer to reserve it alongside existing keywords.
- [x] 1.2 Implement `AST::ArtifactDef` node with struct name, field list (name + type pairs), and span information.
- [x] 1.3 Add grammar rules for artifact definition syntax: `artifact TypeName { field1: Type1; field2: Type2; }`.
- [x] 1.4 Implement `AST::ArtifactLiteral` node for instantiation syntax: `TypeName { field1: value1, field2: value2 }`.
- [x] 1.5 Add `AST::FieldAccess` node for dot notation: `instance.field_name`.
- [x] 1.6 Add `AST::FieldAssignment` node for field mutation: `instance.field_name = value`.
- [x] 1.7 Update formatter to pretty-print artifact definitions, literals, and field access expressions.
- [x] 1.8 Add parser diagnostics for malformed artifact definitions and instantiations.

## 2. Type System and Environment
- [x] 2.1 Extend environment to store artifact type schemas with field name-to-type mappings.
- [x] 2.2 Add validation logic for artifact definitions (no duplicate fields, valid type annotations).
- [x] 2.3 Implement type checking for artifact instantiation (all fields present, correct types).
- [x] 2.4 Add support for using artifact types in variable declarations, function parameters, and return types.

## 3. Runtime and Evaluation
- [x] 3.1 Extend `Value` and `EvalResult` with `Artifact` variant containing type name and field-value map.
- [x] 3.2 Implement evaluation for `AST::ArtifactDef` to register the schema in the environment.
- [x] 3.3 Implement evaluation for `AST::ArtifactLiteral` to validate fields and construct runtime artifact values.
- [x] 3.4 Implement evaluation for `AST::FieldAccess` to retrieve field values from artifact instances.
- [x] 3.5 Implement evaluation for `AST::FieldAssignment` with `morph` enforcement for mutable instances.
- [x] 3.6 Add cloning, display, and equality helpers for artifact values.

## 4. Quality Gates
- [x] 4.1 Add parser tests for artifact syntax including edge cases and error scenarios.
- [x] 4.2 Add evaluator tests for artifact definitions, instantiation, field access, and mutation.
- [x] 4.3 Add type system tests for artifact type checking and validation.
- [x] 4.4 Create example `.aby` scripts demonstrating artifact usage.
- [x] 4.5 Run `cargo fmt`, `cargo test`, and ensure all tests pass.
- [x] 4.6 Validate proposal structure and completeness.
