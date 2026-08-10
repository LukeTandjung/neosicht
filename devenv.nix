{pkgs, ...}: {
  packages = with pkgs; [
    ast-grep
    pre-commit
    python3
  ];

  languages.rust.enable = true;

  tasks."project:check".exec = ''
    cargo check --workspace
  '';

  tasks."project:test".exec = ''
    cargo test --workspace
  '';

  tasks."app:build".exec = ''
    nix build .#neosicht
  '';

  tasks."app:run".exec = ''
    output="$(nix build --no-link --print-out-paths .#neosicht)"
    open "$output/Applications/Neosicht.app"
  '';

  enterShell = ''
    if git_dir="$(git rev-parse --git-dir 2>/dev/null)"; then
      if [ ! -x "$git_dir/hooks/pre-commit" ] || [ ! -x "$git_dir/hooks/commit-msg" ]; then
        pre-commit install --hook-type pre-commit --hook-type commit-msg
      fi
    fi
  '';
}
