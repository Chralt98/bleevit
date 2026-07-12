Run ONE spec-driven implementation increment for this repository, following AGENTS.md
(which you must treat as binding). Protocol:

1. ORIENT. Read PLAN.md fully: Current focus, milestone tables, Spec questions, last
   Session log rows. Target = the milestone named at the end of this prompt if any;
   otherwise the 🔨 in-progress milestone; otherwise the first ⬜ milestone whose
   Depends column is all ✅. Confirm the scope in one sentence before coding.

2. READ THE SPEC FIRST. docs/architecture/ is the frozen, authoritative specification
   — you MUST NOT modify, rename, delete, or add anything under it, under any
   circumstances. Read every section in the milestone's Spec column, plus the relevant
   parts of 02-integration-contract.md (frozen names/types), 13-parameters.md (the only
   source of numeric values), and 15-invariants-and-testing.md (verification duties).
   If the spec is ambiguous or contradictory: record it in PLAN.md · Spec questions
   with a precise citation; proceed only if a conservative reading is safe, else mark
   the milestone ⛔ and stop.

3. IMPLEMENT. One milestone only. Non-negotiables: no floats / no wall-clock phase
   logic / checked or saturating arithmetic and typed errors everywhere (status-quo
   default, G-1); bounded collections with bounds from 13 §4; storage/event/call names
   byte-identical to 02; parameters never hardcoded (kernel constants from
   futarchy-primitives, tunables from pallet-constitution); rounding against the
   claimant; explicit origin checks; try-state per 15 §1. Frontend: INV-FE-1…15 bind
   (15 §2) — finalized verified reads, provenance typing, package firewall, no telemetry.

4. VERIFY. Write the tests the milestone's Verify column and doc 15 demand. Run the
   gates: cargo fmt --all -- --check · cargo clippy --workspace --all-targets -- -D warnings ·
   cargo test --workspace (scale to what exists; frontend: lint/typecheck/test/build).
   Then re-read the owning spec sections and self-review the diff against them,
   adversarially.

5. CLOSE. Update PLAN.md in this same session: milestone status (✅ only with green
   gates and no known spec deviations; else 🔨 with exact resume notes), Current focus,
   and a Session log row `| YYYY-MM-DD | <id> | <done> | <next> |`. Refresh README.md /
   AGENTS.md if repo shape or commands changed. Report honestly: delivered work, gate
   output (verbatim on failure), open questions, suggested conventional-commit message
   with the milestone ID (e.g. `feat(ledger): split/merge families (A2)`). Do not
   commit or push unless the user asked.

Never end the session with the repo changed but PLAN.md stale.
