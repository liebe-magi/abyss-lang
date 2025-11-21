[![Crates.io Version](https://img.shields.io/crates/v/abyss-lang)](https://crates.io/crates/abyss-lang)
[![Crates.io](https://img.shields.io/crates/l/abyss-lang)](https://github.com/liebe-magi/abyss-lang/blob/main/LICENSE)
[![Build](https://github.com/liebe-magi/abyss/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/liebe-magi/abyss/actions/workflows/build.yml)
[![Coverage](https://codecov.io/gh/liebe-magi/abyss-lang/branch/develop/graph/badge.svg)](https://app.codecov.io/gh/liebe-magi/abyss-lang)

# **AbySS: Advanced-scripting by Symbolic Syntax**

![logo](./docs/public/img/logo_512.jpg)

AbySS blends symbolic, spell-inspired syntax with a fast Rust core. Use it to iterate in an interpreter, script automation, or explore new language ideas—all while staying in a magical frame of mind.

- **Arcane syntax** with themed keywords like `forge`, `oracle`, and `orbit`.
- **Collections & artifacts** for structured data plus first-class mutation rules.
- **VS Code tooling** (AbySS Codex Familiar) that shares the same TextMate grammar as the docs.
- **Rust-powered compiler** built on `chumsky` and `ariadne` for resilient parsing and diagnostics.

## Quick Install

```bash
cargo install abyss-lang
```

Or build from source:

```bash
cd abyss-lang
cargo install --path .
```

### Try it quickly

```bash
abyss cast                 # interactive interpreter
abyss invoke examples/hello.aby
abyss align path/to/script.aby
```

## Documentation

The full language reference, tutorials, and roadmap now live in the [Starlight docs](./docs).

```bash
cd docs
bun install
bun run dev
```

Every ` ```abyss` sample on the site uses the same grammar as the VS Code extension, so highlighting stays in sync with the repo.

## Tooling

- [AbySS Codex Familiar](./editors/code) – VS Code extension with highlighting, snippets, and completions.
- `cargo llvm-cov` ready – run `cargo llvm-cov --open` for coverage reports; CI uploads to Codecov with the same command.

## License

MIT License © [liebe-magi](LICENSE)
