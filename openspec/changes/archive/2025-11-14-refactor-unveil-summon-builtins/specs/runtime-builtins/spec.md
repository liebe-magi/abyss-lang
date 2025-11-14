## ADDED Requirements
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
