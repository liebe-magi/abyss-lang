## Why
The AbySS Codex Familiar VS Code extension still reflects an older subset of the language grammar. Its TextMate definition (`editors/code/syntaxes/abyss.tmLanguage.json`) highlights only five primitive types, omits the `scroll`/`lexicon`/`materia`/`glyph` additions that shipped in AbySS v0.3.0, treats builtin I/O helpers such as `unveil` and `summon` as control-flow keywords, and never marks the `artifact` or `core` tokens that power structs and methods. As a result, example code copied from `README.md` or the interpreter looks partially unhighlighted and misleads users about which identifiers are actually reserved by the parser (`src/parser/tokens.rs`).

## What Changes
- Expand the `keyword.control.abyss` list so it matches the reserved tokens emitted by the lexer (`forge`, `morph`, `core`, `oracle`, `orbit`, `resume`, `eject`, `engrave`, `reveal`, `artifact`) and relocate builtin functions (`unveil`, `summon`) plus the `trans` method into the `support.function.abyss` group.
- Extend the `storage.type.abyss` matcher to include every v0.3.0 type keyword (`arcana`, `aether`, `rune`, `omen`, `abyss`, `scroll`, `lexicon`, `materia`, `glyph`) so variable declarations, function signatures, and artifact fields render consistently with the docs.
- Add grammar patterns for artifact/method syntax tokens (`artifact` declarations, `Type::method` double-colon segments, `core`/`morph core` receivers) and ensure range/arrow operators (`..`, `..=`, `=>`, `->`, `::`, `+=`, `&&`, `||`, etc.) are recognized as `keyword.operator.abyss`.
- Document these expectations in a new `editor-syntax` capability so future language releases keep the grammar aligned with the interpreter and README samples.

## Impact
- Specs: new `editor-syntax` capability containing requirements for the VS Code grammar.
- Code: `editors/code/syntaxes/abyss.tmLanguage.json` plus any related tests or snippet previews.
- Tooling: extension contributors gain a checklist tied to language releases, reducing drift between the interpreter and syntax highlighting.
