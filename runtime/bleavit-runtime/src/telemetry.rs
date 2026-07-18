//! Monitoring-only `TelemetryApi` implementation (12 §6.3, B13).
//!
//! Explicitly OUTSIDE the 02 integration contract: the frontend never consumes
//! this surface, it carries no contract version, and its shape may change
//! without a 02 §13 bump. Consumed by the §6.3 chain exporter via `state_call`.
//! Solvency-relevant methods MUST read the same quantities the owning pallet's
//! try-state compares — this module is a window onto audited state, never a
//! second bookkeeping.
