# Change Log

All notable changes to the "abyss-codex-familiar" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

## [v0.4.0] - 2025-11-23

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
