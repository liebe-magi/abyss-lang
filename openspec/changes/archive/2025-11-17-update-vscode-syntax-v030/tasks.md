## 1. Implementation
- [x] 1.1 Compare the current `keyword`, `type`, `constant`, and `operator` patterns in `editors/code/syntaxes/abyss.tmLanguage.json` against `src/parser/tokens.rs`, then update the grammar so reserved words and builtin functions match the v0.3.0 lexer output.
- [x] 1.2 Add TextMate patterns that recognize artifact constructs (`artifact` declarations, `Type::method` receivers, `core`/`morph core`) and ensure range/arrow/double-colon operators share the same highlighting scope as other arithmetic/logical tokens.
- [x] 1.3 Run the VS Code extension checks (`bun install && bun run check`) and visually verify that README examples display the updated scopes before marking this change complete.
