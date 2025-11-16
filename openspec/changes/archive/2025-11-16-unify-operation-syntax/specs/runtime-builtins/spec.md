## ADDED Requirements
### Requirement: Materia Trans Method
The stdlib SHALL register a builtin method named `trans` for the universal `materia` type so any runtime value can invoke conversions through dot syntax. The method SHALL be discoverable through the method dispatch table, accept exactly one glyph argument, and delegate to the evaluator's conversion helpers.

#### Scenario: Register trans as builtin method
- **GIVEN** the interpreter initialises the stdlib tables
- **WHEN** the environment requests the method metadata for a `materia` receiver named `trans`
- **THEN** it SHALL receive a `Callable::Builtin` entry referencing the Rust implementation, marked immutable because it does not mutate the receiver.

#### Scenario: Method call reaches builtin
- **GIVEN** source `"1".trans(arcana)`
- **WHEN** method dispatch resolves the call
- **THEN** it SHALL route to the registered builtin, passing both the receiver and glyph argument without going through the legacy global `trans` function symbol.

## MODIFIED Requirements
### Requirement: Stdlib exposes collection methods
The stdlib SHALL expose collection-oriented helpers as themed builtin methods attached to `scroll` and `lexicon` receivers so scripts call them via dot syntax (`bag.tally()`, `lex.define(...)`). The runtime SHALL enforce `morph core` restrictions on mutating helpers by checking the receiver's mutability before borrowing the shared collection handles.

#### Scenario: tally returns scroll length
- **GIVEN** `forge bag: scroll = [1, 2, 3];`
- **WHEN** code executes `forge count: arcana = bag.tally();`
- **THEN** the builtin SHALL borrow the scroll immutably and return `EvalResult::Data(Value::Arcana(3))`.

#### Scenario: scribe appends via morph core
- **GIVEN** `forge morph bag: scroll = [];`
- **WHEN** the program calls `bag.scribe("sigil");`
- **THEN** the builtin SHALL verify the receiver is mutable, borrow the shared scroll mutably, push the rune, and return `abyss`.

#### Scenario: scribe rejects immutable receiver
- **GIVEN** `forge bag: scroll = [];`
- **WHEN** the program calls `bag.scribe(1);`
- **THEN** the builtin SHALL raise an `EvalError` explaining that the receiver must be declared with `morph` before a mutating method may run, leaving the scroll unchanged.

#### Scenario: extract pops last element
- **GIVEN** `forge morph bag: scroll = [1];`
- **WHEN** code executes `forge last: materia = bag.extract();`
- **THEN** the builtin SHALL borrow the scroll mutably, remove the final entry, and return the removed value as `EvalResult::Data`.

#### Scenario: lexicon tally counts entries
- **GIVEN** `forge codex: lexicon = {"id": 7};`
- **WHEN** code runs `codex.tally();`
- **THEN** the builtin SHALL return the number of keys as an `arcana` without mutating the map.

#### Scenario: define writes lexicon entry
- **GIVEN** `forge morph codex: lexicon = {};`
- **WHEN** the program calls `codex.define("id", 7);`
- **THEN** the builtin SHALL verify mutability, borrow the lexicon mutably, insert the key/value pair, and return `abyss`.

#### Scenario: expunge removes lexicon entry
- **GIVEN** `forge morph codex: lexicon = {"id": 7};`
- **WHEN** the program executes `codex.expunge("id");`
- **THEN** the builtin SHALL delete the entry, returning `abyss`, and subsequent lookups SHALL observe the missing key.

#### Scenario: glossary returns lexicon keys
- **GIVEN** `forge codex: lexicon = {"id": 7, "name": "abyss"};`
- **WHEN** the program calls `forge keys: scroll = codex.glossary();`
- **THEN** the builtin SHALL borrow the lexicon immutably, collect rune keys into a new `scroll`, and return it through `EvalResult::Data`.
