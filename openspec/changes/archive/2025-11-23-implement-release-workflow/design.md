# Design: Release Workflow

## Architecture

### Local Workflow (Developer)
The developer uses `cargo release` to manage version bumps.
Command: `cargo release [level] --execute`
Configuration (`Cargo.toml`):
```toml
[workspace.metadata.release]
shared-version = true
tag = false
push = false
publish = false
```

### CI Workflow (GitHub Actions)
Trigger: `push` to `main`.

Steps:
1.  **Checkout**: Get code.
2.  **Check Version**: Compare `Cargo.toml` version with `git describe --tags --abbrev=0`.
    - If `Cargo.toml` version > git tag, proceed.
    - Else, skip release steps.
3.  **Test & Build**: Run `cargo test`, `cargo build`.
4.  **Tag**: Create a git tag `v{version}` and push it.
    - *Note*: We need to handle race conditions or ensure only one release happens.
    - *Auth*: Needs `GITHUB_TOKEN` with write permissions.
5.  **Publish**: Run `cargo publish` for each crate.
    - Order matters: `abyss-core` -> `abyss-interpreter` -> `abyss-lang`.
    - `cargo release` can actually handle the publishing order if we use it in CI too, or we script it.
    - *Alternative*: Use `cargo release publish --execute --no-confirm` in CI?
        - If we use `cargo release` in CI, it simplifies the logic.
        - But the user said "GitHub Actions for tag creation".
        - We can use `cargo release` in CI to do the tagging and publishing!
        - Flow:
            - Dev bumps version locally (modifies `Cargo.toml`), commits, pushes.
            - CI sees change.
            - CI runs `cargo release --tag --publish --execute`?
            - But `cargo release` usually expects to *make* the version bump.
            - If version is already bumped, `cargo release` might be confused or we just use `cargo publish`.
            - Better:
                - Dev bumps version.
                - CI:
                    - `git tag vX.Y.Z`
                    - `cargo publish -p abyss-core`
                    - `cargo publish -p abyss-interpreter`
                    - `cargo publish -p abyss-lang`
6.  **Release**: Create GitHub Release (Draft) with artifacts.

## Trade-offs
- **Manual vs Automated Tagging**: Automated tagging ensures tags match `Cargo.toml`.
- **Publishing Order**: Must publish dependencies first. `cargo publish --all` might not work if dependencies aren't on crates.io yet. `cargo release` handles this well. Maybe we can use `cargo release publish` in CI?
    - `cargo release publish` assumes it's running the release process.
    - We can use `cargo workspaces` or just a script.
