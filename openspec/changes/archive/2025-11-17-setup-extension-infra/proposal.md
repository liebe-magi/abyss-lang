## Why
The VS Code extension under `editors/code` currently relies on manual yarn-based builds and is not part of the CI/CD surface. This slows iteration, introduces drift from the repo-wide tooling preferences, and prevents packaging the `.vsix` artifact during releases. We need consistent automation so extension releases stay reliable and aligned with the rest of the toolchain.

## What Changes
- Adopt Bun as the package manager for `editors/code`, replacing yarn and committing the Bun lockfile.
- Normalize the extension `package.json` scripts (compile, watch, check, package, publish) and hook `vscode:prepublish` so local and CI builds execute the same commands.
- Extend `.github/workflows/build.yml` with an `extension-check` job that installs Bun via `oven-sh/setup-bun` and runs `bun install && bun run check`.
- Update the release workflow to build the extension via `bunx vsce package`, upload the `.vsix` as an artifact, and attach it to GitHub Releases.
- Expand `renovate.json` so dependency automation also tracks the VS Code extension packages.

## Impact
- Config updates: `.github/workflows/build.yml`, `renovate.json`, `editors/code/package.json`.
- Tooling: deprecate yarn usage inside `editors/code`, standardize on Bun, and execute `vsce` packaging inside CI/CD.
- Release assets: GitHub Releases will now include the compiled `.vsix` artifact produced by the automated pipeline.
