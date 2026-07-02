# Changelog

All notable changes to the `abyss-lang` workspace (the `abyss-core`, `abyss-interpreter`, and `abyss-lang` crates) are documented here. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html), with the Cargo convention that a `0.x.0` bump signals a potentially-breaking release while `0.x.y` is compatibility-preserving.

The accompanying VS Code extension has its own changelog at [`editors/code/CHANGELOG.md`](editors/code/CHANGELOG.md).

## [Unreleased]

### Removed

- **`abyss_core::semantic` module (`SymbolTable` / `SymbolInfo`)** ([#503](https://github.com/liebe-magi/abyss-lang/issues/503)) — the static-analysis scaffold had zero consumers anywhere in the workspace since its introduction. The roadmap's LSP milestone (v0.8.1) is preceded by the v0.8 span-tracking refactor, and a symbol table without span data would have been rewritten there anyway; git history preserves the removed code. Breaking for `abyss-core` per Cargo 0.x semver, so this ships with the next `0.x.0` bump.

## [0.5.0] - 2026-05-06

A pattern-matching milestone. The `oracle` match-mode arms grew five new powers — guard clauses with `ward`, fresh bindings, and three destructuring shapes (`scroll`, `artifact`, `lexicon`) — composing freely so a single arm can pull values out of nested structured data without follow-up index access. The cycle is intentionally additive: every script that compiled on 0.4.1 keeps compiling.

### Added

- **`ward` keyword for guard clauses** ([#414](https://github.com/liebe-magi/abyss-lang/pull/414)) — `(x) ward x > 0 => …` lets a match arm carry an extra condition that must hold for the arm to fire. The arm's pattern is evaluated as before, and the ward expression is only evaluated if the pattern matched. A non-omen ward expression surfaces a runtime error (`Oracle ward must evaluate to an omen, found …`).
- **Bare-identifier bindings in match-mode patterns** ([#417](https://github.com/liebe-magi/abyss-lang/pull/417)) — a bare identifier in a match-mode pattern position introduces a fresh binding to the scrutinee value, scoped to that arm. Visible in the ward expression and the body, gone when the arm finishes. Wildcards (`_`) and literal patterns are unaffected; if-else mode still treats bare identifiers as boolean expressions.
- **Scroll head/tail destructuring** ([#420](https://github.com/liebe-magi/abyss-lang/pull/420)) — `[head, ..rest]`, `[a, b]`, `[]`, `[..]`, `[..rest]` shapes against scroll scrutinees, with element-level bindings, wildcards, literal compares, and a named or anonymous trailing rest segment that captures the unmatched tail as a fresh sub-scroll.
- **Artifact field destructuring** ([#421](https://github.com/liebe-magi/abyss-lang/pull/421)) — `Player { name, health }` (shorthand binding), `Player { name: "Ardyn", health }` (literal compare + binding), `Player { name: _ }` (explicit wildcard), and the empty `Type {}` for type-only dispatch. Pattern types fall through when the scrutinee is the wrong artifact type so sibling arms can dispatch by type. Unknown field names raise the existing "did you mean?" hint via `missing_field_error`.
- **Lexicon key destructuring** ([#423](https://github.com/liebe-magi/abyss-lang/pull/423)) — `{ "name": n, "port": p }`, partial key sets, literal compares, and the empty `{}` for "any lexicon". Listed keys not present in the scrutinee fall through so chained arms with progressively smaller key sets compose naturally.
- **Pattern Matching reference page + `examples/pattern.aby`** ([#424](https://github.com/liebe-magi/abyss-lang/pull/424)) — dedicated [Pattern Matching](https://abyss-lang.dev/reference/pattern-matching/) page covering all five features in one place, plus a single example file exercising every match-arm shape end-to-end. The example is locked down by `tests/test_examples.rs::pattern_example_executes`.
- **Roadmap published with v0.5–v0.8 plan** ([#412](https://github.com/liebe-magi/abyss-lang/pull/412)) — the [Roadmap](https://abyss-lang.dev/roadmap/) replaces the previous topic-grouped wishlist with an ordered Release Plan: v0.5 Pattern Matching, v0.6 Web Playground & Wasm, v0.6.x Standard Library Growth, v0.7 First-class Error Handling (Option / Result + `?` operator), v0.8 Span-tracking Refactor, v0.8.1+ LSP MVP. Generics + user-defined enums sit in *Later*, tied together because their canonical motivating examples need both.

### Changed

- **Oracle evaluator refactored for scope cleanup** ([#415](https://github.com/liebe-magi/abyss-lang/pull/415)) — the `AST::Oracle` arm now pushes its scope, delegates to a `evaluate_oracle` helper that uses `?` freely, and pops on every exit (including error paths). Previously five hand-written `env.pop_scope(); return Err(...)` branches were prone to drift; consolidating them removes a real scope-leak class observable in the REPL after a runtime error inside an oracle.
- **`AST::Var` in match-mode pattern position now binds rather than looks up** ([#417](https://github.com/liebe-magi/abyss-lang/pull/417)) — semantically additive on the example surface (no existing script in the repository used a bare identifier as a match-mode pattern), but worth noting because users wanting to compare against an existing variable's value should now use a ward, e.g. `(any) ward any == existing_var =>`.
- **VS Code TextMate grammar and keyword completion gain `ward`** ([#424](https://github.com/liebe-magi/abyss-lang/pull/424)) — highlighted in the same `keyword.control` group as `forge` / `oracle` / `engrave` and offered by the static keyword-completion provider.
- **`reference/conditionals.mdx`** ([#412](https://github.com/liebe-magi/abyss-lang/pull/412), [#424](https://github.com/liebe-magi/abyss-lang/pull/424)) — the bare match-mode summary now points readers at the dedicated Pattern Matching page for the richer arm shapes; the stale "future enums" reference was dropped because v0.5.0's artifact-pattern type-dispatch already covers the original use case.

### Fixed

- **CHANGELOG forge example was syntactically invalid AbySS** ([#411](https://github.com/liebe-magi/abyss-lang/pull/411)) — the v0.4.1 entry illustrated the new "did you mean?" hint with `forge x = 1;`, which actually fails at parse time because `forge` requires an explicit type annotation. Corrected to `forge x: arcana = 1;` and added an `AGENTS.md` rule mandating that every AbySS snippet in user-facing text be executed against the local interpreter before publishing.

For the full diff including Renovate-driven dependency-lock-file updates, see the [GitHub compare v0.4.1...v0.5.0](https://github.com/liebe-magi/abyss-lang/compare/v0.4.1...v0.5.0).

## [0.4.1] - 2026-05-03

A diagnostics-polish release. Runtime errors now render through the same `ariadne` reporter the parser already uses, and three new "did you mean?" / "available alternatives" hint paths fire when AbySS programs reference identifiers, artifact fields, or methods that nearly match a known name. **Language semantics are unchanged from 0.4.0**; existing scripts continue to work without modification.

### Added

- **"Did you mean?" hints for undefined identifiers** ([#397](https://github.com/liebe-magi/abyss-lang/pull/397)) — `forge x: arcana = 1; reveal y;` now reports `Variable y (did you mean: x?) is not defined!` when a close lexical match exists in scope. Suggestions are deterministic, capped at three, and ordered by Levenshtein distance.
- **"Did you mean?" + available-alternatives hints for methods and artifact fields** ([#401](https://github.com/liebe-magi/abyss-lang/pull/401)) — three new error sites enrich their messages: missing artifact field, missing artifact method, and missing builtin method on a value type. The artifact-field error additionally lists every defined field name so the schema is obvious without re-reading the declaration.
- **Runtime errors render through `ariadne`** ([#404](https://github.com/liebe-magi/abyss-lang/pull/404)) — when an `EvalError` carries a source position, the offending column is underlined in the same labelled, coloured report style the parser uses, so the visual treatment is consistent across parser and runtime diagnostics. A plain `Error: …` line is still printed when no position is attached.

### Changed

- `EvalError` is now `#[non_exhaustive]` ([#404](https://github.com/liebe-magi/abyss-lang/pull/404)). The crate is pre-1.0, but the marker future-proofs it against the planned span-tracking refactor — downstream `match` on the error enum now needs a wildcard arm.

### Removed

- The redundant Claude Code Review GitHub workflow has been retired ([#408](https://github.com/liebe-magi/abyss-lang/pull/408)). It was failing on every Renovate PR ("Workflow initiated by non-human actor"), which had been blocking dependency-bump auto-merge. Copilot review covers the same feedback loop, so the duplicate stage was net cost.

For the full diff including Renovate-driven dependency-lock-file updates, see the [GitHub compare v0.4.0...v0.4.1](https://github.com/liebe-magi/abyss-lang/compare/v0.4.0...v0.4.1).

## [0.4.0] - 2026-04-25

A major housekeeping release: the Rust crate layout becomes a 3-crate workspace published in lockstep, the release pipeline is fully automated end-to-end (crates.io publish, GitHub Release with binary archives, VS Code Marketplace publish), and the spec-driven OpenSpec workflow is retired in favour of a leaner contributor guide. **Language semantics are unchanged from 0.3.1**; existing scripts continue to work without modification.

### Added

- **Multi-crate workspace** — split the interpreter source into three independently published crates ([#201](https://github.com/liebe-magi/abyss-lang/pull/201), [#203](https://github.com/liebe-magi/abyss-lang/pull/203)):
  - `abyss-core` (new): AST, `chumsky` parser, semantic analysis, formatter. Lightweight and platform-agnostic so future LSP / Wasm playground tooling can depend on it without pulling in runtime code.
  - `abyss-interpreter` (new): runtime environment, evaluator, value model, standard library.
  - `abyss-lang`: the `abyss` CLI binary; binary-only.
- **Automated release pipeline** ([#379](https://github.com/liebe-magi/abyss-lang/pull/379), [#383](https://github.com/liebe-magi/abyss-lang/pull/383), [#385](https://github.com/liebe-magi/abyss-lang/pull/385), [#387](https://github.com/liebe-magi/abyss-lang/pull/387)) — pushing a workspace version bump to `main` triggers, in order: full test suite → tag creation → draft GitHub Release with auto-generated notes → crates.io publish (`abyss-core` → `abyss-interpreter` → `abyss-lang`) → per-target binary archive attachment → VS Code Marketplace publish. The whole flow is idempotent and can be re-run via `workflow_dispatch` with a `force` input.
- **Pre-built binary archives** ([#385](https://github.com/liebe-magi/abyss-lang/pull/385)) attached to every GitHub Release for six targets: Linux x86_64 / aarch64, macOS x86_64 / aarch64, Windows x86_64 / aarch64. Each archive ships the `abyss` binary plus `LICENSE` and `README.md`, with a `.sha256` sidecar for verification.
- **Workspace-wide version-sync drift guard** ([#379](https://github.com/liebe-magi/abyss-lang/pull/379)) — `scripts/check_version_sync.py` now also enforces that intra-workspace dependency versions in `[workspace.dependencies]` stay aligned with `[workspace.package].version`, preventing partial bumps from reaching a release pipeline.
- **Expanded test coverage** ([#203](https://github.com/liebe-magi/abyss-lang/pull/203)) — over 270 integration tests across `abyss-core` and `abyss-interpreter`, with `cargo llvm-cov` reports uploaded to Codecov on every PR.

### Changed

- Documentation lives at <https://abyss-lang.dev>, built with Astro + Starlight (the migration completed in 0.3.1; 0.4.0 is the first release where the docs site is the canonical reference rather than the README).
- The `OpenSpec` spec-driven workflow has been retired. The `openspec/` directory is preserved as a frozen historical archive of the v0.3.x design process; current contributor / agent guidance lives in the consolidated root `AGENTS.md` ([#378](https://github.com/liebe-magi/abyss-lang/pull/378)).
- Crate metadata centralised under `[workspace.package]` and `[workspace.dependencies]`, so a future workspace-version bump only needs to be edited in one place ([#379](https://github.com/liebe-magi/abyss-lang/pull/379)).

### Fixed

- Build status badge URLs in the root and `crates/abyss-cli` READMEs (pointed at the pre-rename `liebe-magi/abyss` repository slug, so the badges rendered as "unknown" on the crates.io page) ([#381](https://github.com/liebe-magi/abyss-lang/pull/381)).
- The `abyss-lang` crate previously published an empty autolib alongside the `abyss` binary; `crates/abyss-cli/src/lib.rs` removed ([#381](https://github.com/liebe-magi/abyss-lang/pull/381)).

### Dependencies

- `chumsky` 0.11 → 0.12 ([#225](https://github.com/liebe-magi/abyss-lang/pull/225))
- `rustyline` 17 → 18 ([#347](https://github.com/liebe-magi/abyss-lang/pull/347))
- VS Code extension TypeScript devDep → v6 ([#340](https://github.com/liebe-magi/abyss-lang/pull/340))
- Documentation site: Astro → v6 ([#318](https://github.com/liebe-magi/abyss-lang/pull/318))

For the full diff including the ~150 Renovate-driven dependency-lock-file updates, see the [GitHub compare v0.3.1...v0.4.0](https://github.com/liebe-magi/abyss-lang/compare/v0.3.1...v0.4.0).

## [0.3.1] - 2025-11-22

Documentation overhaul plus the first iteration of the automated release infrastructure.

### Added

- **Documentation migration to Astro Starlight** — the docs at <https://abyss-lang.dev> are now structured into Getting Started + Reference sections, with code highlighting driven by the same TextMate grammar the VS Code extension uses (single source of truth between editor and docs).
- **Initial automated release workflow** — `release.yml` was introduced (`feat: Implement automated release workflow`); the full hardening (idempotency, binary attach, Marketplace publish) lands in 0.4.0.
- **Version sync script** — `scripts/check_version_sync.py` enforces that the root `Cargo.toml` and `editors/code/package.json` stay at the same version. Wired into pre-commit and CI.
- **Coverage spec and additional interpreter tests** — `cargo llvm-cov` ready, with regression tests around `SymbolTable`, evaluator error paths, and stdlib argument validation.

### Changed

- Project logo migrated from JPG to PNG; assets refactored under `docs/public/img/`.
- Pre-commit configuration tightened to also verify version sync.

For the full diff see [GitHub compare v0.3.0...v0.3.1](https://github.com/liebe-magi/abyss-lang/compare/v0.3.0...v0.3.1).

## [0.3.0] - 2025-11-16

The biggest single release on the language axis: typed records (artifacts) with methods, tightened control-flow keywords, and rewired evaluator and stdlib around shared, well-documented abstractions.

### Added

- **Artifact structs** — `artifact Player { name: rune; health: arcana; }` declares a typed record with named fields. Literals, formatter integration, and field mutation rules under `morph` ship together.
- **Artifact methods** — `engrave Player::heal(morph core, amount: arcana) -> abyss { … }` syntax with `core` / `morph core` receivers; methods are dispatched through a unified stdlib registry and called via `hero.heal(50)`.
- **Glyph type** — types are first-class values; `glyph` reserves a type-token so functions can accept `target: glyph` parameters.
- **Materia `trans` method** — conversion is now a builtin method: `"123".trans(arcana)` instead of the legacy `trans("123" as arcana)` form.
- **Codecov integration** — coverage uploads from CI.

### Changed

- **`oracle` syntax cleaned up** — `oracle { … }` runs in one of two unambiguous modes: if-else mode (no parens, top-down guard expressions) or match mode (`oracle (expr) { … }` with pattern arms).
- **Unified operation syntax** — built-in operations (length, conversion, etc.) consolidated to method-call syntax across `scroll`, `lexicon`, `materia`. The standalone `trans()` legacy form is rejected by the parser.
- **Evaluator architecture refactored** — split into focused submodules (`result`, `values`, `collections`, `expressions`, `statements`).
- **Builtin method dispatch refactored** — `stdlib::methods` owns a per-type registry keyed by runtime type and method name, keeping the evaluator agnostic of stdlib semantics.
- **Shared collection semantics** — `scroll` / `lexicon` values use `Rc<RefCell<…>>` so aliases share one allocation; `rune` uses `Rc<String>` for shared-immutable storage.

### Removed

- Legacy `trans(value as type)` standalone form. Use the method form `value.trans(type)` instead.
- Inline binding inside `oracle (… = …)` parentheses. Replace with an explicit `forge` declaration followed by an if-else `oracle`.

For the full diff see [GitHub compare v0.2.0...v0.3.0](https://github.com/liebe-magi/abyss-lang/compare/v0.2.0...v0.3.0).

## [0.2.0] - 2025-11-15

### Added

- **Collection types** — `scroll` (lists), `lexicon` (key/value maps), and `materia` (the universal type slot) become first-class typed primitives with literal syntax (`[…]`, `{ "k": v }`) and matching method APIs.
- Argument validation on `unveil` for clearer diagnostics.

### Changed

- **`unveil` and `summon` refactored** — both now flow through the unified `Callable` abstraction in the runtime; the parser lexes them as ordinary identifiers and emits `AST::FuncCall`, dropping the bespoke AST variants.
- Materia conversion error handling improved.

For the full diff see [GitHub compare 0.1.0...v0.2.0](https://github.com/liebe-magi/abyss-lang/compare/0.1.0...v0.2.0). (The `0.1.0` tag was pushed without a `v` prefix; the convention switches to `vX.Y.Z` from `v0.2.0` onward.)

## [0.1.0] - 2025-11-08

The first significant `abyss-lang` release on crates.io after the 2024 preview. Marks the start of the modern incarnation of the project.

### Added

- **Migration from `pest` to `chumsky` parser combinators** with `ariadne`-rendered themed diagnostics and span-preserving AST nodes.
- Comprehensive parser regression tests, comment handling, and example-test infrastructure.
- Pre-commit configuration covering `cargo fmt` / `clippy` / `check` / `test`, aligned with the CI checks.

### Changed

- Adopted Rust 2024 edition.
- Replaced `unwrap()` with `expect()` in numeric parsing for clearer panic diagnostics.

For the full diff see [GitHub compare 0.0.2...0.1.0](https://github.com/liebe-magi/abyss-lang/compare/0.0.2...0.1.0) (≈129 commits, including a heavy stream of Renovate dependency-lock-file updates over the months between the two releases).

## [0.0.2] - 2024-08-23

Preview release of the interpreter and matching VS Code extension. By this point the language exposed `forge`, `morph`, `engrave`, `oracle`, `orbit`, `resume`, `eject`, `reveal`, `summon`, `unveil`, `trans`, plus the `arcana` / `aether` / `rune` / `omen` / `abyss` types and the `boon` / `hex` constants. See [`editors/code/CHANGELOG.md`](editors/code/CHANGELOG.md) for the keyword-by-keyword timeline.

## [0.0.1] - 2024-08-21

Initial preview release of the interpreter and the AbySS Codex Familiar VS Code extension. See [`editors/code/CHANGELOG.md`](editors/code/CHANGELOG.md) for the original keyword set.
