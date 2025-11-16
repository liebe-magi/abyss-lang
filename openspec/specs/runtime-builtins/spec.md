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

### Requirement: Shared heap-backed values
The runtime SHALL store every heap-backed `Value` variant (`rune`, `scroll`, `lexicon`) in reference-counted pointers with interior mutability so aliases share one allocation, and SHALL wrap runtime data in `EvalResult::Data(Value)` so interpreter code has a single data representation.

#### Scenario: Assignment keeps shared handle
- **GIVEN** `forge a: scroll = [1]; forge b: materia = a;`
- **WHEN** the evaluator executes `b.scribe(9);`
- **THEN** `a` and `b` SHALL reference the same `Rc<RefCell<Vec<Value>>>`
- **AND** reading `a` afterwards SHALL observe the appended `9` without copying the entire scroll.

#### Scenario: EvalResult differentiates control flow
- **GIVEN** an expression evaluates to a `rune`
- **WHEN** the evaluator returns from `evaluate`
- **THEN** it SHALL emit `EvalResult::Data(Value::Rune(_))`
- **AND** control-flow signals such as `reveal` SHALL continue to use dedicated `EvalResult` variants so callers can distinguish data from flow.

### Requirement: Runtime stores artifact type schemas
The environment SHALL extend its symbol table to register artifact type definitions as schemas mapping artifact names to field-name/field-type pairs, enabling validation during instantiation and field access.

#### Scenario: Register artifact definition
- **GIVEN** source `artifact Player { name: rune; health: arcana; }`
- **WHEN** the evaluator processes the `AST::ArtifactDef`
- **THEN** the environment SHALL store a schema entry for "Player" with two fields and their types
- **AND** subsequent references to `Player` in type annotations SHALL resolve to this schema.

#### Scenario: Reject duplicate artifact definitions
- **GIVEN** two artifact definitions with the same name
- **WHEN** the evaluator processes the second definition
- **THEN** it SHALL raise an error indicating "Player" artifact is already defined
- **AND** the error SHALL include spans for both definitions.

#### Scenario: Artifact definition scope
- **GIVEN** an artifact defined in a function body
- **WHEN** the function returns
- **THEN** the artifact schema SHALL remain in the defining scope
- **AND** SHALL NOT be accessible outside that scope unless artifact definitions are always global (design decision needed).

### Requirement: Runtime stores artifact instances
The environment SHALL extend `Value` and `EvalResult` with `Artifact(String, HashMap<String, Value>)` variants that carry the artifact type name and a map of field names to runtime values.

#### Scenario: Store artifact variable
- **GIVEN** `forge hero: Player = Player { name: "Ardyn", health: 100 };`
- **WHEN** the evaluator executes the declaration
- **THEN** the resulting `VarInfo` SHALL hold `Value::Artifact("Player", fields)` where `fields` maps "name" to `Value::Rune("Ardyn")` and "health" to `Value::Arcana(100)`
- **AND** any later lookup SHALL return `EvalResult::Artifact` with the preserved field map.

#### Scenario: Clone artifact value
- **GIVEN** an artifact instance assigned to a variable
- **WHEN** that variable is assigned to another variable
- **THEN** the artifact value SHALL be cloned with a deep copy of all field values
- **AND** mutations to one instance SHALL NOT affect the other unless references are introduced later.

#### Scenario: Display artifact value
- **GIVEN** `unveil(hero)` where hero is an artifact instance
- **WHEN** the builtin formats the output
- **THEN** it SHALL display the artifact type name and field values in a readable format (e.g., `Player { name: "Ardyn", health: 100 }`)
- **AND** nested artifacts SHALL be displayed recursively.

### Requirement: Evaluator validates artifact instantiation
The evaluator SHALL validate artifact literals against registered schemas, ensuring all required fields are provided with correctly typed values, and SHALL raise errors for missing or mistyped fields.

#### Scenario: Instantiate valid artifact
- **GIVEN** `Player { name: "Hero", health: 100 }` where Player is defined with these fields
- **WHEN** the evaluator processes the literal
- **THEN** it SHALL look up the Player schema, validate field presence and types, construct a `Value::Artifact`, and return it as the expression result.

#### Scenario: Reject missing field
- **GIVEN** `Player { name: "Hero" }` missing the health field
- **WHEN** the evaluator processes the literal
- **THEN** it SHALL raise an error listing the missing field "health"
- **AND** the error SHALL include the literal's span.

#### Scenario: Reject extra field
- **GIVEN** `Player { name: "Hero", health: 100, level: 5 }` where level is not defined
- **WHEN** the evaluator processes the literal
- **THEN** it SHALL raise an error indicating field "level" is not defined in artifact Player
- **AND** the error SHALL suggest checking the artifact definition.

#### Scenario: Reject field type mismatch
- **GIVEN** `Player { name: "Hero", health: "high" }` where health expects arcana
- **WHEN** the evaluator processes the literal
- **THEN** it SHALL raise a type error indicating field "health" expects `arcana` but received `rune`
- **AND** the error SHALL include spans for the field assignment.

#### Scenario: Evaluate field expressions
- **GIVEN** `Player { name: summon("Name: "), health: 50 + 50 }`
- **WHEN** the evaluator constructs the literal
- **THEN** it SHALL evaluate each field value expression in declaration order
- **AND** SHALL validate the resulting values against field types after evaluation.

### Requirement: Evaluator handles field access
The evaluator SHALL implement field access for artifact instances, looking up field names in the instance's field map and returning the stored values, with errors for undefined fields or non-artifact targets.

#### Scenario: Access artifact field
- **GIVEN** `forge hp: arcana = hero.health;` where hero is an artifact with a health field
- **WHEN** the evaluator processes the `AST::FieldAccess`
- **THEN** it SHALL retrieve the "health" value from hero's field map and return it
- **AND** the result SHALL have type matching the field's declared type.

#### Scenario: Reject field on non-artifact
- **GIVEN** `forge x: arcana = 42;
forge y: arcana = x.health;`
- **WHEN** the evaluator attempts field access
- **THEN** it SHALL raise an error indicating arcana values do not have fields
- **AND** the error SHALL include the expression span.

#### Scenario: Reject undefined field
- **GIVEN** `hero.power` where hero is a Player without a power field
- **WHEN** the evaluator processes the access
- **THEN** it SHALL raise an error indicating field "power" does not exist on artifact Player
- **AND** the error SHALL list valid field names.

#### Scenario: Chain field access
- **GIVEN** `character.stats.current_hp` where character has a stats field and stats is an artifact with current_hp
- **WHEN** the evaluator processes the chain
- **THEN** it SHALL evaluate left-to-right, accessing "stats" from character and then "current_hp" from the intermediate artifact
- **AND** errors SHALL identify which level of the chain failed.

### Requirement: Evaluator handles field assignment
The evaluator SHALL implement field mutation for artifact instances, verifying the instance is declared with `morph`, validating field existence and type compatibility, and updating the field map in place.

#### Scenario: Assign artifact field
- **GIVEN** `forge morph hero: Player = Player { name: "Hero", health: 100 };
hero.health = 50;`
- **WHEN** the evaluator processes the field assignment
- **THEN** it SHALL verify hero is mutable, locate the "health" field, validate the new value is arcana, and update the field in the stored artifact
- **AND** subsequent reads of hero.health SHALL return 50.

#### Scenario: Reject assignment to immutable artifact
- **GIVEN** `forge hero: Player = Player { name: "Hero", health: 100 };
hero.health = 50;`
- **WHEN** the evaluator attempts the assignment
- **THEN** it SHALL raise an error indicating hero is immutable and cannot be mutated
- **AND** the error SHALL suggest using `forge morph`.

#### Scenario: Reject field assignment type mismatch
- **GIVEN** `forge morph hero: Player = Player { name: "Hero", health: 100 };
hero.health = "low";`
- **WHEN** the evaluator validates the assignment
- **THEN** it SHALL raise a type error indicating field "health" expects arcana but received rune
- **AND** the artifact instance SHALL remain unchanged.

#### Scenario: Chain field assignment
- **GIVEN** `forge morph character: Character = ...;
character.stats.current_hp = 75;`
- **WHEN** the evaluator processes the nested assignment
- **THEN** it SHALL verify character is mutable, access the stats field, verify stats is also an artifact (and mutable if it's stored by value), then update current_hp
- **AND** shallow vs deep mutability semantics SHALL be clearly defined.

### Requirement: Artifact type checking in declarations and calls
The evaluator SHALL enforce artifact type annotations in variable declarations, function parameters, and return values, ensuring assigned values match the declared artifact type by name.

#### Scenario: Validate artifact variable type
- **GIVEN** `forge hero: Player = Player { name: "Test", health: 100 };`
- **WHEN** the evaluator processes the declaration
- **THEN** it SHALL verify the literal's artifact type matches "Player" and assign the value.

#### Scenario: Reject mismatched artifact type
- **GIVEN** `artifact Enemy { name: rune; damage: arcana; }
forge hero: Player = Enemy { name: "Orc", damage: 10 };`
- **WHEN** the evaluator validates the declaration
- **THEN** it SHALL raise a type error indicating expected Player but received Enemy
- **AND** the error SHALL include both type names and the assignment span.

#### Scenario: Pass artifact to function
- **GIVEN** `engrave display(p: Player) -> abyss { unveil(p.name); }
display(hero);`
- **WHEN** the evaluator binds the argument
- **THEN** it SHALL verify hero's artifact type matches Player and bind the value to parameter p
- **AND** SHALL raise an error if the types don't match.

#### Scenario: Return artifact from function
- **GIVEN** `engrave create_default() -> Player { reveal Player { name: "Default", health: 100 }; }`
- **WHEN** the evaluator processes the return statement
- **THEN** it SHALL verify the revealed value is an artifact of type Player
- **AND** SHALL raise an error if the types don't match or if a non-artifact is returned.

### Requirement: Artifact equality and comparison
The runtime SHALL implement equality checks for artifact instances by comparing artifact types and field values recursively, enabling use in conditionals and oracle patterns.

#### Scenario: Compare equal artifacts
- **GIVEN** two artifact instances of the same type with identical field values
- **WHEN** evaluated in an equality expression
- **THEN** the result SHALL be `boon` (true)
- **AND** field order SHALL NOT affect equality.

#### Scenario: Compare unequal artifacts
- **GIVEN** two Player artifacts with different health values
- **WHEN** compared for equality
- **THEN** the result SHALL be `hex` (false)
- **AND** the comparison SHALL stop at the first differing field.

#### Scenario: Reject equality between different artifact types
- **GIVEN** a Player artifact and an Enemy artifact
- **WHEN** compared for equality
- **THEN** the result SHALL be `hex` (false) as they are different types
- **AND** no field comparison SHALL occur.

### Requirement: Stdlib centralises builtin method dispatch
The runtime stdlib SHALL own a registry of builtin methods keyed first by runtime type (e.g., `Type::Scroll`, `Type::Lexicon`, `Type::Materia`) and then by method name so the evaluator can remain agnostic of stdlib behavior. The registry SHALL live inside `stdlib::methods` alongside Rust implementations for each supported receiver type, and SHALL expose a single dispatcher that accepts the receiver `Value`, method name, argument list, and evaluation context.

#### Scenario: Register method tables during startup
- **WHEN** `stdlib::create_global_environment` runs
- **THEN** it SHALL call `stdlib::methods::get_all_builtin_methods()` (or equivalent) to build a per-type method table
- **AND** it SHALL store the resulting registry in the global environment so every scope can reuse the dispatcher.

#### Scenario: Evaluator delegates method calls
- **GIVEN** user code invokes `bag.tally()` on a scroll
- **WHEN** the evaluator resolves the receiver value and observes its runtime type
- **THEN** it SHALL pass the receiver, method name (`"tally"`), and arguments into the stdlib dispatcher
- **AND** the dispatcher SHALL look up the scroll method table and execute the registered Rust implementation.

#### Scenario: Unknown method surfaces coherent error
- **GIVEN** user code calls `lexicon.unknown()`
- **WHEN** the dispatcher fails to find the requested method in the lexicon table
- **THEN** it SHALL raise a runtime error that identifies the receiver type and method name, leaving the evaluator logic unchanged.

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

