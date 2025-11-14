## 1. Implementation
- [x] 1.1 Scaffold `src/stdlib` with `mod.rs` and `io.rs`, exposing `create_global_environment()` that registers I/O built-ins.
- [x] 1.2 Replace the `Function` struct usages with a `Callable` enum so environments store both engraved and built-in functions uniformly.
- [x] 1.3 Update the evaluator to execute `Callable::Builtin` entries and delete the dedicated `AST::Unveil` / `AST::Summon` code paths.
- [x] 1.4 Remove parser, token, and formatter branches that treated `unveil` and `summon` as special syntax; ensure they parse as ordinary function calls.
- [x] 1.5 Adjust examples/tests for the new `summon` return behaviour and run `cargo fmt` + `cargo test`.
