# OpenSpec Archive (Frozen)

This directory is a **historical archive** of the OpenSpec-driven design process used on AbySS through 2025-11. It is **read-only**: no new proposals will be added, and the `openspec` CLI / workflow is no longer part of the development process. Current contributor and agent guidance lives in the root `AGENTS.md`.

## What is preserved here

- `specs/` — the "what IS built" snapshot captured at the time OpenSpec was retired. Useful as context for the archived changes below, not as a current source of truth. For the up-to-date language reference, see the [Starlight docs](https://abyss-lang.dev) and the crate source under `crates/`.
- `changes/archive/` — 16 completed change proposals (2025-11-08 through 2025-11-23) with their `proposal.md`, `tasks.md`, optional `design.md`, and spec deltas. These capture the reasoning behind major work such as the `chumsky` parser migration, the collections / artifacts additions, the multi-crate workspace refactor, the release automation, and the Starlight docs migration.

## Why it was retired

The core language and its surrounding tooling stabilised after v0.4.0, and subsequent work has been routine (dependency updates, lockfile maintenance, incremental fixes). The spec-driven flow was adding more friction than value for that kind of work. Retiring the workflow while leaving this archive in place keeps the design history easy to read without obliging future contributors to adopt the ceremony.

## If a large design discussion is needed again

Prefer writing a design note under `docs/src/content/docs/` or a dedicated RFC issue on GitHub. There is no longer a need to run `openspec` commands against this directory.
