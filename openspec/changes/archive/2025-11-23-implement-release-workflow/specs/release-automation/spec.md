# Spec: Release Automation

## ADDED Requirements

### Requirement: Local Version Management
The developer MUST be able to bump versions locally using `cargo release` without triggering git tags or publishing to crates.io immediately.

#### Scenario: Local Version Bump
Given the developer wants to release a new version
When they run `cargo release [level] --execute`
Then the `Cargo.toml` versions are updated
And the changes are committed
But no git tag is created
And no package is published to crates.io
And no commits are pushed automatically

### Requirement: Automated Release Trigger
The CI pipeline MUST automatically detect version bumps on the main branch and trigger the release process.

#### Scenario: CI Release Trigger
Given a commit is pushed to `main`
And the commit contains a version bump in `Cargo.toml`
And the version is higher than the latest tag
Then the CI pipeline triggers the release job

### Requirement: Automated Tagging and Publishing
The CI pipeline MUST handle the creation of git tags, GitHub releases, and publishing to crates.io.

#### Scenario: CI Tagging and Publishing
Given the release job is running
When tests and checks pass
Then a git tag `v{version}` is created and pushed
And a GitHub Draft Release is created
And the crates are published to crates.io in the correct order
