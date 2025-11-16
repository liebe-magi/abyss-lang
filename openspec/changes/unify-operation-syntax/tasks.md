## 1. Parser and AST
- [x] 1.1 Add `glyph` to the lexer/token set and reserve it for type annotations.
- [x] 1.2 Remove the dedicated `trans(... as ...)` production, tokens, and AST variant so only dot-based conversions remain.

## 2. Evaluator and Environment
- [x] 2.1 Extend `Value`/`EvalResult` with a glyph-carrying variant and seed the global environment during stdlib initialisation with glyph variables for every built-in type name (`arcana`, `rune`, `scroll`, etc.).
- [x] 2.2 Update call dispatch to treat `.trans()` as a builtin method on `materia`, performing the existing conversion matrix via a glyph parameter.
- [x] 2.3 Register a glyph-valued global variable whenever an `artifact Foo` definition is evaluated so `Foo` can be referenced like any other identifier.

## 3. Stdlib Methods and Mutability
- [x] 3.1 Register `tally`, `scribe`, and `extract` as scroll methods, ensuring `scribe`/`extract` validate `morph core` receivers before borrowing mutably.
- [x] 3.2 Register `tally`, `define`, `expunge`, and `glossary` as lexicon methods with the same mutability enforcement for the mutating operations.
- [x] 3.3 Remove the legacy collection helper functions plus the global `measure`, `inscribe`, `retract`, `expunge`, and `contents` exports from the stdlib tables.
- [x] 3.4 Add regression tests covering `.trans`, `scroll` methods, and `lexicon` methods, including failure cases for immutable receivers and invalid glyph arguments.
