## ADDED Requirements
### Requirement: Runtime stores scroll and lexicon values
The environment SHALL extend `Value` and `EvalResult` with `Scroll(Vec<Value>)` and `Lexicon(HashMap<String, Value>)` variants, treat them as first-class data, and permit `Type::Materia` annotations to accept any runtime `Value`.

#### Scenario: Store scroll variable
- **GIVEN** `forge bag: scroll = [1, 2];`
- **WHEN** the evaluator executes the declaration
- **THEN** the resulting `VarInfo` SHALL hold `Value::Scroll` and any later lookup SHALL return `EvalResult::Scroll` with the preserved element order.

#### Scenario: Pass materia argument
- **GIVEN** `engrave echo_any(val: materia) -> materia { val }
 echo_any(bag);`
- **WHEN** the evaluator binds the argument
- **THEN** it SHALL treat `Type::Materia` as compatible with the `scroll` value and skip type-mismatch errors.

### Requirement: Evaluator handles collection literals and indexing
The evaluator SHALL construct runtime vectors/maps for literal nodes, support bracket-based reads for both `scroll` and `lexicon`, and allow indexed assignment only when the target expression resolves to a `morph` variable of a collection type.

#### Scenario: Access lexicon entry
- **GIVEN** `forge entry: arcana = data["id"];` where `data` is a `lexicon`
- **WHEN** the evaluator processes the `AST::IndexAccess`
- **THEN** it SHALL return the stored `Value` for key `"id"` or raise a runtime error if the key is missing.

#### Scenario: Assign scroll slot
- **GIVEN** `morph bag: scroll = [1]; bag[0] = 9;`
- **WHEN** the evaluator executes the assignment
- **THEN** it SHALL verify `bag` is mutable, update index `0` to `Value::Arcana(9)`, and raise an error if the target is immutable or not a collection.

### Requirement: Stdlib registers collection helpers
The stdlib SHALL expose collection-oriented builtins via `Callable::Builtin` so scripts can introspect and mutate collections consistently.

#### Scenario: measure returns length
- **GIVEN** `measure([1, 2, 3])`
- **WHEN** the builtin executes
- **THEN** it SHALL return an `arcana` count of `3` for a scroll and the number of keys for a lexicon.

#### Scenario: inscribe appends value
- **GIVEN** `morph bag: scroll = []; inscribe(bag, "sigil");`
- **WHEN** the builtin runs
- **THEN** it SHALL append the `"sigil"` rune to `bag`'s backing vector and return `abyss`.

#### Scenario: retract pops and returns element
- **GIVEN** `morph bag: scroll = [1]; forge last: materia = retract(bag);`
- **WHEN** the builtin executes
- **THEN** it SHALL remove the final element, return it as `materia`, and error if the scroll is empty.

#### Scenario: expunge removes lexicon key
- **GIVEN** `morph data: lexicon = {"id": 1}; expunge(data, "id");`
- **WHEN** the builtin runs
- **THEN** it SHALL delete the `"id"` entry and return `abyss` whether or not the key existed.

#### Scenario: contents lists lexicon keys
- **GIVEN** `contents({"id": 1, "name": "abyss"})`
- **WHEN** the builtin executes
- **THEN** it SHALL return a `scroll` of rune keys in an unspecified order.
