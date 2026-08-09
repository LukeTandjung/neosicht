# neosicht

An alternate desktop shell for macOS, built with Rust and GPUI. neosicht keeps
WindowServer and normal macOS applications while replacing visible shell chrome
with a custom top bar.

See [`docs/PLAN.md`](docs/PLAN.md) for the product and architecture plan and
[`docs/EXPERIMENTS.md`](docs/EXPERIMENTS.md) for the Phase-0 findings.

## Development

Enter the development environment:

```bash
direnv allow
```

Run the bar:

```bash
cargo run --bin neosicht
```

Run checks and tests:

```bash
devenv tasks run project:check
devenv tasks run project:test
```

## Layout

```text
crates/neosicht/  Shell foundation and GPUI bar
docs/             Plan and experiment findings
ast-grep-rules/    Repository-specific structural rules
```
