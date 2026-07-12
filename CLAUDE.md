# CLAUDE.md

@AGENTS.md

Everything above (imported from AGENTS.md) is binding. Below is the Claude-Code-specific wiring.

## Skills (invoke with `/name`; auto-invoke when the description matches)

| Skill | Use for |
|---|---|
| `/implement [id]` | The session driver: one PLAN.md milestone, spec-first, verified, PLAN updated. Default entry point for "continue"/"next step". |
| `/spec-audit [scope]` | Compliance sweep of implemented code against `docs/architecture/` (report-only; logs to PLAN.md · Audit log). |
| `/sync-docs` | Re-true README/PLAN/AGENTS/CLAUDE and the `.claude`/`.codex` assets against the actual repo. |
| `/new-pallet <name>` | Scaffold a FRAME pallet with spec-cited stubs, mock, test/benchmark stubs, try-state hook. |

## Subagents (delegate via the Agent tool)

| Agent | Role |
|---|---|
| `spec-reviewer` | Read-only compliance audit of a component vs its owning doc. Run it before marking any milestone ✅ (R-6). |
| `test-engineer` | Authors the doc-15 test obligations (PT suites, limit-coverage, negative origin tests, try-state, differential vectors). |
| `doc-curator` | End-of-session living-document sync when the delta is large. |

## Hooks (installed via `.claude/settings.json` — expect these behaviors)

- **SessionStart** injects git state + PLAN.md focus/milestones/last log rows. Trust it
  for orientation, but still read PLAN.md before implementing.
- **PreToolUse guard** denies any write/edit/mutating-bash touching `docs/architecture/`
  (rule R-1). If it fires unexpectedly on a read-only command, rephrase the command
  rather than looking for a bypass. Change control uses
  `.claude/architecture-amendment.flag` — only ever create it on explicit user
  instruction (AGENTS.md · Amending the architecture).
- **Stop guard** blocks ending a session when the tree changed but PLAN.md wasn't
  updated, or when the amendment flag is still present. Comply (update PLAN.md /
  finish change control) instead of retrying.

Permissions: `Edit`/`Write` into `docs/architecture/**` are deny-listed; common
read-only git and cargo commands are pre-allowed; `git push` always asks.

## Memory notes

Auto-memory exists for this project. PLAN.md — not memory — is the canonical
implementation status; keep memories as pointers (e.g. "status lives in PLAN.md"),
never as duplicated status that can go stale.
