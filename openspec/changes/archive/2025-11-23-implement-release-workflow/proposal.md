# Release Workflow Automation

## Goal
Implement a robust release workflow that synchronizes crate versions using `cargo release` locally, while delegating tagging, release creation, and crates.io publishing to GitHub Actions.

## Context
Currently, the project uses a basic CI workflow that creates draft releases on pushes to `main`. However, it lacks:
- Synchronized version bumping across the workspace.
- Automated tagging based on version changes.
- Controlled publishing to crates.io.

The user wants to standardize the release process:
1.  **Local**: `cargo release` bumps versions (synced) but does *not* tag or publish.
2.  **CI**: Detects version bumps, runs checks, creates tags/releases, and publishes to crates.io.

## Strategy
- **Local**: Use `cargo-release` configuration to ensure workspace synchronization and disable git tagging/pushing/publishing.
- **CI**: Update `.github/workflows/release.yml` (or modify `build.yml`) to:
    - Trigger on push to `main`.
    - Check if the version in `Cargo.toml` differs from the latest git tag.
    - If new version:
        - Create git tag.
        - Create GitHub Release (Draft).
        - Publish to crates.io.
