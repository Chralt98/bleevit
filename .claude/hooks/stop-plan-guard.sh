#!/usr/bin/env bash
# Stop hook — enforces rule R-3 (AGENTS.md): PLAN.md must reflect every session
# that changed the repository. Blocks at most once (stop_hook_active).
set -euo pipefail

INPUT=$(cat)
ACTIVE=$(jq -r '.stop_hook_active // false' <<<"$INPUT")
[ "$ACTIVE" = "true" ] && exit 0

cd "${CLAUDE_PROJECT_DIR:-$(pwd)}"
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || exit 0

# Repo changed but PLAN.md untouched → ask for the doc-sync pass.
CHANGES=$(git status --porcelain 2>/dev/null | grep -vE '\.claude/settings\.local\.json$' || true)
[ -z "$CHANGES" ] && exit 0

NONPLAN=$(grep -vE '(^|[[:space:]])PLAN\.md$' <<<"$CHANGES" || true)
PLAN_TOUCHED=$(git status --porcelain -- PLAN.md 2>/dev/null || true)

if [ -n "$NONPLAN" ] && [ -z "$PLAN_TOUCHED" ]; then
  jq -n '{decision:"block", reason:"The working tree has changes but PLAN.md was not updated (rule R-3, AGENTS.md). Before stopping: (1) set the status of the affected milestone(s) in PLAN.md; (2) append a Session log row: | date | milestone | what was done | what comes next |; (3) if the repo layout, commands, or workflow changed, refresh README.md / AGENTS.md / CLAUDE.md as well (or run /sync-docs). If the changes pre-date this session or are trivial, still record one Session log line saying exactly that, so the next session inherits accurate state."}'
  exit 0
fi
exit 0
