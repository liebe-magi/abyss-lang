# AbySS Codex Familiar

![logo](https://raw.githubusercontent.com/liebe-magi/abyss-lang/main/editors/code/assets/logo_512.png)

AbySS Codex Familiar is a Visual Studio Code extension that provides custom support for the programming language [AbySS](https://github.com/liebe-magi/abyss-lang), including syntax highlighting, code snippets, and basic code completion.

> **Versioning policy:** Starting with release `0.3.0`, this extension's version number always matches the `abyss-lang` crate version so you immediately know which language grammar is covered.

## Features

- **Syntax Highlighting**: Full support for the AbySS language surface — keywords, types (including `scroll`, `lexicon`, `materia`, `glyph`), artifact declarations/methods, boolean literals, match/range operators, and more.
- **Code Snippets**: Handy snippets for common constructs like `forge`, `unveil`, `oracle`, `orbit`, and more.
- **Code Completion**: Auto-complete support for AbySS keywords to streamline coding.
- **Version-aligned Releases**: Each extension release is tagged with the same version as the root `abyss-lang` crate, signaling grammar parity out of the box.
- **Custom Folding Markers**: Utilize `// #region` and `// #endregion` to create collapsible code regions.

## How to Use

1. Install the extension in Visual Studio Code.
2. Open any `.aby` file to start using the features of AbySS Codex Familiar, including syntax highlighting, snippets, and code completion.
3. Enjoy a streamlined and enhanced coding experience with AbySS!

## Requirements

- Visual Studio Code version 1.92.0 or higher.

## Known Issues

None at the moment. Please report any issues via GitHub.

## Release Notes

### 0.4.0

- Version bumped to track `abyss-lang` 0.4.0 (multi-crate workspace split and release-automation milestone on the Rust side). No grammar changes since 0.3.0 — existing snippets and completions continue to cover the current language surface.
- Tidied the TextMate grammar by removing redundant `name` fields from captures.
- Migrated the extension logo from JPG to PNG and refreshed asset paths.
- Updated the TypeScript devDependency to v6.

### 0.3.0

- Synced the extension version number with `abyss-lang` (both are `0.3.0`) so you can trust the grammar matches the core language release.
- Overhauled the TextMate grammar to highlight every v0.3.0 construct: artifact definitions/methods, glyph conversions, scroll/lexicon/materia literals, boolean literals, and match/range operators.
- Kept snippets and completions aligned with the latest language keywords.

### 0.0.2

- Added new keywords: `engrave`, `summon`.
- Expanded syntax highlighting to include the `abyss` type and boolean constants `boon`, `hex`.
- Introduced new snippets for function definitions (`engrave`) and standard input (`summon`).
- Updated auto-completion to support the new keywords `engrave` and `summon`.

### 0.0.1

- Initial release of `AbySS Codex Familiar` with syntax highlighting and folding markers.
- Added code snippets for common AbySS constructs.
- Introduced basic keyword completion for faster coding.
