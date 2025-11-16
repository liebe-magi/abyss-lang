## Why
Evaluator-owned builtin method implementations have grown into a monolith. Every scroll, lexicon, or materia method is hardcoded inside `evaluate_builtin_method_call`, forcing the evaluator to understand stdlib semantics and discouraging new method families (rune, arcana, etc.). Moving method logic into the stdlib keeps the evaluator focused on control flow and enables modular extension of builtin behaviors.

## What Changes
- Create a dedicated `stdlib::functions` module that owns global builtin registration (`unveil`, `summon`, etc.).
- Introduce a new `stdlib::methods` tree (with `mod.rs`, `materia.rs`, `scroll.rs`, `lexicon.rs`, etc.) that builds method tables per type and exposes a central dispatcher.
- Update the evaluator to remove the `evaluate_builtin_method_call` match and instead call into the stdlib dispatcher based on the receiver's runtime type.
- Extend the environment / stdlib initialisation to register and store the method registry so runtime code can invoke it uniformly.
- Adjust project documentation/specs to describe the restructured stdlib layout and dispatch responsibilities.

## Impact
- Affected specs: `runtime-builtins` (method registration + dispatch requirements).
- Affected code: `src/stdlib/mod.rs`, new `src/stdlib/functions/*` and `src/stdlib/methods/*` modules, `src/eval/expressions.rs`, `src/env.rs`.
- Tooling: Requires updating OpenSpec validation once new files are authored; no external dependency changes expected.
