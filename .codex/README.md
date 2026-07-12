# .codex/ — Codex CLI playbooks for this repository

Codex reads the root [`AGENTS.md`](../AGENTS.md) automatically — the rules there
(frozen architecture, spec-first, living documents, quality gates) bind Codex
sessions exactly as they bind Claude Code sessions.

This directory adds **session playbooks**: self-contained prompts that mirror the
Claude Code skills in `.claude/skills/`, so both agents run the same protocol.

## Usage

Interactive:

```sh
codex "$(cat .codex/prompts/implement-next.md)"
codex "$(cat .codex/prompts/implement-next.md) Target milestone: A2"
```

Non-interactive:

```sh
codex exec "$(cat .codex/prompts/spec-audit.md) Scope: ledger"
```

| Playbook | Mirrors | Purpose |
|---|---|---|
| `prompts/implement-next.md` | `/implement` | One spec-driven milestone increment, PLAN.md updated |
| `prompts/spec-audit.md` | `/spec-audit` | Compliance audit vs `docs/architecture/` (report-only) |
| `prompts/sync-docs.md` | `/sync-docs` | Re-true the living documents against the repo |

## Keeping in sync

These playbooks and the `.claude/skills/` procedures must stay equivalent; whoever
changes one updates the other (`/sync-docs` checks this). Codex has no equivalent of
the Claude hooks, so the playbooks restate the two hook-enforced rules explicitly:
never modify `docs/architecture/`, and never end a session that changed the repo
without updating `PLAN.md`.
