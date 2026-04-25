# editor-syntax Specification

## Purpose
Specify how the AbySS Codex Familiar VS Code extension's TextMate grammar (`editors/code/syntaxes/abyss.tmLanguage.json`) scopes every reserved keyword, primitive and collection type, boolean constant, builtin function, artifact construct, and structural operator emitted by the v0.3+ lexer, so editor highlighting stays aligned with the parser tokens defined in the interpreter.
## Requirements
### Requirement: Reserved keywords match the lexer
The AbySS Codex Familiar TextMate grammar (`editors/code/syntaxes/abyss.tmLanguage.json`) SHALL tag every reserved keyword emitted by the v0.3.0 lexer (`forge`, `morph`, `core`, `oracle`, `orbit`, `resume`, `eject`, `engrave`, `reveal`, `artifact`) with the `keyword.control.abyss` scope, and SHALL keep builtin helpers such as `unveil`, `summon`, and the `.trans(...)` method inside the `support.function.abyss` scope so users can distinguish actual keywords from callable symbols.

#### Scenario: Highlight README forge/oracle/orbit sample
- **GIVEN** the README snippet that declares `forge morph counter: arcana = 10;` and runs an `oracle { ... }`/`orbit (i = 0..5)` block
- **WHEN** it is opened in VS Code with the AbySS extension enabled
- **THEN** the tokens `forge`, `morph`, `oracle`, `orbit`, `resume`, `eject`, and `reveal` SHALL all receive the `keyword.control.abyss` scope according to the tmLanguage file
- **AND** calls to `unveil(...)` or `.trans(arcana)` in the same snippet SHALL be scoped as `support.function.abyss` rather than keywords.

### Requirement: Type keywords reflect the full v0.3.0 surface
The grammar SHALL tag every primitive and collection type supported by AbySS v0.3.0 (`arcana`, `aether`, `rune`, `omen`, `abyss`, `scroll`, `lexicon`, `materia`, `glyph`) with the `storage.type.abyss` scope wherever they appear in declarations, signatures, or artifact fields, and SHALL continue to scope `boon`/`hex` constants as `constant.language.abyss`.

#### Scenario: Highlight collection and glyph declarations
- **GIVEN** README examples that declare `forge spellbook: scroll = [1, boon];`, `forge ledger: lexicon = {...};`, and `engrave transcribe(target: glyph) -> rune { ... }`
- **WHEN** the files are opened in VS Code
- **THEN** the identifiers `scroll`, `lexicon`, `materia`, `glyph`, and the primitive types SHALL emit the `storage.type.abyss` scope
- **AND** the literals `boon` and `hex` SHALL emit the `constant.language.abyss` scope to reflect boolean semantics.

### Requirement: Artifact and operator syntax receives dedicated scopes
The grammar SHALL provide patterns that (a) match `artifact` definitions plus `TypeName::method` headers with `core`/`morph core` receivers, emitting `keyword.control.abyss` for `artifact`/`morph`/`core` and `entity.name.type.abyss` for the type identifier, and (b) classify structural operators introduced in v0.3.0 (`::`, `=>`, `->`, `..`, `..=`, compound assignment operators, logical `&&`, `||`, and unary `!`) as `keyword.operator.abyss` so artifact methods, match branches, and range expressions render consistently with the interpreter syntax described in `README.md` and implemented in `src/parser/tokens.rs`.

#### Scenario: Highlight artifact method definition and match ranges
- **GIVEN** the README artifact method sample `engrave Player::set_level(morph core, next: arcana) -> abyss { core.level = next; }` followed by an `oracle (i) { (0..=10) => ... }` match arm
- **WHEN** the snippet is viewed with the updated grammar
- **THEN** `artifact`, `morph`, `core`, and `engrave` SHALL carry the `keyword.control.abyss` scope while `Player` in both the definition and literal contexts SHALL emit `entity.name.type.abyss`
- **AND** the tokens `::`, `=>`, `..=`, `=`/`+=`/`-=` plus logical operators (`&&`, `||`, `!`) SHALL emit the `keyword.operator.abyss` scope to mirror the lexer tokens used by the interpreter.

