{pkgs, ...}: {
  packages = with pkgs; [
    ast-grep
    deno
    pre-commit
    python3
  ];

  languages.rust.enable = true;
  services.postgres = {
    enable = true;
    initialDatabases = [{name = "app";}];
  };

  env.PGDATABASE = "app";

  tasks."project:check:typescript".exec = "deno task check";
  tasks."project:check:rust".exec = ''
    if find crates -mindepth 2 -name Cargo.toml -print -quit | grep -q .; then
      cargo check --workspace
    else
      echo "No Rust crates yet; skipping Cargo check."
    fi
  '';
  tasks."project:check".after = [
    "project:check:typescript"
    "project:check:rust"
  ];

  tasks."project:test:typescript".exec = "deno task test";
  tasks."project:test:rust".exec = ''
    if find crates -mindepth 2 -name Cargo.toml -print -quit | grep -q .; then
      cargo test --workspace
    else
      echo "No Rust crates yet; skipping Cargo tests."
    fi
  '';
  tasks."project:test".after = [
    "project:test:typescript"
    "project:test:rust"
  ];

  enterShell = ''
    if git_dir="$(git rev-parse --git-dir 2>/dev/null)"; then
      if [ ! -x "$git_dir/hooks/pre-commit" ] || [ ! -x "$git_dir/hooks/commit-msg" ]; then
        pre-commit install --hook-type pre-commit --hook-type commit-msg
      fi
    fi
  '';
}
