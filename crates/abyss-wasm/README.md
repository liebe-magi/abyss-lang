# abyss-wasm

Wasm adapter for the [AbySS interpreter](https://abyss-lang.dev). Wraps `abyss-core` and `abyss-interpreter` behind a single `evaluate(source)` entry point for the docs-site Playground.

This crate is **not published to crates.io** — it is a private workspace member built by the docs-site bundler (PR3+ of the v0.6.0 cycle).

## Building

```bash
cargo install wasm-pack   # one-time
wasm-pack build crates/abyss-wasm --target web
```

The output lands in `crates/abyss-wasm/pkg/` and is consumed by the Starlight site under `docs/`.

## Public surface

```ts
type EvalOutcome = {
  stdout: string;
  stderr: string;
  error: string | null;
};

// Throws if the EvalOutcome cannot be serialised to a JS value (very
// rare — only on a serde-wasm-bindgen internal failure). Successful
// runs and ordinary user errors (parser diagnostics, runtime errors)
// resolve normally with `error` populated.
//
// The function is named `evaluate` rather than `eval` because `eval`
// is reserved in strict-mode ES modules and `wasm-bindgen` would
// rename it to `_eval` on the JS side.
function evaluate(source: string): EvalOutcome;
```

`stdout` captures everything `unveil` would print on the CLI. `stderr` carries the parser / runtime diagnostic, ANSI-coloured the same way the CLI prints it (so the playground UI can either render the ANSI codes via xterm.js or strip them for a plain inline message). `error` is non-null when evaluation could not complete; `stdout` may still contain partial output up to the failure point.

`summon` (interactive input) is **not supported** in the playground build — calling it raises a runtime error with a clear message ("summon (interactive input) is not available in the Playground").
