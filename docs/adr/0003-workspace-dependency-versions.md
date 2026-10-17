# ADR 0003: Centralize Dependency Versions in the Workspace

- Status: Accepted
- Date: 2026-10-17

## Context

The workspace already centralizes most dependency declarations in the root `Cargo.toml`, but individual crates can still introduce version requirements directly in their own manifests. This spreads version policy across the repository and makes dependency updates harder to review consistently.

## Decision

Dependency versions must be declared centrally in the root `Cargo.toml` under `[workspace.dependencies]`. Workspace members must use `{ workspace = true }` for these dependencies and may add crate-specific features or other supported dependency options where needed.

New dependencies must follow this convention. Workspace package versions and shared package metadata remain centralized under `[workspace.package]` and should be inherited by member crates.

## Alternatives considered

- Declare dependency versions in each member manifest: rejected because version policy becomes scattered and inconsistent.
- Centralize only production dependencies: rejected because development and test dependencies also need a single version policy.

## Consequences

- Dependency version updates have one canonical location.
- Member manifests express usage and crate-specific features rather than independent version requirements.
- Existing direct dependency version declarations in member manifests must be moved to the workspace manifest.
