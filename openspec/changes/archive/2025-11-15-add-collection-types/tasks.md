## 1. Parser and AST
- [x] 1.1 Add `scroll`, `lexicon`, and `materia` token/Type variants plus formatter support.
- [x] 1.2 Implement `AST::ListLiteral`, `AST::MapLiteral`, `AST::IndexAccess`, and `AST::IndexAssignment` along with grammar rules and diagnostics.

## 2. Runtime and Stdlib
- [x] 2.1 Extend `Value`/`EvalResult` with `Scroll` and `Lexicon`, including cloning, display, and equality helpers.
- [x] 2.2 Implement evaluation for collection literals, indexing, and indexed assignment with `morph` enforcement and `Type::Materia` handling.
- [x] 2.3 Register `measure`, `inscribe`, `retract`, `expunge`, and `contents` builtins in `stdlib` with full argument validation.

## 3. Quality Gates
- [x] 3.1 Add parser, evaluator, and stdlib tests covering mixed-type lists, rune-keyed maps, and error cases.
- [x] 3.2 Run `cargo fmt`, `cargo test`, and `openspec validate add-collection-types --strict`.
