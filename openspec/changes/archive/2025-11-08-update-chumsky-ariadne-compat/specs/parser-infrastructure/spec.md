## MODIFIED Requirements
### Requirement: Parser uses chumsky combinators
The system SHALL construct the AbySS AST by applying `chumsky` parser combinators defined in Rust code, compiled against the latest supported `chumsky 0.11` API and its lifetime-aware parser traits, replacing the legacy `pest` grammar and `build_ast` transformer.

#### Scenario: Parse valid incantation
- **GIVEN** a `.aby` script whose syntax is valid under the current language rules
- **WHEN** the CLI or library consumer invokes `parser::parse` using the `chumsky::input::Stream` adapter
- **THEN** the parser SHALL return a `Vec<AST>` identical to the legacy implementation's output
- **AND** no `pest`-specific code SHALL be executed during parsing
- **AND** the implementation SHALL compile without relying on deprecated `chumsky` symbols such as `SimpleReason`, the root-level `Stream`, or `map_with_span`

### Requirement: Parser emits themed diagnostics through ariadne
The system SHALL format parse errors via an abstraction that renders `ariadne`-based, AbySS-themed diagnostics to standard error and remains compatible with the `ariadne` 0.6 span and report builder APIs.

#### Scenario: Report unexpected token
- **GIVEN** a `.aby` script with a missing semicolon after a statement
- **WHEN** the parser encounters the syntax error
- **THEN** it SHALL emit a diagnostic using the reporting abstraction that highlights the offending span, includes a themed title, and suggests adding the semicolon while passing a span type accepted by `ariadne::Report::build`
- **AND** the abstraction SHALL avoid removed helpers such as `Simple::expected()` or the three-argument form of `Report::build`, ensuring diagnostics render without compilation warnings
