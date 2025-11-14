## Why
`unveil` and `summon` bypass the shared function-call machinery today, which complicates the evaluator, spreads I/O behaviour across multiple modules, and makes it hard to expand the set of built-ins. Aligning them with the standard function pipeline lets us register built-ins consistently and simplifies future extensions to the standard library.

## What Changes
- Refactor the runtime environment to treat engraved (user-defined) and built-in functions via a shared `Callable` abstraction.
- Introduce a `stdlib` module that seeds the global environment with I/O built-ins implemented in Rust.
- Route `unveil` and `summon` through `AST::FuncCall`, removing their bespoke AST nodes, parser branches, and formatter logic. **BREAKING**: `summon` will always return a `rune`, so callers must cast when other types are required.

## Impact
- Affected specs: runtime-builtins
- Affected code: src/env.rs, src/eval.rs, src/ast.rs, src/parser/*, src/format.rs, src/main.rs, examples/summon.aby, tests/*
