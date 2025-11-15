## Context
AbySS presently stores `Value::Scroll(Vec<Value>)`, `Value::Lexicon(HashMap<_, _>)`, and `Value::Rune(String)` inline. Every assignment or function argument clones the entire structure, causing quadratic slowdowns, diverging from user expectations of reference semantics, and preventing GC since no shared heap objects exist. The interpreter is single-threaded, so `Rc<RefCell<_>>` fits the borrowing model without introducing `Send` constraints.

## Goals / Non-Goals
- Goals:
	- Share heap-backed values via `Rc` handles so aliases observe mutations.
	- Keep a single runtime data type by wrapping all concrete values inside `EvalResult::Data`.
	- Establish the foundation for future tracing/mark-sweep GC by ensuring containers already live on the heap with shared ownership metadata.
- Non-Goals:
	- Implement GC or cycle detection in this change.
	- Introduce multi-threading or `Arc` support (interpreter remains single-threaded).

## Decisions
- Decision: Wrap `String`, `Vec<Value>`, and `HashMap<String, Value>` in `Rc` (and `RefCell` for mutable containers) so clones only increment refcounts, matching dynamic-language expectations.
- Decision: Collapse `EvalResult` data variants into `EvalResult::Data(Value)` while retaining dedicated variants for control-flow constructs to simplify evaluation pipelines and reduce duplication.
- Decision: Update stdlib builtins and index helpers to borrow via `RefCell` instead of replacing entire containers, ensuring alias-safe semantics and enabling future GC instrumentation around borrow points.

## Risks / Trade-offs
- `RefCell` borrow checking happens at runtime; violating borrowing rules will panic. Careful review and tests are required to avoid overlapping mutable borrows.
- `Rc` introduces reference cycles risk (e.g., scroll holding lexicon referencing scroll). Without GC, these leaks persist; change should document this limitation until a collector ships.
- Sharing values increases mutation visibility; some scripts might have relied on copy semantics. The proposal is intentionally breaking but documented via spec update.

## Migration Plan
1. Change `Value`/`EvalResult` definitions and adjust helper constructors.
2. Update evaluator patterns across `src/eval/statements.rs`, `src/eval/expressions.rs`, and helper conversions in `src/eval/values.rs` to allocate shared handles and unwrap `EvalResult::Data` where necessary.
3. Refactor index helpers in `src/eval/collections.rs` and stdlib builtins to borrow through `RefCell` for reads/writes.
4. Update REPL / IO formatting plus tests to use the new `EvalResult` shape.
5. Document behavior in release notes and ensure new spec deltas are approved before implementation continues.

## Open Questions
- Should we provide debugging utilities (e.g., `measure_rc(bag)`) to introspect reference counts for troubleshooting shared-state bugs?
- How will we migrate from `Rc<RefCell<_>>` to a full GC—can we reuse the handles or should we plan an adapter layer now?
