---
paths: ["reference-model/**"]
---

# Reference model rules (independent executable spec)

The Python reference model exists to catch bugs in the Rust implementation by
**independent derivation** (15 §4.4). Its value dies if it mirrors the implementation.

1. **Independence is the point.** Never port, transcribe, or "align" Rust runtime code
   into the model — implement from `docs/architecture/` text alone. When model and
   pallet disagree, that is a finding to investigate against the spec, not a diff to
   silently make green on either side.
2. **Vectors are generated, never hand-maintained.** The normative LMSR vectors V1–V6
   and the MPFR differential corpus are regenerated in CI from this model on every
   change (04 §5); a hand-edited expected value is a defect (the shipped hand-computed
   V1 error is the standing justification).
3. **Determinism.** Fixed seeds, no wall-clock, no environment-dependent behavior;
   byte-identical JSON output for identical inputs (stable key order, explicit precision).
   MPFR reference math runs at 256-bit precision.
4. **Shared corpus schema is contract-owned.** The JSON vector schema is defined by
   `02-integration-contract.md` §11 — the same artifacts feed the backend differential
   suites and the frontend's TypeScript port. Changing the schema is a change to doc 02
   (rule R-1): bump `INTEGRATION_CONTRACT_VERSION` and keep both consumers in step.
5. **Scope.** The model covers LMSR cost/pricing, TWAP, the ledger operation semantics
   (incl. gate/Baseline/VOID and rounding), the welfare pipeline, and the decision rule
   with reason codes (05 §4.4 bit-identical requirement), plus the treasury arithmetic
   of 08 (its worked examples are normative for Phase 0).
