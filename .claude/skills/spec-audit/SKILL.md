---
name: spec-audit
description: Audit implemented code against the frozen architecture in docs/architecture/. Use before commits or releases, after refactors, when the user asks whether something is spec-compliant, or as a periodic sweep. Delegates per-component review to spec-reviewer subagents and records the result in PLAN.md's Audit log.
argument-hint: "[component | path | milestone-id | all]"
---

# Architecture-compliance audit

Report-only: this skill finds and records deviations; it fixes nothing unless the
user explicitly asks afterwards.

## 1. Resolve scope

- `$ARGUMENTS` may name a component (`ledger`, `market`, …), a path, a milestone ID,
  or `all`. Default when empty: every component with changes since the last audit
  recorded in PLAN.md · Audit log (fall back to: everything implemented so far).
- Map scope to owning docs (spec-reviewer knows the mapping; pass it explicitly anyway):
  constitution→06/13 · ledger→03 · market→04 · epoch/welfare→05 · governance/guardian/
  attestor/origins→06 · oracle/registry→07 · treasury→08 · execution-guard→09 ·
  runtime assembly→01 §5–6 · frontend→10/11 · release tooling→12 · reference model→04 §5.

## 2. Fan out

- Launch one `spec-reviewer` subagent per component in scope (in parallel when several).
  Each returns a verdict + deviation table + SPEC-QUESTION list.

## 3. Aggregate and record

- Merge results, deduplicate, and order: blockers → majors → minors → spec questions.
- Append one row to PLAN.md · **Audit log**:
  `| YYYY-MM-DD | <scope> | <verdict summary, e.g. 2 blocker / 3 major / 5 minor> | <pointer> |`
- Add genuinely new SPEC-QUESTIONs to PLAN.md · **Spec questions** (cite doc §; never
  edit docs/architecture/ itself).

## 4. Report

- Present the full deviation table with spec citations and `path:line` locations.
- Recommend, don't act: which blockers must be fixed before the next milestone, and
  whether any finding invalidates a ✅ milestone (if so, propose flipping it back to 🔨 —
  flip it only with user consent).
