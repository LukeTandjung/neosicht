# Repository Responsibilities

## Cargo

Cargo owns the Rust workspace, dependencies, builds, checks, and tests.

- The root `Cargo.toml` owns the workspace.
- Rust crates live under `crates/`.
- Each crate represents one cohesive service or product feature and follows the
  repository's hexagonal architecture conventions internally.
- Run Rust commands from the repository root.
- Keep tests under each crate's `tests/` directory, separate from production
  modules under `src/`.

## devenv and Nix

- devenv owns repository tools and task composition.
- Nix pins devenv and supports reproducible build or deployment artifacts.
- `flake.nix` must not define the development shell.
- Keep development dependencies in devenv and Rust dependencies in Cargo.

## macOS integration

- Rust never touches Objective-C objects directly.
- Native C/Objective-C sources, build scripts, and unsafe FFI declarations stay
  with the adapter that owns them.
- Private APIs must be runtime-probed and degrade gracefully.
