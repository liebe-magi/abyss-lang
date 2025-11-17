## 1. Renovate Configuration
- [x] Update `renovate.json` so `editors/code/package.json` is covered by the npm manager rules (bun usage still resolves npm metadata).

## 2. Extension Configuration (Bun Migration)
- [x] Replace the `scripts` block in `editors/code/package.json` with:
  ```json
  "scripts": {
    "vscode:prepublish": "bun run compile",
    "compile": "tsc -p .",
    "watch": "tsc -watch -p .",
    "check": "tsc --noEmit",
    "package": "vsce package",
    "publish": "vsce publish"
  }
  ```
- [x] Remove `yarn` references from the extension config/tooling and note in the PR description that contributors must delete `yarn.lock` locally, then run `bun install` to generate and commit `bun.lock`.

## 3. CI Workflow
- [x] Amend `.github/workflows/build.yml` with an `extension-check` job that uses `oven-sh/setup-bun` and runs `bun install` followed by `bun run check` inside `editors/code`.

## 4. CD Workflow
- [x] Update the release job in `.github/workflows/build.yml` to execute `bunx vsce package`, upload the resulting `.vsix` as a workflow artifact, and make it available to downstream jobs.
- [x] Ensure the `attach-assets-to-release` job downloads the `.vsix` artifact and publishes it as a GitHub Release asset alongside existing deliverables.
