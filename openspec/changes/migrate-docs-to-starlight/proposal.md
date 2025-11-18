## Why
The root `README.md` mixes onboarding steps, tutorials, and deep language reference content, making it hard to navigate and impossible to keep syntax examples up to date. The docs site that already lives under `docs/` is not leveraging the definitive grammar shipped inside `editors/code`, so its code samples quickly diverge from the editor experience. We need a single documentation surface that pulls the latest VS Code grammar and hosts structured content so new and existing users can rely on it as the source of truth.

## What Changes
1. **Unify syntax highlighting** by importing `../editors/code/syntaxes/abyss.tmLanguage.json` inside `docs/astro.config.mjs` and registering it with Expressive Code / Shiki so every ` ```abyss` block uses the same grammar as the extension.
2. **Migrate content out of `README.md`** into the Astro Starlight site: move Installation and Getting Started instructions into `docs/src/content/docs/getting-started.mdx`, split the Language Syntax section into dedicated files under `docs/src/content/docs/reference/`, and ensure each MDX code block declares the `abyss` language for highlight validation.
3. **Trim the root README** so it focuses on a high-level overview, install instructions, project badges, and a pointer to the new documentation, turning the docs site into the canonical location for tutorials and reference material.
4. **Adopt a documentation capability spec** (`documentation-structure`) that captures the requirement for a Getting Started flow, a structured Language Reference, and unified syntax highlighting sourced from the extension grammar.

## Impact
- **Specs**: Introduces the `documentation-structure` capability with requirements for site structure and highlighting.
- **Docs**: Touches `README.md`, `docs/astro.config.mjs`, and new/updated MDX files under `docs/src/content/docs/**` to house the migrated content and reference structure.
- **DX**: Updating `editors/code/syntaxes/abyss.tmLanguage.json` will automatically refresh documentation highlighting after `bun run build`, ensuring the docs always display the same tokens that the VS Code extension supports.
- **Tooling**: `docs` builds now depend on the extension grammar file; local preview and CI must confirm `bun run build` succeeds with the new integration.
