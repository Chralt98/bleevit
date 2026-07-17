//! Read-only assembly for the seven B2 `FutarchyApi` views that do not depend
//! on the not-yet-wired epoch pallet. The `impl FutarchyApi for Runtime` block
//! deliberately remains deferred until all eleven methods are available.

use alloc::vec::Vec;

use frame_support::traits::{fungibles::Inspect, Get};
use futarchy_primitives::{
    bounds, AccountId as ViewAccountId, Balance, BoundedVec, Branch, FixedU64, MarketId, NavView,
    OracleRoundView, ParamKey, ParamView, PositionView, ProposalClass, QueuedExecutionView,
    QuoteView, TradeSide, VaultState, WelfareView,
};

use crate::{AccountId, ForeignAssets, Runtime, USDC_ASSET_ID};

fn quote_sentinel(max_trade: Balance) -> QuoteView {
    QuoteView {
        cost: 0,
        fee: 0,
        p_after_1e9: FixedU64(0),
        max_trade,
        within_domain: false,
    }
}

/// Assemble `FutarchyApi::quote` per 02 §3/§4 and 04 §4/§6.
/// Missing, closed, overflowing, inventory-invalid, or out-of-domain books use
/// the G-1 zero sentinel; an existing book retains its real `b/4` maximum.
pub fn quote(market: MarketId, side: TradeSide, amount: Balance) -> QuoteView {
    let Some(book) = pallet_market::Markets::<Runtime>::get(market) else {
        return quote_sentinel(0);
    };
    let max_trade = pallet_market::core_market::max_trade_amount(book.b);
    match pallet_market::core_market::quote(
        &book,
        side,
        amount,
        <Runtime as pallet_market::Config>::Fee::get(),
    ) {
        Ok(view) => view,
        Err(_) => quote_sentinel(max_trade),
    }
}

fn push_position(
    out: &mut BoundedVec<PositionView, { bounds::MAX_ACCOUNT_POSITIONS }>,
    who: &AccountId,
    position: futarchy_primitives::PositionId,
    vault_state: VaultState,
) -> bool {
    let balance = pallet_conditional_ledger::Positions::<Runtime>::get(position, who);
    balance == 0
        || out
            .try_push(PositionView {
                position,
                balance,
                vault_state,
            })
            .is_ok()
}

/// Assemble `FutarchyApi::account_positions` per 02 §3/§7.4 and 03 §2.
/// Proposal vaults sort by proposal id and precede Baseline vaults sorted by
/// epoch; each vault uses conditional-ledger-core's canonical instrument order.
/// Truncation at 64 is deterministic. User accounts cannot exceed 64, while
/// 13 §4 explicitly exempts protocol accounts, so only those can be truncated.
pub fn account_positions(
    who: ViewAccountId,
) -> BoundedVec<PositionView, { bounds::MAX_ACCOUNT_POSITIONS }> {
    let who = AccountId::new(who);
    let mut out = BoundedVec::new();
    let mut proposals =
        pallet_conditional_ledger::Vaults::<Runtime>::iter_keys().collect::<Vec<_>>();
    proposals.sort_unstable();
    for proposal in proposals {
        let Some(vault) = pallet_conditional_ledger::Vaults::<Runtime>::get(proposal) else {
            continue;
        };
        for position in pallet_conditional_ledger::core_ledger::proposal_positions(proposal) {
            if !push_position(&mut out, &who, position, vault.state) {
                return out;
            }
        }
    }

    let mut baselines =
        pallet_conditional_ledger::BaselineVaults::<Runtime>::iter_keys().collect::<Vec<_>>();
    baselines.sort_unstable();
    for epoch in baselines {
        let Some(vault) = pallet_conditional_ledger::BaselineVaults::<Runtime>::get(epoch) else {
            continue;
        };
        // 02 §4 freezes PositionView to proposal `VaultState`, while 03 §2.3
        // gives Baseline vaults the distinct `BaselineState`. Open is exact;
        // for Settled, preserve terminality and score with Accept as a
        // representation-only sentinel. Consumers MUST ignore `winner` for a
        // `PositionId::Baseline` because Baseline instruments have no branch.
        let state = match vault.state {
            pallet_conditional_ledger::core_ledger::BaselineState::Open => VaultState::Open,
            pallet_conditional_ledger::core_ledger::BaselineState::Settled(s) => {
                VaultState::ScalarSettled {
                    winner: Branch::Accept,
                    s,
                }
            }
        };
        for position in pallet_conditional_ledger::core_ledger::baseline_positions(epoch) {
            if !push_position(&mut out, &who, position, state) {
                return out;
            }
        }
    }
    out
}

/// Assemble `FutarchyApi::execution_queue` per 02 §3/§4 and 09 §1.
/// The pallet accessor single-homes queue ordering, ratification, and blocked-
/// meter semantics; defensive truncation retains the first 32 proposal ids.
pub fn execution_queue() -> BoundedVec<QueuedExecutionView, { bounds::MAX_LIVE_PROPOSALS }> {
    let mut out = BoundedVec::new();
    for view in pallet_execution_guard::Pallet::<Runtime>::queue_view() {
        if out.try_push(view).is_err() {
            break;
        }
    }
    out
}

fn welfare_sentinel(epoch: u32, spec_version: u16, reserve_flag: bool) -> WelfareView {
    WelfareView {
        epoch,
        spec_version,
        s_pillar_1e9: FixedU64(0),
        c_onchain_1e9: FixedU64(0),
        c_attested_1e9: FixedU64(0),
        p_pillar_1e9: FixedU64(0),
        a_pillar_1e9: FixedU64(0),
        gate_s_1e9: FixedU64(0),
        gate_c_1e9: FixedU64(0),
        w_current_1e9: FixedU64(0),
        s_breached: false,
        c_breached: false,
        reserve_flag,
    }
}

/// Assemble `FutarchyApi::welfare_current` per 02 §3/§4 and 05 §4.6.
/// A spec version is active only when every component's activation epoch has
/// arrived, matching welfare-core's snapshot admission rule; the greatest
/// active version is selected deterministically. No active version or missing
/// snapshot returns the G-1 zero sentinel while retaining epoch/version and the
/// real reserve flag. Oracle reserve health is authoritative; constitution bit
/// 7 is only its 02 §7.3 mirror.
pub fn welfare_current() -> WelfareView {
    let epoch = <Runtime as pallet_welfare::Config>::CurrentEpoch::get();
    let spec_version = pallet_welfare::MetricSpecs::<Runtime>::iter()
        .filter_map(|(version, specs)| {
            (!specs.is_empty() && specs.iter().all(|spec| spec.activation_epoch <= epoch))
                .then_some(version)
        })
        .max()
        .map_or(0, |version| version);
    let reserve_flag = pallet_oracle::Pallet::<Runtime>::reserve_unhealthy();
    match pallet_welfare::Pallet::<Runtime>::welfare_state().current_view(
        epoch,
        spec_version,
        reserve_flag,
    ) {
        Ok(view) => view,
        Err(_) => welfare_sentinel(epoch, spec_version, reserve_flag),
    }
}

/// Assemble `FutarchyApi::params` per 02 §3/§4 and 13 reading rule 7.
/// Unknown keys are skipped and found keys retain input order (including
/// duplicates). `max_delta` is the maximum absolute step from the current
/// value, as single-homed in `ParamRecord::max_delta_allowance`; zero means no
/// per-step rule. Malformed records are skipped rather than presented as
/// unbounded. Cooldowns use the live `epoch.length`, saturating at `u32::MAX`.
pub fn params(
    keys: BoundedVec<ParamKey, { bounds::MAX_PARAM_KEYS }>,
) -> BoundedVec<ParamView, { bounds::MAX_PARAM_KEYS }> {
    let epoch_length =
        pallet_constitution::Params::<Runtime>::get(pallet_constitution::key16(b"epoch.length"))
            .and_then(|record| match record.value {
                pallet_constitution::ParamValue::U32(value) => Some(value),
                _ => None,
            });
    let mut out = BoundedVec::new();
    for key in keys {
        let Some(record) = pallet_constitution::Params::<Runtime>::get(key) else {
            continue;
        };
        let Ok(max_delta) = record.max_delta_allowance() else {
            continue;
        };
        let cooldown_blocks = if record.cooldown_epochs == 0 {
            0
        } else {
            match epoch_length {
                Some(length) => record.cooldown_epochs.saturating_mul(length),
                None => u32::MAX,
            }
        };
        if out
            .try_push(ParamView {
                key,
                value: record.value.as_u128(),
                min: record.min.as_u128(),
                max: record.max.as_u128(),
                max_delta,
                cooldown_blocks,
                last_change: record.last_change_block,
                class: record.class.as_proposal_class(),
            })
            .is_err()
        {
            break;
        }
    }
    out
}

/// Assemble contract-v4 `FutarchyApi::nav` per 02 §3/§4 and 08
/// §1.1/§1.2/§4.1. POL includes both proposal and dedicated Baseline
/// lines. Insurance comes from the actual INSURANCE USDC custody account.
pub fn nav() -> NavView {
    let components = pallet_futarchy_treasury::Pallet::<Runtime>::nav();
    let treasury = pallet_futarchy_treasury::Pallet::<Runtime>::treasury();
    let proposal_pol = pallet_futarchy_treasury::Pallet::<Runtime>::line_balance(
        pallet_futarchy_treasury::BudgetLine::Pol,
    );
    let baseline_pol = pallet_futarchy_treasury::Pallet::<Runtime>::line_balance(
        pallet_futarchy_treasury::BudgetLine::PolBaseline,
    );
    let insurance = <ForeignAssets as Inspect<AccountId>>::balance(
        USDC_ASSET_ID,
        &crate::configs::insurance_account(),
    );
    NavView {
        total: components.nav,
        main: treasury.main_usdc,
        pol: proposal_pol.saturating_add(baseline_pol),
        insurance,
        keeper: pallet_futarchy_treasury::Pallet::<Runtime>::line_balance(
            pallet_futarchy_treasury::BudgetLine::Keeper,
        ),
        oracle: pallet_futarchy_treasury::Pallet::<Runtime>::line_balance(
            pallet_futarchy_treasury::BudgetLine::Oracle,
        ),
        rewards: pallet_futarchy_treasury::Pallet::<Runtime>::line_balance(
            pallet_futarchy_treasury::BudgetLine::Rewards,
        ),
        stream_remainders: treasury.open_stream_remainders(),
        obligations: treasury.obligations(),
        haircut_flag: components.reserve_impaired,
        spendable_nav: components.spendable_nav,
        meter_utilization_bps: components.meter_utilization_bps,
        class_floors: [
            pallet_futarchy_treasury::Pallet::<Runtime>::floor(ProposalClass::Param),
            pallet_futarchy_treasury::Pallet::<Runtime>::floor(ProposalClass::Treasury),
            pallet_futarchy_treasury::Pallet::<Runtime>::floor(ProposalClass::Code),
            pallet_futarchy_treasury::Pallet::<Runtime>::floor(ProposalClass::Meta),
        ],
    }
}

/// Assemble `FutarchyApi::open_oracle_rounds` per 02 §3/§4/§7.2 and
/// 07 §5. `escalated` means a prior round advanced the game (`round > 1`):
/// `challenger.is_some()` is only a live challenge in the current round and is
/// cleared when escalation occurs. Results sort by the frozen triple key.
pub fn open_oracle_rounds() -> BoundedVec<OracleRoundView, { bounds::MAX_OPEN_ORACLE_ROUNDS }> {
    let mut rounds = pallet_oracle::Rounds::<Runtime>::iter_values().collect::<Vec<_>>();
    rounds.sort_unstable_by_key(|round| (round.component, round.epoch, round.spec_version));
    let mut out = BoundedVec::new();
    for round in rounds {
        if out
            .try_push(OracleRoundView {
                component: round.component,
                epoch: round.epoch,
                spec_version: round.spec_version,
                round: round.round,
                reporter: round.reporter,
                value_1e9: round.value,
                evidence_hash: round.evidence_hash,
                bond: round.bond,
                challenge_deadline: round.challenge_deadline,
                acked_by_watchtowers: round.acks,
                escalated: round.round > 1,
            })
            .is_err()
        {
            break;
        }
    }
    out
}
