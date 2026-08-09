#!/usr/bin/env bash
# Exp 0 helper: prints the frontmost application whenever it changes.
# Run in a terminal while clicking the exp-panel bar. If "exp-panel" ever
# appears here after a bar interaction, the non-activating panel FAILED.
set -euo pipefail

previous=""
while true; do
  current=$(lsappinfo info -only name "$(lsappinfo front)" | cut -d'"' -f4)
  if [[ "$current" != "$previous" ]]; then
    printf '%s frontmost: %s\n' "$(date '+%H:%M:%S')" "$current"
    previous="$current"
  fi
  sleep 0.3
done
