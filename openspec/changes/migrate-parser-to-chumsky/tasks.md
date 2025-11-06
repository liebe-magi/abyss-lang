## 1. Planning & Scaffolding
- [ ] 1.1 Review current grammar in `src/abyss.pest` and map each rule to the corresponding AST shape.
- [ ] 1.2 Catalogue every parser-facing test in `tests/` to ensure coverage during the migration.

## 2. Parser Migration
- [ ] 2.1 Add `chumsky` and `ariadne` dependencies to `Cargo.toml`; update lockfile.
- [ ] 2.2 Remove `src/abyss.pest` and any unused `pest`-specific code paths.
- [ ] 2.3 Rebuild literal parsers (`arcana`, `aether`, `rune`, `omen`, identifiers) with `chumsky`, attaching spans for `LineInfo` reconstruction.
- [ ] 2.4 Implement expression precedence using `chumsky::primitive::just` and operator helpers, matching the semantics of the former grammar.
- [ ] 2.5 Recreate statement-level parsers (`forge`, assignments, `unveil`, `oracle`, `orbit`, `engrave`, `summon`, `trans`, etc.) and ensure they emit the existing AST variants.
- [ ] 2.6 Replace `build_ast` with integrated AST construction inside the new combinators, keeping the public `parse` API stable for callers.

## 3. Error Reporting
- [ ] 3.1 Define an internal error model that categorises parse failures for themed messages.
- [ ] 3.2 Implement translation from `chumsky` errors to the internal model and render them through `ariadne` with AbySS-flavoured messaging.
- [ ] 3.3 Provide convenience functions (e.g., `print_parse_errors`) for CLI and REPL entry points.

## 4. Validation & Cleanup
- [ ] 4.1 Run `cargo test` and ensure the entire suite passes without changes to expectations.
- [ ] 4.2 Add targeted regression tests if new edge cases arise (e.g., improved diagnostics scenarios).
- [ ] 4.3 Update documentation (README, CHANGELOG if present) to note the parser migration and diagnostic improvements.
- [ ] 4.4 Audit remaining code for lingering `pest` imports or unused utilities.
