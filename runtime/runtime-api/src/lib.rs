#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

use futarchy_primitives::{
    AccountId, Balance, BoundedVec, CohortSummaryView, DecisionStatsView, EpochStatusView,
    MarketId, NavView, OracleRoundView, ParamKey, ParamView, PositionView, ProposalId,
    ProposalSummaryView, QueuedExecutionView, QuoteView, TradeSide, WelfareView,
};

/// Versioned chain↔frontend runtime API surface from architecture 02 §3.
///
/// In the production SDK runtime this trait is declared with `sp_api::decl_runtime_apis!`;
/// this workspace model keeps the same method set and SCALE view types dependency-light so
/// contract compatibility can be tested before full node assembly.
pub trait FutarchyApi {
    fn epoch_status(&self) -> EpochStatusView;
    fn proposal_summaries(&self) -> BoundedVec<ProposalSummaryView, 32>;
    fn quote(&self, market: MarketId, side: TradeSide, amount: Balance) -> QuoteView;
    fn decision_stats(&self, pid: ProposalId) -> Option<DecisionStatsView>;
    fn account_positions(&self, who: AccountId) -> BoundedVec<PositionView, 64>;
    fn execution_queue(&self) -> BoundedVec<QueuedExecutionView, 32>;
    fn welfare_current(&self) -> WelfareView;
    fn params(&self, keys: BoundedVec<ParamKey, 64>) -> BoundedVec<ParamView, 64>;
    fn nav(&self) -> NavView;
    fn recent_cohorts(&self) -> BoundedVec<CohortSummaryView, 32>;
    fn open_oracle_rounds(&self) -> BoundedVec<OracleRoundView, 192>;
}

pub const FUTARCHY_API_METHODS: &[&str; 11] = &[
    "epoch_status",
    "proposal_summaries",
    "quote",
    "decision_stats",
    "account_positions",
    "execution_queue",
    "welfare_current",
    "params",
    "nav",
    "recent_cohorts",
    "open_oracle_rounds",
];
