//! Wasm-bindgen bridge functions exposed to JavaScript.
//!
//! Kept in a dedicated module so the [`crate::run`] core (which is what
//! the unit tests actually exercise) stays separate from the
//! `JsValue`-touching wrappers that can only execute under a real wasm
//! runtime. `codecov.yml` ignores this file because the wrapper bodies
//! are unreachable from native `cargo test`.

use wasm_bindgen::prelude::*;

use crate::run;

/// Initialise panic hooks once per Wasm instance so a Rust panic gets
/// surfaced to the browser console instead of an opaque
/// `RuntimeError: unreachable executed`.
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

/// Evaluate `source` and return the captured stdout, the rendered
/// diagnostic (if any), and an `error` string when evaluation could not
/// complete. The function is exposed to JavaScript as a plain function
/// taking a string and returning an `EvalOutcome` JSON object.
///
/// Named `evaluate` rather than `eval` because `eval` is reserved in
/// strict-mode ES modules and `wasm-bindgen` renames it to `_eval` on
/// the JS side, which is awkward for hand-written callers. `evaluate`
/// reaches the browser unmangled.
#[wasm_bindgen]
pub fn evaluate(source: String) -> Result<JsValue, JsValue> {
    let outcome = run(&source);
    serde_wasm_bindgen::to_value(&outcome).map_err(|err| JsValue::from_str(&err.to_string()))
}
