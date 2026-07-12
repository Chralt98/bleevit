#!/usr/bin/env bash
# PreToolUse hook — enforces rule R-1 (AGENTS.md): docs/architecture/** is frozen.
#
# Matched tools: Edit | Write | MultiEdit | NotebookEdit | Bash.
# - File tools: denies any edit/write whose target lies in docs/architecture/.
# - Bash: denies commands that would mutate docs/architecture/ (rm/mv/sed -i/
#   redirects/tee/git restore/cp-into/...). Read-only access is always allowed.
#
# Escape hatch (change control, see AGENTS.md · "Amending the architecture"):
# if .claude/architecture-amendment.flag exists, the guard stands down. The flag
# may be created ONLY on explicit user instruction, and the Stop hook refuses to
# end the session while it still exists.
#
# This guard prevents accidents, not adversaries; git history is the backstop.
set -euo pipefail

INPUT=$(cat)
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(pwd)}"

if [ -f "$PROJECT_DIR/.claude/architecture-amendment.flag" ]; then
  exit 0
fi

TOOL=$(jq -r '.tool_name // empty' <<<"$INPUT")

deny() {
  jq -n --arg reason "$1" \
    '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:$reason}}'
  exit 0
}

REASON_EDIT="docs/architecture/ is the frozen, authoritative specification (rule R-1, AGENTS.md). Implementation conforms to the spec — never the other way around. If the spec looks wrong, ambiguous, or contradictory, record the issue in PLAN.md under 'Spec questions' and ask the user. Amendments require the explicit change-control procedure in AGENTS.md."

REASON_BASH="This command could modify or delete files under docs/architecture/ — the frozen specification (rule R-1, AGENTS.md). Read-only access (cat, grep, diff, git log) is always fine. If you were not trying to modify the spec, rephrase the command so no mutating operation touches a docs/ path. Spec problems go to PLAN.md · 'Spec questions'; amendments require the change-control procedure in AGENTS.md."

case "$TOOL" in
  Edit|Write|MultiEdit|NotebookEdit)
    FILE=$(jq -r '.tool_input.file_path // .tool_input.notebook_path // empty' <<<"$INPUT")
    case "$FILE" in
      *docs/architecture/*) deny "$REASON_EDIT" ;;
    esac
    ;;
  Bash)
    CMD=$(jq -r '.tool_input.command // empty' <<<"$INPUT")
    # Fast path: nothing here can touch the spec.
    grep -q 'docs' <<<"$CMD" || exit 0

    # A docs path token: "docs" or "docs/..." preceded by start/space/quote/=//.
    DOCS_TOKEN='(^|[[:space:]"'"'"'=/])docs(/[^;|&[:space:]"'"'"']*)?([[:space:]"'"'"']|$|[;|&])'

    # 1. Mutating verb with a docs path among its args (same shell segment).
    if grep -Eq '(^|[;&|[:space:]])(rm|rmdir|mv|unlink|shred|truncate|touch|mkdir|chmod|chown|ln)[[:space:]][^;|&]*docs(/|[[:space:]]|$|["'"'"'])' <<<"$CMD" \
       && grep -Eq "$DOCS_TOKEN" <<<"$CMD"; then
      deny "$REASON_BASH"
    fi
    # 2. In-place editors targeting a docs path.
    if grep -Eq '(^|[;&|[:space:]])(sed|perl)[[:space:]][^;|&]*(-[a-zA-Z]*i|--in-place)[^;|&]*docs(/|[[:space:]]|$)' <<<"$CMD"; then
      deny "$REASON_BASH"
    fi
    # 3. Output redirection or tee into a docs path.
    if grep -Eq '(>>?|[0-9]>>?)[[:space:]]*["'"'"']?[^;|&[:space:]]*docs/' <<<"$CMD" \
       || grep -Eq '(^|[;&|[:space:]])tee[[:space:]][^;|&]*docs/' <<<"$CMD"; then
      deny "$REASON_BASH"
    fi
    # 4. git operations that rewrite the working tree copy of docs/.
    if grep -Eq 'git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?(rm|mv|checkout|restore|clean)[^;|&]*docs(/|[[:space:]]|$)' <<<"$CMD"; then
      deny "$REASON_BASH"
    fi
    # 5. cp/rsync/install with a docs path as the final (destination) argument.
    if grep -Eq '(^|[;&|[:space:]])(cp|rsync|install)[[:space:]][^;|&]*[[:space:]]["'"'"']?[^;|&[:space:]]*docs/[^;|&[:space:]]*["'"'"']?[[:space:]]*($|[;|&])' <<<"$CMD"; then
      deny "$REASON_BASH"
    fi
    # 6. find ... -delete / xargs rm reaching into docs.
    if grep -Eq '(^|[;&|[:space:]])find[[:space:]][^;|&]*docs[^;|&]*-(delete|exec[[:space:]][^;|&]*(rm|mv|sed))' <<<"$CMD" \
       || { grep -Eq 'xargs[^;|&]*(rm|mv|sed[[:space:]][^;|&]*-[a-zA-Z]*i)' <<<"$CMD" && grep -Eq "$DOCS_TOKEN" <<<"$CMD"; }; then
      deny "$REASON_BASH"
    fi
    ;;
esac
exit 0
