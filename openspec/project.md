# Project Context

## Purpose
AbySS is a custom, magic-themed scripting language with its own interpreter, formatter, and CLI tooling. The language focuses on symbolic, approachable syntax (keywords such as `forge`, `oracle`, `orbit`) while retaining deterministic, strongly-typed evaluation semantics. The project aims to provide:
- A command-line tool (`abyss`) that can invoke `.aby` scripts, launch an interactive REPL, and format source files.
- A reusable Rust library (`abyss_lang`) that exposes the parser, AST, evaluator, and formatter for editor integrations and tooling.
- Example programs and tests that document the language surface area and ensure backwards-compatible behaviour.

## Tech Stack
- Rust stable toolchain (edition 2024) for the interpreter, CLI, and library code.
- `chumsky` 0.11 for parser combinators and incremental-friendly grammar definitions.
- `ariadne` 0.6 for themed diagnostic rendering.
- `clap` 4 for the multi-command CLI interface (`cast`, `invoke`, `align`).
- `rustyline` 17 for the REPL editor and history management.
- `colored` 3 for terminal styling and diagnostics.
- `dirs` 6 for OS-specific config directory discovery.
- `ordered-float` 5 for deterministic floating-point ordering during comparisons and formatting.

## Project Conventions

### Code Style
- Follow `rustfmt` defaults; run `cargo fmt` before opening pull requests.
- Prefer explicit error handling (`Result`/`?`) over `unwrap` in CLI paths; propagate errors with user-friendly messages.
- Module-level API uses `PascalCase` types and `snake_case` functions; enums implement `Display` or `Debug` when surfaced in errors.
- Keep user-facing strings thematic ("spell casting" vocabulary) but concise; centralise repeated text in helper functions where possible.

### Architecture Patterns
- **Front-end pipeline**: `parser::parse` (`chumsky` combinators) → `eval::evaluate`. AST definitions live in `ast.rs`; evaluation state is stored in `env::Environment`.
- **Interpreter**: `start_interpreter` in `main.rs` powers the REPL, orchestrating input buffering, brace balancing, AST inspection (debug mode), and evaluation.
- **Formatter**: `format::format_ast` pretty-prints AST nodes for both CLI formatting and REPL output capture.
- **Library/CLI split**: Core language functionality remains in `abyss_lang` so editor tooling and tests can consume the same APIs as the CLI.
- **Stdlib builtin dispatch**: `stdlib::methods` registers per-type method tables (scroll, lexicon, materia, etc.) plus a fallback dispatcher so evaluator code routes every `value.method(...)` invocation through a unified lookup instead of ad-hoc branching.
- **Examples**: Canonical `.aby` programs sit in `examples/` and serve both documentation and manual regression testing.

### Testing Strategy
- Primary coverage comes from integration-style tests in `tests/`, which feed example scripts through the parser/evaluator and assert typed `EvalResult` outputs.
- New language behaviour must add focused tests (e.g., `tests/test_calc.rs`, `tests/test_oracle.rs`) to prevent regressions in parsing precedence, environment state, and runtime errors.
- Run `cargo test` locally before pushing. For coverage checks, use `cargo llvm-cov` (requires `llvm-tools-preview` component).
- Coverage artifacts (`lcov.info`) are published by the `build.yml` workflow via `cargo llvm-cov --all-features --lcov --output-path lcov.info` and uploaded to Codecov. Forks must provide a `CODECOV_TOKEN` secret to keep coverage uploads working.
- When adding CLI flags or REPL features, supplement with targeted tests or scripted example runs to confirm line-editing and history mechanics work across platforms.

### Git Workflow
- Default branch is `develop`; feature work should occur on topic branches named with verb-led kebab-case (e.g., `add-aether-intrinsics`).
- Open pull requests against `develop`; keep commits scoped and `cargo fmt`/`cargo test` clean.
- GitHub Actions run build/test workflows and must pass before merging. Sync `develop` frequently to minimise conflicts.

## Domain Context
- Language types map to magical concepts: `arcana` (integers), `aether` (floats), `rune` (strings), `omen` (booleans using `boon`/`hex`), `abyss` (unit), `scroll`/`lexicon` (collections), `materia` (untyped slot), and `glyph` (type tokens passed to conversion APIs).
- Control flow uses themed keywords: `oracle` (conditionals/patterns), `orbit` (loops with `resume`/`eject`), `engrave` (function def), `summon` (input), `unveil` (output).
- Statements terminate with semicolons; block structure relies on braces, so formatter and REPL brace counting must remain accurate.
- Error reporting surfaces evaluation line info using `EvalError` variants; CLI paths should render coloured diagnostics via `display_error_with_source`.
- VS Code extension (`abyss-codex-familiar`) consumes the Rust library, so exported APIs must remain stable or follow semver when breaking changes occur.

## Important Constraints
- Keep the interpreter single-threaded and deterministic; evaluation depends on sequential state in `Environment`.
- Preserve backwards compatibility with the published language grammar unless a proposal is approved via OpenSpec.
- CLI must run on macOS, Linux, and Windows terminals; avoid platform-specific assumptions beyond standard path handling.
- Avoid introducing heavy dependencies; current binary footprint should stay small for quick installs via `cargo install`.
- User-facing commands (`cast`, `invoke`, `align`) must remain stable because they are referenced in published documentation and the VS Code extension.

## External Dependencies
- Crates: `ariadne`, `chumsky`, `clap`, `colored`, `dirs`, `ordered-float`, `rustyline` (all declared in `Cargo.toml`).
- Tooling: `cargo llvm-cov` with `llvm-tools-preview` for coverage; `cargo fmt`/`cargo clippy` for linting.
- Services: GitHub Actions workflow `build.yml` for CI; Codecov for coverage dashboards (requires `CODECOV_TOKEN`); crates.io for binary distribution; VS Code extension `abyss-codex-familiar` for editor integration.
