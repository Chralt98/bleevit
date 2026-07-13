#!/usr/bin/env bash
# Stop hook — enforces rule R-11 (AGENTS.md): README.md's pinned opening
# (thank-you to Prof. Robin Hanson) and closing (Bon appétit) lines are fixed
# verbatim text set by explicit user instruction (2026-07-13) and must survive
# every edit to README.md. Blocks at most once (stop_hook_active).
set -euo pipefail

INPUT=$(cat)
ACTIVE=$(jq -r '.stop_hook_active // false' <<<"$INPUT")
[ "$ACTIVE" = "true" ] && exit 0

cd "${CLAUDE_PROJECT_DIR:-$(pwd)}"
[ -f README.md ] || exit 0

OPENING="Futarchy was invented by Prof. Robin Hanson — thank you for your work; this project exists to build one."
CLOSING="You theorized it, we are cooking it. Bon appétit, Prof. Hanson."

# First paragraph after the "# Bleavit" heading, unwrapped onto one line.
ACTUAL_OPENING=$(awk '
  /^# Bleavit/ { found=1; next }
  found && NF==0 && buf=="" { next }
  found && NF==0 && buf!="" { exit }
  found { buf = buf (buf=="" ? "" : " ") $0 }
  END { print buf }
' README.md)

# Last non-empty line of the file, trimmed.
ACTUAL_CLOSING=$(awk 'NF{line=$0} END{print line}' README.md | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')

FAIL=""
[ "$ACTUAL_OPENING" = "$OPENING" ] || FAIL="opening"
[ "$ACTUAL_CLOSING" = "$CLOSING" ] || FAIL="${FAIL:+$FAIL,}closing"

[ -z "$FAIL" ] && exit 0

jq -n --arg fail "$FAIL" --arg opening "$OPENING" --arg closing "$CLOSING" \
  '{decision:"block", reason:("README.md is missing its pinned " + $fail + " line(s) (rule R-11, AGENTS.md — standing user instruction, 2026-07-13). Restore verbatim: the first paragraph immediately after the \"# Bleavit\" heading must read \"" + $opening + "\"; the last line of the file must read \"" + $closing + "\". Do not paraphrase either line.")}'
