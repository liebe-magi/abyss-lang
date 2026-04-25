# documentation-structure Specification

## Purpose
Describe how the AbySS documentation site under `docs/` is organised — onboarding and structured reference sections built on Starlight — and how its code highlighting stays in lockstep with the VS Code extension by treating `editors/code/syntaxes/abyss.tmLanguage.json` as the single source of truth that the docs build pipeline consumes directly.
## Requirements
### Requirement: Documentation Site Structure
The project SHALL provide a documentation site under `docs/` powered by a static site generator that organizes content into clear onboarding and reference sections.

#### Scenario: Getting Started
- **GIVEN** a new user arrives at the documentation site
- **THEN** they can navigate to a Getting Started page that explains installation and how to run example programs.

#### Scenario: Language Reference
- **GIVEN** a user needs to inspect the language syntax or semantics
- **THEN** they can open structured pages under `docs/src/content/docs/reference/` to review detailed topics such as types, variables, and control flow.

### Requirement: Unified Syntax Highlighting
The documentation site SHALL treat `editors/code/syntaxes/abyss.tmLanguage.json` as the single source of truth for syntax highlighting and load that grammar directly into its code-highlighting pipeline.

#### Scenario: Highlighting Sync
- **GIVEN** the VS Code extension grammar file changes
- **WHEN** the documentation site rebuilds
- **THEN** all `abyss` code blocks render with the updated tokens and scopes from the refreshed grammar without manual duplication.

