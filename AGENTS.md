# AGENTS.md

Guidance for AI coding assistants (and humans) working in this repository.

## Project at a Glance

AbySS is a magic-themed scripting language with its own interpreter, formatter, and CLI tooling. The repo is a single Cargo workspace that also hosts the Starlight documentation site and the VS Code extension.

- **`crates/abyss-core`** — AST (`ast.rs`), `chumsky`-based parser (`parser/`), static analysis (`semantic.rs`), and formatter (`format.rs`). Designed to stay lightweight so a future LSP or Wasm playground can depend on it without pulling in runtime code.
- **`crates/abyss-interpreter`** — `RuntimeEnv` (`env.rs`), the `Value` enum, the evaluator (`eval/`), and the standard library (`stdlib/`, including the per-type method tables).
- **`crates/abyss-cli`** — the user-facing binary `abyss` (crate name `abyss-lang`, published to crates.io). Hosts `main.rs` with `start_interpreter` (the REPL driver) and the `clap` subcommands `cast`, `invoke`, and `align`.
- **`docs/`** — Starlight (Astro + bun) source for <https://abyss-lang.dev>.
- **`editors/code/`** — the `abyss-codex-familiar` VS Code extension. Version is kept in lockstep with the `abyss-lang` crate (see `scripts/check_version_sync.py`).
- **`examples/`** — canonical `.aby` programs referenced by both the docs and tests.
- **`openspec/`** — frozen archive of the retired spec-driven workflow. See `openspec/README.md`; do not add new proposals there.

## Tech Stack

- Rust stable toolchain, edition 2024.
- `chumsky` 0.12 for parser combinators and `ariadne` 0.6 for themed diagnostics (both in `abyss-core`).
- `ordered-float` 5 for deterministic floating-point comparisons.
- `clap` 4, `rustyline` 18, `colored` 3, and `dirs` 6 in `abyss-cli` for the CLI, REPL, terminal styling, and OS config paths.

## Conventions

### Code style
- `rustfmt` defaults; run `cargo fmt --all` before opening PRs.
- Prefer explicit `Result`/`?` handling in CLI paths; surface user-facing errors via `display_error_with_source` rather than `unwrap`.
- Keep user-facing strings thematic ("spell casting" vocabulary) but concise; centralise repeated text in helpers where practical.

### Documentation samples
- Any AbySS snippet that appears in user-facing text — `README.md`, `CHANGELOG.md`, GitHub release notes, the Starlight site under `docs/`, the VS Code extension `README`, PR descriptions, or commit messages — must be executed end-to-end against the current interpreter before publishing. Write the snippet to a temp `.aby` file and run it via `cargo run -p abyss-lang -- invoke <file>`, or paste it into `cargo run -p abyss-lang -- cast`. Do not paste pseudocode or guess syntax — even for one-line examples — because subtle requirements (e.g. mandatory type annotations on `forge`) are easy to misremember.
- For error-message examples, capture the *actual* string the interpreter emits rather than paraphrasing. Both the variant `Display` impl and any wrapper formatting (`label_with_suggestions`, `did_you_mean_hint`, ariadne's report header from `kind_label`) shape the final output, and the result is easy to get subtly wrong from memory.

### Testing
- Primary coverage comes from per-crate integration tests under `crates/abyss-core/tests/` and `crates/abyss-interpreter/tests/`. New language behaviour must land with a focused test (e.g. `test_calc.rs`, `test_oracle.rs`, `test_collections.rs`).
- Run `cargo test` locally before pushing. For coverage, use `cargo llvm-cov --all-features --lcov --output-path lcov.info` (requires the `llvm-tools-preview` component).
- Coverage is uploaded to Codecov by `.github/workflows/build.yml`.

### Git workflow
- Default branch is `develop`; `main` is release-only. Direct pushes to either branch are blocked by branch protection — always branch off with a verb-led kebab-case topic branch (`add-…`, `fix-…`, `refactor-…`) and open a PR. `develop` → `main` promotion also goes through a PR.
- PR titles and bodies are written in English, matching the established practice on the repository, even when the originating conversation is in another language.
- CI (`build.yml`) runs `cargo check`, `cargo test` with coverage, `cargo fmt --check`, `cargo clippy -D warnings`, version-sync validation, and the VS Code extension type-check + package step. All must pass before merge.
- Releases are driven by `release.yml` on push to `main`: when the root `Cargo.toml` `[workspace.package].version` differs from the latest tag, the workflow tags, drafts a GitHub release, publishes `abyss-core` → `abyss-interpreter` → `abyss-lang` to crates.io in that order, attaches per-target binary archives (`tar.gz` for Linux/macOS, `zip` for Windows, with `.sha256` sidecars) to the release, and publishes the VS Code extension to the Marketplace at the same version. The workflow is idempotent on retry: tag creation, draft release, per-crate publishes, and the Marketplace publish all skip work already done, while binary archives are rebuilt and re-uploaded deterministically via `gh release upload --clobber`. The workflow can also be re-run manually via `workflow_dispatch` with the `force` input to recover from a partial failure.
- The Marketplace publish step requires a repository Secret named `VSCE_PAT` containing an Azure DevOps Personal Access Token scoped to "Marketplace > Manage" on the `liebe-magi` publisher.

### Pre-commit hooks (`.pre-commit-config.yaml`)
- `cargo fmt`, `cargo clippy -D warnings`, `cargo check`, `cargo test` on any `.rs` change.
- `bun run compile` on any `editors/code/` change (requires `bun install --frozen-lockfile` to succeed).
- `python3 scripts/check_version_sync.py` enforces that the root `Cargo.toml` workspace version (`[workspace.package].version` plus intra-workspace dep versions in `[workspace.dependencies]`) stays aligned with `editors/code/package.json`.

## Domain Vocabulary

Language types map to magical concepts: `arcana` (integer), `aether` (float), `rune` (string), `omen` (boolean with `boon`/`hex`), `abyss` (unit), `scroll`/`lexicon` (collections), `materia` (untyped slot), and `glyph` (type token passed to conversion APIs).

Control flow keywords: `oracle` (conditionals / patterns), `orbit` (loops with `resume`/`eject`), `engrave` (function definition), `summon` (input), `unveil` (output). Statements terminate with semicolons; block structure relies on braces, so formatter and REPL brace counting must stay accurate.

Errors surface line info via `EvalError` variants and are rendered with `display_error_with_source` for coloured diagnostics.

## Important Constraints

- The interpreter is single-threaded and deterministic; evaluation depends on sequential state in `RuntimeEnv`. Do not introduce hidden concurrency.
- Preserve backwards compatibility with the published language grammar. A breaking change requires an explicit decision and coordinated extension / docs updates.
- User-facing commands (`cast`, `invoke`, `align`) are referenced in published docs and the VS Code extension — treat them as stable APIs.
- The CLI must run on macOS, Linux, and Windows terminals. Avoid platform-specific assumptions beyond standard path handling.
- Keep the binary footprint small; avoid heavy new dependencies without clear justification.
- When changing exported APIs in `abyss-core` or `abyss-interpreter`, remember that the VS Code extension and any future editor tooling consume them — follow semver.

## External References

- crates.io — `abyss-core`, `abyss-interpreter`, `abyss-lang`.
- VS Code Marketplace — `liebe-magi.abyss-codex-familiar`.
- Docs — <https://abyss-lang.dev> (Starlight, deployed from `docs/`).
- Codecov — coverage dashboard (token: `CODECOV_TOKEN`).
