# Change Log

All notable changes to the "abyss-codex-familiar" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

## [v0.7.0] - 2026-07-07

### Added

- `fate` and `augury` join the storage-type highlight group, and the six error-handling names (`fate`, `augury`, `bless`, `curse`, `manifest`, `naught`) join keyword completion, matching the v0.7.0 language release.

### Changed

- Bumped the extension version to 0.7.0 to stay in lockstep with the `abyss-lang` crate. The `?` operator needs no grammar change (punctuation).

## [v0.6.1] - 2026-07-06

### Fixed

- The support-function highlight group now colours `transmute` instead of the removed `trans` (follow-up to the 0.6.0 rename; completion and snippets were already correct).

### Changed

- Bumped the extension version to 0.6.1 to stay in lockstep with the `abyss-lang` crate. The 0.6.1 cycle adds stdlib methods (`rune` / `scroll` / math rituals); per the established convention the completion list covers keywords rather than method names, so no grammar changes beyond the fix above.

## [v0.6.0] - 2026-07-03

### Changed

- **Breaking keyword rename followed from the language**: `resume` → `revolve`. The TextMate grammar's `keyword.control` group, the snippets, and the static keyword-completion provider all track the new name; `resume` is no longer highlighted or completed because it no longer parses.
- The completion list's method vocabulary follows the `trans` → `transmute` stdlib rename.
- Bumped the extension version to 0.6.0 to stay in lockstep with the `abyss-lang` crate. The 0.6.0 cycle is the Web Playground plus an internal compiler overhaul (spans, AST split, comment-preserving `align`); apart from the two renames above, the grammar surface is unchanged.

## [v0.5.0] - 2026-05-06

### Added

- New `ward` keyword in the `keyword.control` highlight group, matching the syntax landed in the `abyss-lang` crate's pattern-matching cycle. The static keyword-completion provider also offers `ward` so it auto-completes alongside `forge`, `oracle`, `engrave`, etc.

### Changed

- Bumped the extension version to 0.5.0 to stay in lockstep with the `abyss-lang` crate. The 0.5.0 cycle introduces guard clauses (`ward`), bare-identifier bindings, and scroll / artifact / lexicon destructuring patterns; these are all expressed with existing tokens (parentheses, brackets, braces, commas, colons, the new `ward` keyword) so the TextMate grammar additions stay minimal.

## [v0.4.1] - 2026-05-03

### Changed

- Bumped the extension version to 0.4.1 to stay in lockstep with the `abyss-lang` crate release. The 0.4.1 cycle was diagnostics polish ("did you mean?" hints, ariadne-rendered runtime errors) on the language side; **the grammar surface is unchanged from 0.4.0**, so existing snippets and completions continue to cover the current language surface.

## [v0.4.0] - 2026-04-25

### Changed

- Bumped the extension version to 0.4.0 to stay in lockstep with the `abyss-lang` crate release (the Rust side split into the `abyss-core` / `abyss-interpreter` / `abyss-cli` workspace and shipped the automated release workflow). The language grammar surface is unchanged since 0.3.0.
- Cleaned up redundant `name` fields from TextMate grammar captures.
- Migrated the extension logo from JPG to PNG and updated asset references.

### Updated

- TypeScript devDependency bumped to v6.

## [v0.3.0] - 2025-11-18

### Changed

- Extension version numbers now stay in lockstep with the `abyss-lang` crate (starting at v0.3.0) so grammar parity is always obvious.

### Updated

- TextMate grammar now mirrors the full AbySS v0.3.0 syntax: all reserved keywords, artifact/glyph types, collection literals, range/match operators, and builtin functions highlight consistently with `src/parser/tokens.rs` and the top-level README samples.

## [v0.0.2] - 2024-08-24

### Added

- Added new keywords: `engrave`, `summon`.
- Added a new snippet for function definitions: `Engrave Function`.
- Added a new snippet for standard input: `Summon Expression`.
- Syntax highlighting support for `abyss` type.

### Updated

- Included `engrave` and `summon` in keyword auto-completion.

## [v0.0.1] - 2024-08-22

### Added

- Initial release: Implemented basic keyword auto-completion and syntax highlighting for `forge`, `unveil`, `oracle`, `trans`, `orbit`, `resume`, `eject`, `reveal`, `morph`.
- Syntax highlighting for basic types: `omen`, `aether`, `arcana`, `rune`.
- Provided snippets for basic structures: `forge`, `unveil`, `oracle`, `orbit`, `resume`, `eject`, `reveal`.
