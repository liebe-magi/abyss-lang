## ADDED Requirements
### Requirement: Parser uses chumsky combinators
The system SHALL construct the AbySS AST by applying `chumsky` parser combinators defined in Rust code, replacing the legacy `pest` grammar and `build_ast` transformer.

#### Scenario: Parse valid incantation
- **GIVEN** a `.aby` script whose syntax is valid under the current language rules
- **WHEN** the CLI or library consumer invokes `parser::parse`
- **THEN** the parser SHALL return a `Vec<AST>` identical to the legacy implementation's output
- **AND** no `pest`-specific code SHALL be executed during parsing

### Requirement: Parser emits themed diagnostics through ariadne
The system SHALL format parse errors via an abstraction that renders `ariadne`-based, AbySS-themed diagnostics to standard error.

#### Scenario: Report unexpected token
- **GIVEN** a `.aby` script with a missing semicolon after a statement
- **WHEN** the parser encounters the syntax error
- **THEN** it SHALL emit a diagnostic using the reporting abstraction that highlights the offending span, includes a themed title, and suggests adding the semicolon

### Requirement: AST nodes capture precise source spans
The system SHALL attach `LineInfo` derived from the new parser's spans to every AST node that previously carried line information.

#### Scenario: Preserve line info for errors
- **GIVEN** a script that triggers an evaluation-time type error referencing a specific line
- **WHEN** the evaluator formats the error using `LineInfo`
- **THEN** the reported line and column SHALL match the positions produced by the new parser
