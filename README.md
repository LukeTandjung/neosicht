# Project Skeleton

A private GitHub template for Luke T's polyglot personal projects.

## Tool ownership

- **Deno** owns TypeScript dependencies, formatting, linting, checks, and tests.
- **Cargo** owns Rust dependencies, checks, and tests.
- **devenv** provides repository tools, PostgreSQL, and cross-ecosystem tasks.
- **Nix** pins the repository-scoped devenv executable.

## Layout

```text
crates/                 Rust workspace crates
workspaces/postgres/    Shared PostgreSQL infrastructure
workspaces/migrations/  Central migration composition
workspaces/services/    TypeScript Effect services
ast-grep-rules/         Repository-specific structural rules
```

## Getting started

Enter the development environment:

```bash
direnv allow
```

Run all checks and tests:

```bash
devenv tasks run project:check
devenv tasks run project:test
```

Add implementations only when the project needs them; the initial repository
contains infrastructure boundaries without example application code.
