## Tasks

### 1. Configure Syntax Highlighting
- [x] Update `docs/astro.config.mjs` so it imports `../editors/code/syntaxes/abyss.tmLanguage.json` and registers the grammar inside `integrations.starlight.expressiveCode.shiki.langs`.

### 2. Content Migration: Getting Started
- [x] Move the `README.md` Installation and Getting Started sections into `docs/src/content/docs/getting-started.mdx`.

### 3. Content Migration: Language Reference
- [x] Split the `README.md` Language Syntax content into topic-specific pages under `docs/src/content/docs/reference/` (e.g., `types.mdx`, `variables.mdx`, `conditionals.mdx`).
- [x] Ensure every migrated code block declares `abyss` so syntax highlighting can be validated.

### 4. Content Migration: Others
- [x] Relocate the remaining README sections (e.g., roadmap or similar material) into appropriate MDX files such as `roadmap.mdx`.

### 5. README Cleanup
- [x] Reduce `README.md` to an overview, install quickstart, project badges, license, and a prominent link to the docs site.

### 6. Verify
- [x] Run `cd docs && bun run build`, then confirm the local preview renders links correctly and that `abyss` highlighting honors the imported grammar.
