## Why
- Renovate bumped `chumsky` to `0.11` and `ariadne` to `0.6`, and the project must track those minor updates to keep parser and diagnostic dependencies supported.
- Running `cargo build` on the bumped branch fails with unresolved imports (`SimpleReason`, `Stream`), new lifetime requirements on `Parser`, removed combinators like `map_with_span`, and `ariadne::Report::build` signature changes, leaving dozens of compile errors.
- Reviewing the 0.11 guide and migration notes from the chumsky documentation (via Context7) confirms the parser trait now exposes lifetimes on its `Parser` alias and moves `Stream` under `chumsky::input`, while ariadne 0.6 expects spans implementing `ariadne::Span`. Without refactoring our parser and diagnostics layers, we cannot satisfy the dependency update or unblock CI/pre-commit.

## What Changes
- Update `Cargo.toml` and `Cargo.lock` to depend on `chumsky = "0.11"` and `ariadne = "0.6"`, ensuring supporting feature flags align with upstream defaults.
- Refactor the lexer, helper, grammar, and diagnostics modules to the 0.11 API: adopt `chumsky::input::Stream`, add explicit lifetimes to `Parser` aliases, replace removed combinators (`map_with_span`, top-level `filter` helpers), and adjust number literal handling to the new slice-based tokens.
- Rewrite parser error plumbing to the latest `Simple`/`Rich` error APIs, mapping them into our themed diagnostic abstraction without relying on the removed `reason()`/`expected()` helpers.
- Update diagnostic rendering to use `ariadne` 0.6 builders (passing `Range<usize>` spans and named sources) and to ensure span conversions continue to round-trip with our `SimpleSpan` wrapper.
- Refresh or extend tests/repl fixtures so that pre-commit hooks (`cargo fmt`, `cargo clippy`, `cargo test`, formatting checks) succeed with the new dependency versions.

## Impact
- Affected specs: `parser-infrastructure`.
- Affected code: `Cargo.toml`, `Cargo.lock`, `src/parser/{diagnostics,grammar,helpers,mod,tokens}.rs`, supporting AST/eval modules if signatures shift, and the pre-commit configuration.

## PR Notes
- Highlight lifetime adjustments to the boxed parsers and the switch to `IterInput` so reviewers understand why the grammar signatures grew.
- Mention the new diagnostic conversion helper and the need to pass `(source_id, Range<usize>)` into `ariadne::Report::build` to remain compatible with 0.6.
- Call out that lexer/token parsing now collects repeated combinators explicitly, mirroring chumsky 0.11's iterator outputs.
