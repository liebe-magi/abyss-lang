## ADDED Requirements
### Requirement: Artifact Methods Register With Schemas
The evaluator SHALL associate every `engrave Type::method(core ...)` definition with the owning artifact schema, capturing metadata such as method name, parameter list, return type, and whether the receiver is mutable, so runtime dispatch can resolve methods by artifact type.

#### Scenario: Register method on artifact
- **GIVEN** artifact `Player` and source `engrave Player::get_level(core) -> arcana { reveal core.level; }`
- **WHEN** the script is evaluated and declarations are loaded into the environment
- **THEN** the resulting artifact schema for `Player` SHALL include a method entry named `get_level` marked immutable with its parameter/return signature, making it discoverable by later method calls.

#### Scenario: Reject duplicate method names
- **GIVEN** two definitions `engrave Player::get_level(core) { ... }` in the same compilation unit
- **WHEN** the environment attempts to register the second definition
- **THEN** it SHALL raise an `EvalError` indicating the method already exists on `Player`, preventing ambiguity during dispatch.

### Requirement: Method Call Evaluation
The evaluator SHALL execute `AST::MethodCall` nodes by evaluating the receiver expression, resolving the method from the receiver's artifact type, implicitly binding the receiver value to the `core` parameter, and evaluating the remaining arguments before invoking the method body.

#### Scenario: Invoke immutable method
- **GIVEN** `forge hero: Player = Player { level: 10 };` and method `engrave Player::get_level(core) -> arcana { reveal core.level; }`
- **WHEN** the program executes `unveil(hero.get_level());`
- **THEN** the evaluator SHALL resolve `get_level` on `Player`, bind `hero` to `core`, pass zero explicit arguments, and reveal `10` without requiring the caller to supply the receiver explicitly.

#### Scenario: Unknown method error
- **GIVEN** `forge hero: Player = Player { level: 10 };`
- **WHEN** the program executes `hero.promote();` without a corresponding method definition
- **THEN** the evaluator SHALL raise an `EvalError` indicating `promote` is undefined for artifact `Player` and SHALL not attempt fallback to global `engrave` functions.

### Requirement: Method Mutability Enforcement
The evaluator SHALL enforce that methods declared with `morph core` run only when the receiver value originates from a variable, field, or temporary marked mutable, and SHALL surface a runtime error when an immutable receiver attempts to invoke a mutable method.

#### Scenario: Mutable method succeeds on morph variable
- **GIVEN** `forge morph hero: Player = Player { level: 10 };` and `engrave Player::set_level(morph core, next: arcana) -> abyss { core.level = next; }`
- **WHEN** the program executes `hero.set_level(11);`
- **THEN** the evaluator SHALL allow the call, mutate `hero.level` to `11`, and continue execution normally.

#### Scenario: Mutable method fails on immutable variable
- **GIVEN** `forge hero: Player = Player { level: 10 };` (immutable) and the same `set_level` method
- **WHEN** the program executes `hero.set_level(11);`
- **THEN** the evaluator SHALL raise an `EvalError` explaining that `hero` is immutable and cannot call a method requiring `morph core`.

#### Scenario: Immutable method callable on either receiver
- **GIVEN** both immutable `hero` and mutable `morph ally: Player`
- **WHEN** they each execute `get_level()`
- **THEN** the evaluator SHALL allow both invocations because the method is tagged immutable, demonstrating that the mutability restriction only applies when `morph core` is specified.
