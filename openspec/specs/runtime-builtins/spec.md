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
The environment SHALL represent scrolls as `Value::Scroll(Rc<RefCell<Vec<Value>>>)`, lexicons as `Value::Lexicon(Rc<RefCell<HashMap<String, Value>>>)`, runes as `Value::Rune(Rc<String>)`, and expose them via `EvalResult::Data(Value)` so every lookup returns the same shared handle instead of deep copies.

#### Scenario: Store scroll variable
- **GIVEN** `forge bag: scroll = [1, 2];`
- **WHEN** the evaluator executes the declaration
- **THEN** the resulting `VarInfo` SHALL hold `Value::Scroll(Rc<RefCell<_>>)` and any later lookup SHALL return `EvalResult::Data(Value::Scroll(_))` that points to the same allocation.

#### Scenario: Pass materia argument
- **GIVEN** `engrave echo_any(val: materia) -> materia { val }
 echo_any(bag);`
- **WHEN** the evaluator binds the argument
- **THEN** it SHALL treat `Type::Materia` as compatible with the shared `scroll` handle and skip type-mismatch errors while keeping `bag`'s allocation shared between caller and callee.

### Requirement: Evaluator handles collection literals and indexing
The evaluator SHALL construct `Rc<RefCell<_>>` handles for literal nodes, clone handles (not data) when propagating values, borrow collections immutably for reads, borrow mutably for writes, and surface results through `EvalResult::Data(Value)`.

#### Scenario: Access lexicon entry
- **GIVEN** `forge data: lexicon = {"id": 7}; forge entry: arcana = data["id"];`
- **WHEN** the evaluator processes the `AST::IndexAccess`
- **THEN** it SHALL borrow the lexicon immutably, read key `"id"`, clone the stored `Value` handle, and return it as `EvalResult::Data(Value::Arcana(7))`.

#### Scenario: Assign scroll slot
- **GIVEN** `morph bag: scroll = [1]; forge alias: materia = bag; bag[0] = 9;`
- **WHEN** the evaluator executes the assignment
- **THEN** it SHALL verify `bag` is mutable, borrow the shared scroll mutably, update index `0`, and both `bag` and `alias` SHALL observe the new value because they share the same handle.

### Requirement: Stdlib registers collection helpers
The stdlib SHALL expose collection-oriented builtins via `Callable::Builtin` so scripts can introspect and mutate shared collections consistently by borrowing the underlying `Rc<RefCell<_>>` handles rather than replacing entire vectors or maps.

#### Scenario: measure returns length
- **GIVEN** `measure([1, 2, 3])`
- **WHEN** the builtin executes
- **THEN** it SHALL borrow the scroll immutably, return an `arcana` count of `3`, and avoid cloning the collection data.

#### Scenario: inscribe appends value
- **GIVEN** `morph bag: scroll = []; forge alias: materia = bag; inscribe(alias, "sigil");`
- **WHEN** the builtin runs
- **THEN** it SHALL borrow `bag`'s shared handle mutably, append the rune, and both `bag` and `alias` SHALL report the appended element afterwards.

#### Scenario: retract pops and returns element
- **GIVEN** `morph bag: scroll = [1]; forge last: materia = retract(bag);`
- **WHEN** the builtin executes
- **THEN** it SHALL borrow the scroll mutably, remove the final element, return it as `EvalResult::Data(Value::Arcana(1))`, and share the mutated scroll with all aliases.

#### Scenario: expunge removes lexicon key
- **GIVEN** `morph data: lexicon = {"id": 1}; forge alias: materia = data; expunge(alias, "id");`
- **WHEN** the builtin runs
- **THEN** it SHALL borrow the shared lexicon mutably, delete the `"id"` entry, and both bindings SHALL observe the deletion.

#### Scenario: contents lists lexicon keys
- **GIVEN** `contents({"id": 1, "name": "abyss"})`
- **WHEN** the builtin executes
- **THEN** it SHALL borrow the lexicon immutably, collect rune keys into a new `Value::Scroll(Rc<RefCell<Vec<Value>>>)`, and return it via `EvalResult::Data`.

### Requirement: Shared heap-backed values
The runtime SHALL store every heap-backed `Value` variant (`rune`, `scroll`, `lexicon`) in reference-counted pointers with interior mutability so aliases share one allocation, and SHALL wrap runtime data in `EvalResult::Data(Value)` so interpreter code has a single data representation.

#### Scenario: Assignment keeps shared handle
- **GIVEN** `forge a: scroll = [1]; forge b: materia = a;`
- **WHEN** the evaluator executes `inscribe(b, 9);`
- **THEN** `a` and `b` SHALL reference the same `Rc<RefCell<Vec<Value>>>`
- **AND** reading `a` afterwards SHALL observe the appended `9` without copying the entire scroll.

#### Scenario: EvalResult differentiates control flow
- **GIVEN** an expression evaluates to a `rune`
- **WHEN** the evaluator returns from `evaluate`
- **THEN** it SHALL emit `EvalResult::Data(Value::Rune(_))`
- **AND** control-flow signals such as `reveal` SHALL continue to use dedicated `EvalResult` variants so callers can distinguish data from flow.

