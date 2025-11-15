## Why
AbySS scripts cannot currently define custom, user-defined data structures to aggregate multiple values under a single named type. Following the implementation of collection types (scroll, lexicon, materia) in v0.2.0, the language still requires developers to use separate variables or nested collections when modeling domain entities like a `Player` with health points and a name. User-defined structs (called `artifact` in AbySS's magical theme) are the necessary next step to enable proper data modeling and are a mandatory prerequisite for future features such as method syntax (e.g., `my_artifact.my_method()`).

## What Changes
- Add the `artifact` keyword to the lexer and grammar for defining custom struct types with named fields and type annotations.
- Extend the AST with `AST::ArtifactDef` for struct definitions and `AST::ArtifactLiteral` for struct instantiation using field-value pairs.
- Introduce field access syntax (`artifact_instance.field_name`) and field assignment syntax for mutable artifact instances.
- Teach the evaluator and environment to store artifact type definitions as schemas and validate field types during instantiation and access.
- Add runtime `Value::Artifact` and `EvalResult::Artifact` variants that carry the artifact's type name and field map.
- Update tests, formatter, and diagnostics to handle the new syntax while maintaining backwards compatibility.

## Impact
- Affected specs: `parser-infrastructure`, `runtime-builtins`
- Affected code: `src/ast.rs`, `src/parser/*`, `src/env.rs`, `src/eval.rs`, `src/types.rs`, formatter and evaluator tests in `tests/`
