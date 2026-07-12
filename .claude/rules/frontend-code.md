---
paths: ["frontend/**"]
---

# Frontend code rules (canonical decentralized client)

The frontend invariants INV-FE-1…15 (`docs/architecture/15 §2`) are normative and
certification binds to their exact texts. Practical consequences:

1. **Authoritative reads (INV-FE-1).** Transaction-critical values come only from
   finalized, light-client-verified state. RPC-fallback or provider data is never
   promoted to verified; verified status requires a light-client re-read.
2. **Provenance typing (INV-FE-9, 10 §2.1).** Every displayed value carries a typed
   status (`verified-finalized` / `verified-best` / `derived-local` / `provider` /
   `stale-cache`); `Finalized<T>` is constructible only inside `packages/chain`.
   UI components reject unlabeled values by type. Never cast around this.
3. **Package firewall (INV-FE-3, 10 §10).** Respect the dependency-cruiser boundaries:
   `wallet` never imports `providers`/`local-index`; nothing above `chain` bypasses it;
   `src/tx/**` never imports `src/history/**`. Provider data never satisfies a precondition.
4. **Pre-sign refresh (INV-FE-2, 11 §11.4).** Every submit path goes through the
   structural `refreshAndGate` — never add a code path that reaches a signer without it.
5. **Zero infrastructure (INV-FE-4/6).** Every protocol workflow must work with no
   indexer, no RPC, no provider, cleared storage. If a feature needs a server, it is
   out of scope — do not centralize it.
6. **No telemetry, no remote config (INV-FE-13).** No analytics, no fetch-to-configure
   patterns; behavior changes only by shipping a new verifiable release.
7. **No hardcoded chain constants.** Everything in 02 §9 is read from chain
   metadata/storage; the no-literal lint gate fails the release otherwise. The TS
   protocol math (`packages/protocol`) must match the CI-regenerated vector corpus
   (04 §5, 15 §4.4) — never hand-adjust an expected value.
8. **Local storage is disposable (INV-FE-7).** The transaction path never reads
   IndexedDB; rebuilds are automatic; treat eviction as a performance event.
9. **Fail safe (INV-FE-12).** Unknown runtime ⇒ explicit `restricted`/`read-only-
   incompatible` modes; undecodable data renders as raw SCALE with a warning; never
   guess at encodings.
10. **Pinned versions.** The stack pins live in 01 §9 / 10 — PAPI 2.x, smoldot 3.x,
    Vite 8, Dexie 4. Do not bump majors without a PLAN.md decision-log entry.
