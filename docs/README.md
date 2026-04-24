# AbySS Documentation Site

The Starlight-powered source for <https://abyss-lang.dev>, the official reference and tutorial site for the [AbySS](https://github.com/liebe-magi/abyss-lang) language.

The site is built with [Astro](https://docs.astro.build) + [Starlight](https://starlight.astro.build) and uses [Bun](https://bun.com) as the package manager and script runner. Deployment is handled via Vercel.

## Project Layout

```
docs/
├── public/              # Static assets (favicons etc.)
├── src/
│   ├── assets/          # Images embedded in docs pages
│   ├── content/
│   │   └── docs/        # .md / .mdx pages — each file is a route
│   └── content.config.ts
├── astro.config.mjs
├── package.json
└── tsconfig.json
```

Every ` ```abyss ` fenced block on the site uses the same TextMate grammar as the VS Code extension in `editors/code/`, so highlighting stays in sync with the language release.

## Commands

All commands are run from `docs/` in a terminal.

| Command         | Action                                       |
| :-------------- | :------------------------------------------- |
| `bun install`   | Install dependencies                         |
| `bun dev`       | Start the local dev server at `localhost:4321` |
| `bun build`     | Build the production site to `./dist/`      |
| `bun preview`   | Preview the production build locally         |
| `bun astro ...` | Run Astro CLI commands (e.g. `astro check`)  |

## Contributing

Content pages live under `src/content/docs/`. When you add or rename a page, verify the sidebar configuration in `astro.config.mjs`. For language-wide changes (new keywords, type semantics), coordinate with the corresponding updates in `crates/` and `editors/code/` so docs, grammar, and runtime stay aligned.
