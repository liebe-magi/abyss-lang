## ADDED Requirements
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

## MODIFIED Requirements
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

