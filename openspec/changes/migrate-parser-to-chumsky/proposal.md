## Why
The existing PEG-based parser (`pest` + `build_ast`) has become brittle and hard to extend. Operator precedence rules are difficult to reason about, the `build_ast` function is a maintenance hotspot, and syntax errors surface with minimal guidance. We need a parser architecture that is easier to evolve alongside new language features while delivering first-class diagnostics that reinforce the AbySS theme.

## What Changes
- Replace the `pest` grammar (`src/abyss.pest`) with a `chumsky` parser written in Rust code.
- Rebuild `src/parser.rs` around `chumsky` combinators so AST construction happens in one pass with strong typing and shared span metadata.
- Integrate `ariadne` as the structured error reporting backend and surface themed diagnostics for parse failures.
- Preserve the existing `AST` contract and evaluation pipeline so current behaviour and tests remain unchanged.

## Impact
- Affected specs: `parser-infrastructure`
- Affected code: `src/parser.rs`, removal of `src/abyss.pest`, new error-reporting utilities, updates to supporting modules/tests if they rely on parser APIs.
- Tooling: `Cargo.toml` gains `chumsky` and `ariadne` dependencies; CI remains Rust-based.
