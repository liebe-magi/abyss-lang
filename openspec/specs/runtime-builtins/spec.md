# runtime-builtins Specification

## Purpose
TBD - created by archiving change refactor-unveil-summon-builtins. Update Purpose after archive.
## Requirements
### Requirement: Runtime exposes callable abstraction
The interpreter SHALL register both engraved functions and built-in functions as `Callable` entries so that the environment can resolve any symbol through a single lookup path.

#### Scenario: Register engraved function
- **GIVEN** a script defines `engrave echo(r: rune) -> rune { unveil(r); r }`
- **WHEN** the evaluator processes the `engrave` statement
- **THEN** the environment SHALL store `echo` as a `Callable::Engraved`
- **AND** subsequent calls to `echo` SHALL resolve using that callable.

#### Scenario: Resolve builtin function
- **GIVEN** the interpreter initialises its global environment
- **WHEN** code evaluates `unveil("hi")`
- **THEN** the environment lookup SHALL return a `Callable::Builtin`
- **AND** the evaluator SHALL dispatch to the registered Rust function pointer.

### Requirement: Stdlib registers IO builtins
The runtime SHALL expose a `stdlib` module that seeds the global environment with Rust-implemented I/O built-ins matching the `BuiltinFunc` signature, including `unveil` and `summon`.

#### Scenario: Call unveil builtin
- **WHEN** a program executes `unveil("You are ", name, "!")`
- **THEN** the builtin SHALL stringify each argument using AbySS display rules (e.g., `omen` -> `boon`/`hex`, `rune` honours escape sequences)
- **AND** it SHALL write the concatenated string to standard output
- **AND** it SHALL return `abyss`.

#### Scenario: Call summon builtin
- **WHEN** a program executes `summon("Input your name: ")`
- **THEN** the builtin SHALL print the prompt, flush stdout, read a line from stdin, trim the trailing newline, and return the captured text as a `rune`
- **AND** callers needing another type SHALL use `trans` to perform explicit conversion.

#### Scenario: Summon requires rune prompt
- **WHEN** a program calls `summon` with a non-`rune` argument
- **THEN** the evaluator SHALL raise a type error indicating that `summon` expects a rune prompt.

### Requirement: Parser treats unveil and summon as ordinary function calls
The parser SHALL lex `unveil` and `summon` as identifiers and emit `AST::FuncCall` nodes for them instead of bespoke AST variants, so all function invocations share the same syntax path.

#### Scenario: Parse unveil invocation
- **WHEN** the parser encounters `unveil("hi")`
- **THEN** it SHALL produce an `AST::FuncCall` whose name is `unveil`
- **AND** no `AST::Unveil` node SHALL be created.

#### Scenario: Parse summon invocation
- **WHEN** the parser encounters `summon("prompt")`
- **THEN** it SHALL produce an `AST::FuncCall` node with one argument expression
- **AND** it SHALL NOT emit the legacy `AST::Summon` variant.

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

