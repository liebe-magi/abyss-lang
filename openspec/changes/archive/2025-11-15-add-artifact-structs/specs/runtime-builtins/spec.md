## ADDED Requirements
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
