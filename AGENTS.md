# Repository Responsibilities

This is a polyglot monorepo. Keep each concern within the tool that owns it; do
not introduce a second task graph for the same ecosystem.

## Deno

Deno owns the TypeScript workspace, dependencies, formatting, linting,
type-checking, and tests.

- Workspace members live under `workspaces/`.
- Shared TypeScript infrastructure packages live directly under `workspaces/`.
- Effect services live under `workspaces/services/`.
- Run workspace-wide TypeScript checks and tests through root `deno.json` tasks.
- Do not add Turborepo or Node package-manager workspace manifests.

## Cargo

Cargo owns the Rust workspace, dependencies, builds, checks, and tests.

- The root `Cargo.toml` owns the workspace.
- Rust crates live under `crates/`.
- Run Rust commands from the repository root.

## devenv and Nix

- devenv owns repository tools, local services, and cross-ecosystem task
  composition.
- Nix pins devenv and supports reproducible build or deployment artifacts.
- `flake.nix` must not define the development shell.
- Keep development dependencies in devenv and language dependencies in Deno or
  Cargo.

## PostgreSQL

`workspaces/postgres/` owns shared connectivity and the generic migration
runner. Each service owns its migration assets. `workspaces/migrations/` is the
single executable composition point that registers and runs service migrations.

Zapatos owns generated PostgreSQL table types. Each service keeps its
configuration and generated schema under its repositories layer. Apply central
migrations before regenerating types.

## Service deployment

TypeScript backend services form a modular monolith by default and may share one
container. The frontend is always built as a standalone container. Preserve
explicit service boundaries so deployment topology does not determine business
architecture.
