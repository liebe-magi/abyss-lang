## Why
The language currently mixes three unrelated operation surfaces—method calls, stdlib functions, and the bespoke `trans(value as type)` form—which makes the grammar harder to follow and prevents future APIs from sharing a single dot-based invocation story. Conversions, collection helpers, and other utilities should feel identical to artifact methods so users learn one pattern.

## What Changes
- Introduce the `glyph` type, reserve its keyword in the parser, and have stdlib initialisation seed glyph-valued global variables (`arcana`, `rune`, `scroll`, etc.) plus register glyph globals whenever an `artifact Foo` definition is evaluated so type names behave like ordinary identifiers at runtime.
- **BREAKING** Replace the `trans(value as type)` syntax with the method call `value.trans(glyph_value)` implemented as a builtin on `materia`, ensuring conversions ride on the dot pipeline.
- **BREAKING** Recast the collection helper builtins (`measure`, `inscribe`, `retract`, `expunge`, `contents`) as themed methods on `scroll` and `lexicon` (`tally`, `scribe`, `extract`, `define`, `expunge`, `glossary`) with mutability checks enforced via `morph core` semantics.
- Update parser, evaluator, and stdlib plumbing to remove the old tokens/AST nodes, add glyph-aware values, and register the new method surfaces while keeping type names accessible purely through identifier lookup.

## Impact
- Affected specs: parser-infrastructure, evaluator-infrastructure, runtime-builtins
- Affected code: `ast.rs`, `parser/tokens.rs`, `parser/grammar.rs`, `eval/{mod.rs,values.rs,statements.rs}`, `env.rs`, `stdlib/{mod.rs,collections.rs}`, conversion helpers, and the acceptance tests covering `trans` and collection helpers.
