## Why
`src/eval.rs` has grown past 1,200 lines and now mixes value conversion helpers, collection indexing logic, expression evaluation, control-flow machinery, and statement handling inside a single file. The size and coupling make reviewer load high and block upcoming behavioural work (collection aliasing, REPL features) because any change destabilises the entire evaluator.

## What Changes
- Split the evaluator into a dedicated `eval` module directory (`src/eval/`) with focused submodules for results, values/conversions, collections/indexing helpers, expressions, and statements/control flow.
- Update `lib.rs`, `main.rs`, stdlib helpers, and tests to use the new module paths while keeping public APIs (`evaluate`, `display_error_with_source`, `EvalResult`, `EvalError`) stable.
- Document the architecture requirement via a new `evaluator-infrastructure` capability ensuring future work preserves the modular boundaries.
- Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` to ensure the refactor is mechanical and does not change runtime behaviour.

## Impact
- Affected specs: `evaluator-infrastructure` (new capability)
- Affected code: `src/eval.rs`, `src/lib.rs`, `src/main.rs`, `src/stdlib/*`, `tests/*`
