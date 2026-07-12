Bring this repository's living documents back in line with reality (AGENTS.md is
binding). Scope: README.md, PLAN.md, AGENTS.md, CLAUDE.md, .claude/ assets, .codex/
playbooks. docs/architecture/ is the frozen spec — NEVER modify it; it is not part
of this task.

1. GROUND TRUTH first — never write from memory:
   git status --porcelain; git diff --stat HEAD; git log --oneline -10; the actual
   file tree; Cargo workspace members; package manifests; CI jobs and test entry
   points that exist right now.

2. PER-FILE CONTRACT.
   - PLAN.md: statuses match reality (red gates ⇒ not ✅); Current focus names the
     true next step; Session log has a row for today's work; Spec questions /
     Verification log / Decision log / Audit log reflect what actually happened.
     PLAN.md stays reference-only — milestone rows cite docs/architecture/ sections,
     never restate spec content.
   - README.md: status, repository map, and commands are currently true; links resolve.
   - AGENTS.md: rules, session protocol, quality gates, layout table match how the
     repo works now.
   - CLAUDE.md: skills/subagents/hooks tables match the files under .claude/.
   - .codex/prompts/: still procedurally equivalent to the .claude/skills/ they mirror.

3. APPLY minimal, surgical edits. Append to logs, never rewrite their history. Label
   planned things as planned — never document aspirations as facts. Verify every
   relative link you touched resolves.

4. REPORT: each file changed with a one-line reason, plus anything that needs a human
   decision (flag it, don't decide it).
