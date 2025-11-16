## Why
The current `oracle` statement is the sole conditional construct, but its syntax blurs two distinct control-flow models. Parenthesized forms act like `match` expressions while also permitting inline assignments evaluated like chained `if-else` checks, making intent unclear and blocking future enum exhaustiveness guarantees. We need to separate these semantics while keeping the language free of an `if` keyword.

## What Changes
- Split `oracle` into two explicit modes keyed off the presence of parentheses after the keyword.
- **BREAKING** Remove inline assignments and imperative expressions from the parenthesized form; these scripts must be rewritten using `forge` plus the non-parenthesized mode.
- Teach the parser to emit distinct AST nodes (or flags) per mode and reject the retired syntax.
- Update evaluation logic so `()`-less `oracle` branches behave like `if-else if-else`, while the parenthesized variant evaluates its scrutinee once and pattern-matches.
- Refresh docs, examples, and tests to demonstrate both modes clearly.

## Impact
- Specs: `parser-infrastructure`, `evaluator-infrastructure`.
- Code: `src/parser/grammar.rs`, `src/eval/statements.rs`, `tests/test_oracle.rs`, `examples/oracle.aby`, `README.md`.
