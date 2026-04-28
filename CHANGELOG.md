# Changelog

All notable changes to the `abyss-lang` workspace (the `abyss-core`, `abyss-interpreter`, and `abyss-lang` crates) are documented here. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html), with the Cargo convention that a `0.x.0` bump signals a potentially-breaking release while `0.x.y` is compatibility-preserving.

The accompanying VS Code extension has its own changelog at [`editors/code/CHANGELOG.md`](editors/code/CHANGELOG.md).

## [Unreleased]

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

For the full diff see [GitHub compare v0.1.0...v0.2.0](https://github.com/liebe-magi/abyss-lang/compare/v0.1.0...v0.2.0).

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
