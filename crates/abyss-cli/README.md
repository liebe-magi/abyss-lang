# abyss-lang (CLI)

The "Interface" of the AbySS language. This crate provides the command-line tool for interacting with AbySS.

## Responsibilities

- **CLI Entry Point**: Parses command-line arguments using `clap` (`main.rs`).
- **REPL**: Provides an interactive Read-Eval-Print Loop using `rustyline` for live coding.
- **File I/O**: Handles reading script files from disk.
- **Configuration**: Manages user configuration and history files (e.g., `~/.abyss`).

## Usage

```bash
# Run a script
abyss invoke examples/hello.aby

# Start the REPL
abyss cast

# Format a script
abyss align examples/hello.aby
```

For installation instructions, see the [root README](../../README.md).
