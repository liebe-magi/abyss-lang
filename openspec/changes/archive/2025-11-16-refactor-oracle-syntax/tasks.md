## 1. Parser (parser-infrastructure)
- [x] 1.1 Update `src/parser/grammar.rs` so `oracle` parsing branches on whether the keyword is followed by `(` or `{`, mapping to distinct AST modes.
- [x] 1.2 Ensure the `if-else` mode (no parentheses) parses each branch guard as an expression.
- [x] 1.3 Keep the `match` mode (with parentheses) parsing branch left sides as patterns bound to the scrutinee value.
- [x] 1.4 Remove legacy support for inline assignments or bindings inside the parenthesized `oracle (...)` syntax and emit a syntax error instead.

## 2. Evaluator (evaluator-infrastructure)
- [x] 2.1 Adjust `src/eval/statements.rs` to evaluate `oracle` nodes differently based on the parsed mode marker.
- [x] 2.2 Implement the `if-else` mode semantics: evaluate each branch expression in order until one returns `boon`, using `_` as the fallback.
- [x] 2.3 Preserve and harden the `match` mode semantics: evaluate the scrutinee exactly once, then match against each pattern.
- [x] 2.4 Delete any evaluator logic that performs variable binding or assignment inside the parenthesized syntax.

## 3. Codebase & Documentation (BREAKING)
- [x] 3.1 Update `tests/test_oracle.rs` to stop using `oracle (a = ...)` and instead combine `forge` bindings with the new `oracle { ... }` syntax.
- [x] 3.2 Refresh `examples/oracle.aby` to showcase both `if-else` and `match` modes explicitly.
- [x] 3.3 Rewrite the `README.md` Conditionals section so it contrasts the two oracle modes, explains their use cases, and notes the removed syntax.
