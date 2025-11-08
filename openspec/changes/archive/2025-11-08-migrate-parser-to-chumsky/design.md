## Context
AbySS currently relies on a `pest`-authored PEG grammar (`src/abyss.pest`) that feeds a large `build_ast` transformer. Operator precedence and themed syntax have stretched the PEG approach, making evolution costly and errors hard to understand. Future roadmap items (collections, structs, modules) will further complicate the grammar, so we need a parser that keeps AST construction and diagnostics within Rust's type system.

## Goals / Non-Goals
- Goals:
  - Express the entire grammar using `chumsky` parser combinators with explicit spans for every AST node.
  - Produce themed, high-fidelity parse diagnostics via `ariadne` that can later be localised or further embellished.
  - Maintain backwards compatibility for the public `parse` API and the `AST` contract consumed by `eval` and downstream tooling.
- Non-Goals:
  - Changing the language syntax or semantics beyond what is necessary to mirror existing behaviour.
  - Refactoring the evaluator, environment, or runtime error handling.
  - Introducing incremental parsing or performance tuning beyond parity with `pest`.

## Decisions
- Decision: Use `chumsky`'s `recursive` combinators and precedence helpers to encode expression trees, ensuring right-associative exponentiation and existing operator precedences are preserved.
  - Alternatives considered: `lalrpop` (requires separate grammar files and code generation), hand-written recursive descent (more boilerplate, harder to extend).
- Decision: Introduce an internal `ParserDiagnostic` enum that captures high-level error kinds (unexpected token, unterminated string, missing terminator, etc.) before formatting with `ariadne`.
  - Alternatives considered: Rendering `chumsky::error::Simple` directly; rejected due to insufficient control over theming and future localisation hooks.
- Decision: Continue returning `Vec<AST>` from `parse` while adding a companion `ParseOutcome` type that bundles ASTs and diagnostics, allowing CLI callers to decide whether to proceed on recoverable errors.
  - Alternatives considered: Hard failure on first error; rejected because the REPL benefits from best-effort parsing.

## Risks / Trade-offs
- `chumsky`'s ergonomics differ from PEG; the team needs familiarity to avoid regressions. Mitigation: pair programming sessions and thorough comments in the new parser module.
- Error span fidelity depends on correctly mapping byte offsets to `LineInfo`. Mitigation: centralise span-to-line conversion utilities and add unit tests around tricky cases (multibyte characters, multiline literals).
- Removing `pest` eliminates declarative grammar files that were easy to skim. Mitigation: document the grammar structure within the parser module (sectioned functions, module-level overview).

## Migration Plan
1. Establish the new parser module alongside the existing one, gated behind a feature flag if necessary for incremental testing.
2. Port literal and identifier parsing, verifying unit tests before moving to expressions.
3. Layer expression precedence from the bottom up (`atom` → `pow` → `mul` → `add` → comparisons/logicals).
4. Reimplement statements and block constructs, ensuring AST variants receive spans.
5. Replace the original `parse` entry point, delete `abyss.pest`, and excise `pest` dependencies.
6. Wire up `ariadne`-powered diagnostics in the CLI and REPL, then run the full test suite.

## Grammar Mapping
- `statements` / `statement` (legacy PEG) → `build_parser` + `statement` recursive combinator in `grammar.rs`.
- `block` → `block_parser`, returning `AST::Block` with merged span info.
- `forge_var` / `assignment` → `forge_parser` and `assignment_parser`, emitting `AST::VarAssign` and `AST::Assignment` with `AssignmentOp` mapping.
- `engrave` → `engrave_parser`, building `AST::Engrave` with parameters and optional return type.
- `func_call` / expression towers → `expression_parser` composition (`or_expr_parser`, `and_expr_parser`, etc.) culminating in `atom_parser`.
- `oracle_expr` → `oracle_expr_parser`, including conditional bindings and branch handling via `oracle_branch_parser`.
- `orbit` / `orbit_flow` → `orbit_parser` and `orbit_flow_parser`, covering loop parameters plus `resume`/`eject` forms.
- Literals (`arcana`, `aether`, `rune`, `omen`) and identifiers → token layer (`tokens.rs`) feeding `literal_parser` helpers in `grammar.rs`.

## Test Coverage Inventory
- `tests/test_calc.rs`: arithmetic precedence, assignment operators, literal parsing for `arcana` and `aether`.
- `tests/test_comp.rs`: comparison operators and mixed-type comparisons.
- `tests/test_engrave.rs`: function declaration, invocation, recursion, and string handling.
- `tests/test_examples.rs`: end-to-end execution of `examples/hello.aby` and `examples/fibonacci.aby`.
- `tests/test_forge.rs`: declarative vs mutable bindings, redeclaration errors, and morph semantics.
- `tests/test_logical.rs`: boolean logic combinations and precedence with arithmetic.
- `tests/test_oracle.rs`: pattern and boolean oracles, nested conditionals, and assignments inside oracles.
- `tests/test_orbit.rs`: looping constructs, multi-parameter ranges, `resume`, and `eject` control flow.
- `tests/test_rune.rs`: string literals, concatenation, and `trans` behaviour with runes.
- `tests/test_type.rs`: explicit casting across primitive types.

## Decisions on Prior Open Questions
- Rollout will switch wholesale to the `chumsky` parser once the test suite passes; no temporary Cargo feature gate will be introduced.
- Structured (JSON) diagnostics are out of scope for this change and will be tracked as a future enhancement after the themed `ariadne` reports land.
- No strict performance baseline is required; we will run a light smoke benchmark to confirm there are no severe regressions compared to the `pest` implementation.
