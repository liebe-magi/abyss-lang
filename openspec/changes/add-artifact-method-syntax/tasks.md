## 1. Parser and AST
- [ ] 1.1 Add `Token::Core` and `Token::DoubleColon`, reserving `core` outside identifiers when used as the first parameter in method signatures.
- [ ] 1.2 Update `engrave` parsing to accept `TypeName::method` declarations whose first parameter is `core` or `morph core`, capturing the artifact name and receiver mutability on the AST node.
- [ ] 1.3 Parse `expression.method(args...)` as an `AST::MethodCall`, differentiating it from `AST::FieldAccess` and preserving spans for diagnostics.

## 2. Environment and Evaluator
- [ ] 2.1 Extend artifact definitions to register method metadata (name, mutability, parameter/return types) alongside their schemas.
- [ ] 2.2 Update evaluator dispatch to resolve `AST::MethodCall` by type, implicitly pass the receiver as `core`, and enforce that methods marked `morph core` only run on mutable instances.
- [ ] 2.3 Emit clear runtime errors for unknown methods, mismatched arity, or immutable receivers passed to mutable methods.

## 3. Validation
- [ ] 3.1 Add parser and evaluator tests that cover immutable and mutable method definitions, successful invocations, and failing mutability checks.
- [ ] 3.2 Update documentation/examples to demonstrate defining and calling artifact methods once the feature lands.
