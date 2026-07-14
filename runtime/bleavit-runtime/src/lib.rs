#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use futarchy_primitives::{
    chain_identity, currency, kernel, Balance, BlockNumber, ParamKey, H256,
    INTEGRATION_CONTRACT_VERSION,
};
use pallet_origins::{CallDomain, Origin, RuntimeCall, SafetyFilter};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub const MILLISECS_PER_BLOCK: u64 = kernel::MILLISECS_PER_BLOCK;
pub const SS58_PREFIX: u16 = chain_identity::SS58_PREFIX;
pub const RUNTIME_SPEC_NAME: &[u8] = b"bleavit";
pub const RUNTIME_IMPL_NAME: &[u8] = b"bleavit-runtime";
pub const RUNTIME_SPEC_VERSION: u32 = 1;
pub const TRANSACTION_VERSION: u32 = INTEGRATION_CONTRACT_VERSION;
pub const VIT_DECIMALS: u8 = currency::VIT_DECIMALS;
pub const USDC_DECIMALS: u8 = currency::USDC_DECIMALS;
pub const USDC_ASSET_ID: u32 = 1;
/// Compact model identifier for `Location { parents: 1, X3(Parachain(1000), PalletInstance(50), GeneralIndex(1337)) }`.
pub const USDC_LOCATION: [u8; 32] = [
    1, 0, 0, 0, 232, 3, 0, 0, 50, 0, 0, 0, 57, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];
pub const FEE_VIT_USDC_RATE_KEY: ParamKey = *b"fee.vit_usdc_rat";

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum RuntimePallet {
    System,
    Timestamp,
    Balances,
    ForeignAssets,
    TransactionPayment,
    AssetTxPayment,
    Referenda,
    ConvictionVoting,
    Preimage,
    Scheduler,
    Utility,
    Proxy,
    Multisig,
    Migrations,
    MetadataHashExtension,
    Sudo,
    ParachainSystem,
    XcmpQueue,
    MessageQueue,
    Xcm,
    CollatorSelection,
    Session,
    Aura,
    Authorship,
    Constitution,
    ConditionalLedger,
    Market,
    Epoch,
    Welfare,
    Oracle,
    Registry,
    ExecutionGuard,
    FutarchyTreasury,
    Guardian,
    Attestor,
    Origins,
}

pub const STANDARD_PALLETS: &[RuntimePallet] = &[
    RuntimePallet::System,
    RuntimePallet::Timestamp,
    RuntimePallet::Balances,
    RuntimePallet::ForeignAssets,
    RuntimePallet::TransactionPayment,
    RuntimePallet::AssetTxPayment,
    RuntimePallet::Referenda,
    RuntimePallet::ConvictionVoting,
    RuntimePallet::Preimage,
    RuntimePallet::Scheduler,
    RuntimePallet::Utility,
    RuntimePallet::Proxy,
    RuntimePallet::Multisig,
    RuntimePallet::Migrations,
    RuntimePallet::MetadataHashExtension,
    RuntimePallet::Sudo,
    RuntimePallet::ParachainSystem,
    RuntimePallet::XcmpQueue,
    RuntimePallet::MessageQueue,
    RuntimePallet::Xcm,
    RuntimePallet::CollatorSelection,
    RuntimePallet::Session,
    RuntimePallet::Aura,
    RuntimePallet::Authorship,
];

pub const CUSTOM_PALLETS: &[RuntimePallet] = &[
    RuntimePallet::Constitution,
    RuntimePallet::ConditionalLedger,
    RuntimePallet::Market,
    RuntimePallet::Epoch,
    RuntimePallet::Welfare,
    RuntimePallet::Oracle,
    RuntimePallet::Registry,
    RuntimePallet::ExecutionGuard,
    RuntimePallet::FutarchyTreasury,
    RuntimePallet::Guardian,
    RuntimePallet::Attestor,
    RuntimePallet::Origins,
];

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum SystemCall {
    Remark,
    SetHeapPages,
    SetCode,
    SetCodeWithoutChecks,
    SetStorage,
    KillStorage,
    KillPrefix,
    AuthorizeUpgrade,
    AuthorizeUpgradeWithoutChecks,
    ApplyAuthorizedUpgrade,
}

impl SystemCall {
    pub const fn domain(self) -> CallDomain {
        match self {
            Self::Remark => CallDomain::Public,
            Self::AuthorizeUpgrade => CallDomain::InternalRoot,
            Self::ApplyAuthorizedUpgrade => CallDomain::Public,
            Self::SetHeapPages
            | Self::SetCode
            | Self::SetCodeWithoutChecks
            | Self::SetStorage
            | Self::KillStorage
            | Self::KillPrefix
            | Self::AuthorizeUpgradeWithoutChecks => CallDomain::Nobody,
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum BleavitCall {
    System(SystemCall),
    Domain(CallDomain),
    Wrapped(RuntimeCall),
}

impl BleavitCall {
    pub fn as_filter_call(&self) -> RuntimeCall {
        match self {
            Self::System(call) => RuntimeCall::leaf(call.domain()),
            Self::Domain(domain) => RuntimeCall::leaf(*domain),
            Self::Wrapped(call) => call.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct PendingUpgrade {
    pub code_hash: H256,
    pub authorized_at: BlockNumber,
    pub applicable_at: BlockNumber,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum RuntimeFilterError {
    SafetyFilter,
    PendingUpgradeMissing,
    PendingUpgradeTooEarly,
}

pub struct BaseCallFilter;

impl BaseCallFilter {
    pub fn contains(call: &BleavitCall) -> bool {
        SafetyFilter::contains(&call.as_filter_call())
    }

    pub fn contains_for(origin: Origin, call: &BleavitCall) -> bool {
        SafetyFilter::contains_for(origin, &call.as_filter_call())
    }

    pub fn validate_at(
        origin: Option<Origin>,
        call: &BleavitCall,
        now: BlockNumber,
        pending_upgrade: Option<PendingUpgrade>,
    ) -> Result<(), RuntimeFilterError> {
        let filter_call = call.as_filter_call();
        match origin {
            Some(origin) if !SafetyFilter::contains_for(origin, &filter_call) => {
                return Err(RuntimeFilterError::SafetyFilter);
            }
            None if !SafetyFilter::contains(&filter_call) => {
                return Err(RuntimeFilterError::SafetyFilter);
            }
            _ => {}
        }

        if matches!(
            call,
            BleavitCall::System(SystemCall::ApplyAuthorizedUpgrade)
        ) {
            let pending = pending_upgrade.ok_or(RuntimeFilterError::PendingUpgradeMissing)?;
            if now < pending.applicable_at {
                return Err(RuntimeFilterError::PendingUpgradeTooEarly);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum FeeAsset {
    Vit,
    Usdc,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct FeeConfig {
    pub native_asset: FeeAsset,
    pub foreign_fee_asset: u32,
    pub conversion_rate_key: ParamKey,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            native_asset: FeeAsset::Vit,
            foreign_fee_asset: USDC_ASSET_ID,
            conversion_rate_key: FEE_VIT_USDC_RATE_KEY,
        }
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct ForeignAssetConfig {
    pub asset_id: u32,
    pub location: [u8; 32],
    pub decimals: u8,
    pub is_sufficient: bool,
    pub admin_origin: Origin,
    pub mint_burn_enabled: bool,
}

impl Default for ForeignAssetConfig {
    fn default() -> Self {
        Self {
            asset_id: USDC_ASSET_ID,
            location: USDC_LOCATION,
            decimals: USDC_DECIMALS,
            is_sufficient: true,
            admin_origin: Origin::ConstitutionalValues,
            mint_burn_enabled: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum RuntimeFilter {
    SafetyFilter,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct RuntimeConfig {
    pub millisecs_per_block: u64,
    pub ss58_prefix: u16,
    pub base_call_filter: RuntimeFilter,
    pub usdc: ForeignAssetConfig,
    pub fees: FeeConfig,
    pub sudo_phase_enabled: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            millisecs_per_block: MILLISECS_PER_BLOCK,
            ss58_prefix: SS58_PREFIX,
            base_call_filter: RuntimeFilter::SafetyFilter,
            usdc: ForeignAssetConfig::default(),
            fees: FeeConfig::default(),
            sudo_phase_enabled: true,
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct GenesisConfig {
    pub endowed_vit: Vec<([u8; 32], Balance)>,
    pub usdc_asset: ForeignAssetConfig,
    pub sudo: Option<[u8; 32]>,
    pub bootstrap_calls: Vec<BleavitCall>,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum GenesisError {
    FilteredBootstrapCall,
    UsdcMisconfigured,
}

impl GenesisConfig {
    pub fn validate(&self) -> Result<(), GenesisError> {
        let expected = ForeignAssetConfig::default();
        if self.usdc_asset != expected {
            return Err(GenesisError::UsdcMisconfigured);
        }
        for call in &self.bootstrap_calls {
            if !BaseCallFilter::contains(call) {
                return Err(GenesisError::FilteredBootstrapCall);
            }
        }
        Ok(())
    }
}

pub fn origin_for_proposal_class(class: futarchy_primitives::ProposalClass) -> Option<Origin> {
    Origin::from_proposal_class(class)
}

pub fn all_custom_origins() -> [Origin; 8] {
    [
        Origin::FutarchyParam,
        Origin::FutarchyTreasury,
        Origin::FutarchyCode,
        Origin::FutarchyMeta,
        Origin::ConstitutionalValues,
        Origin::OracleResolution,
        Origin::GuardianHold,
        Origin::EmergencyPlaybook,
    ]
}

pub fn exhaustive_filter_sample() -> Vec<BleavitCall> {
    use pallet_origins::{BoxedCall, RuntimeCall as C};
    fn boxed(call: C) -> BoxedCall {
        BoxedCall::new(call)
    }
    vec![
        BleavitCall::System(SystemCall::Remark),
        BleavitCall::System(SystemCall::SetHeapPages),
        BleavitCall::System(SystemCall::SetCode),
        BleavitCall::System(SystemCall::SetCodeWithoutChecks),
        BleavitCall::System(SystemCall::SetStorage),
        BleavitCall::System(SystemCall::KillStorage),
        BleavitCall::System(SystemCall::KillPrefix),
        BleavitCall::System(SystemCall::AuthorizeUpgrade),
        BleavitCall::System(SystemCall::AuthorizeUpgradeWithoutChecks),
        BleavitCall::System(SystemCall::ApplyAuthorizedUpgrade),
        BleavitCall::Domain(CallDomain::Public),
        BleavitCall::Domain(CallDomain::Nobody),
        BleavitCall::Domain(CallDomain::Param),
        BleavitCall::Domain(CallDomain::Treasury),
        BleavitCall::Domain(CallDomain::Code),
        BleavitCall::Domain(CallDomain::Meta),
        BleavitCall::Domain(CallDomain::ConstitutionalValues),
        BleavitCall::Domain(CallDomain::OracleResolution),
        BleavitCall::Domain(CallDomain::GuardianHold),
        BleavitCall::Domain(CallDomain::EmergencyPlaybook),
        BleavitCall::Domain(CallDomain::InternalRoot),
        BleavitCall::Wrapped(C::UtilityBatch(vec![C::leaf(CallDomain::Public)])),
        BleavitCall::Wrapped(C::UtilityBatchAll(vec![C::leaf(CallDomain::Public)])),
        BleavitCall::Wrapped(C::UtilityForceBatch(vec![C::leaf(CallDomain::Public)])),
        BleavitCall::Wrapped(C::UtilityDispatchAs(boxed(C::leaf(CallDomain::Public)))),
        BleavitCall::Wrapped(C::UtilityAsDerivative(boxed(C::leaf(CallDomain::Public)))),
        BleavitCall::Wrapped(C::UtilityWithWeight(boxed(C::leaf(CallDomain::Public)))),
        BleavitCall::Wrapped(C::Proxy(boxed(C::leaf(CallDomain::Public)))),
        BleavitCall::Wrapped(C::ProxyAnnounced(boxed(C::leaf(CallDomain::Public)))),
        BleavitCall::Wrapped(C::MultisigAsMulti(boxed(C::leaf(CallDomain::Public)))),
        BleavitCall::Wrapped(C::MultisigAsMultiThreshold1(boxed(C::leaf(
            CallDomain::Public,
        )))),
        BleavitCall::Wrapped(C::MultisigApproveAsMulti),
        BleavitCall::Wrapped(C::Scheduler {
            origin: Origin::ConstitutionalValues,
            call: boxed(C::leaf(CallDomain::ConstitutionalValues)),
        }),
        BleavitCall::Wrapped(C::Sudo(boxed(C::leaf(CallDomain::Public)))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use futarchy_primitives::ProposalClass;
    use pallet_origins::{BoxedCall, RuntimeCall as C};

    fn boxed(call: C) -> BoxedCall {
        BoxedCall::new(call)
    }

    #[test]
    fn runtime_includes_standard_and_custom_pallet_sets() {
        assert_eq!(STANDARD_PALLETS.len(), 24);
        assert_eq!(CUSTOM_PALLETS.len(), 12);
        assert!(STANDARD_PALLETS.contains(&RuntimePallet::ForeignAssets));
        assert!(STANDARD_PALLETS.contains(&RuntimePallet::AssetTxPayment));
        assert!(CUSTOM_PALLETS.contains(&RuntimePallet::ExecutionGuard));
        assert_eq!(
            RuntimeConfig::default().base_call_filter,
            RuntimeFilter::SafetyFilter
        );
    }

    #[test]
    fn usdc_and_fee_configuration_match_contract_surface() {
        let cfg = RuntimeConfig::default();
        assert_eq!(cfg.usdc.asset_id, USDC_ASSET_ID);
        assert_eq!(cfg.usdc.location, USDC_LOCATION);
        assert!(cfg.usdc.is_sufficient);
        assert_eq!(cfg.usdc.admin_origin, Origin::ConstitutionalValues);
        assert!(!cfg.usdc.mint_burn_enabled);
        assert_eq!(cfg.fees.native_asset, FeeAsset::Vit);
        assert_eq!(cfg.fees.foreign_fee_asset, USDC_ASSET_ID);
        assert_eq!(cfg.fees.conversion_rate_key, FEE_VIT_USDC_RATE_KEY);
    }

    #[test]
    fn genesis_filter_denies_d13_system_calls_even_for_sudo_bootstrap() {
        for call in [
            SystemCall::SetHeapPages,
            SystemCall::SetCode,
            SystemCall::SetCodeWithoutChecks,
            SystemCall::SetStorage,
            SystemCall::KillStorage,
            SystemCall::KillPrefix,
            SystemCall::AuthorizeUpgradeWithoutChecks,
        ] {
            let genesis = GenesisConfig {
                endowed_vit: Vec::new(),
                usdc_asset: ForeignAssetConfig::default(),
                sudo: Some([7; 32]),
                bootstrap_calls: vec![BleavitCall::System(call)],
            };
            assert_eq!(genesis.validate(), Err(GenesisError::FilteredBootstrapCall));
            assert!(!BaseCallFilter::contains(&BleavitCall::System(call)));
        }
        assert!(GenesisConfig {
            endowed_vit: Vec::new(),
            usdc_asset: ForeignAssetConfig::default(),
            sudo: Some([7; 32]),
            bootstrap_calls: vec![BleavitCall::System(SystemCall::Remark)]
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn authorize_upgrade_is_internal_but_apply_is_permissionless_after_lead_time() {
        assert!(!BaseCallFilter::contains(&BleavitCall::System(
            SystemCall::AuthorizeUpgrade
        )));
        assert!(!BaseCallFilter::contains_for(
            Origin::FutarchyCode,
            &BleavitCall::System(SystemCall::AuthorizeUpgrade)
        ));

        let apply = BleavitCall::System(SystemCall::ApplyAuthorizedUpgrade);
        let pending = PendingUpgrade {
            code_hash: [9; 32],
            authorized_at: 10,
            applicable_at: 20,
        };
        assert!(BaseCallFilter::contains(&apply));
        assert_eq!(
            BaseCallFilter::validate_at(None, &apply, 19, Some(pending)),
            Err(RuntimeFilterError::PendingUpgradeTooEarly)
        );
        assert_eq!(
            BaseCallFilter::validate_at(None, &apply, 20, Some(pending)),
            Ok(())
        );
        assert_eq!(
            BaseCallFilter::validate_at(None, &apply, 20, None),
            Err(RuntimeFilterError::PendingUpgradeMissing)
        );
    }

    #[test]
    fn custom_origin_mapping_is_wired_to_proposal_classes() {
        assert_eq!(
            origin_for_proposal_class(ProposalClass::Param),
            Some(Origin::FutarchyParam)
        );
        assert_eq!(
            origin_for_proposal_class(ProposalClass::Treasury),
            Some(Origin::FutarchyTreasury)
        );
        assert_eq!(
            origin_for_proposal_class(ProposalClass::Code),
            Some(Origin::FutarchyCode)
        );
        assert_eq!(
            origin_for_proposal_class(ProposalClass::Meta),
            Some(Origin::FutarchyMeta)
        );
        assert_eq!(
            origin_for_proposal_class(ProposalClass::Constitutional),
            None
        );
        assert_eq!(all_custom_origins().len(), 8);
    }

    #[test]
    fn base_call_filter_delegates_to_safety_filter_for_domains_and_wrappers() {
        assert!(BaseCallFilter::contains(&BleavitCall::Domain(
            CallDomain::Public
        )));
        assert!(!BaseCallFilter::contains(&BleavitCall::Domain(
            CallDomain::Nobody
        )));
        assert!(BaseCallFilter::contains_for(
            Origin::FutarchyTreasury,
            &BleavitCall::Domain(CallDomain::Treasury)
        ));
        assert!(!BaseCallFilter::contains_for(
            Origin::FutarchyParam,
            &BleavitCall::Domain(CallDomain::Treasury)
        ));
        let hidden =
            BleavitCall::Wrapped(C::ProxyAnnounced(boxed(C::UtilityBatch(vec![C::leaf(
                CallDomain::Code,
            )]))));
        assert!(!BaseCallFilter::contains_for(Origin::FutarchyCode, &hidden));
        let threshold_one = BleavitCall::Wrapped(C::MultisigAsMultiThreshold1(boxed(C::leaf(
            CallDomain::Meta,
        ))));
        assert!(!BaseCallFilter::contains_for(
            Origin::FutarchyMeta,
            &threshold_one
        ));
    }

    #[test]
    fn filter_exhaustiveness_sample_covers_every_runtime_shape() {
        let sample = exhaustive_filter_sample();
        assert_eq!(sample.len(), 34);
        for call in &sample {
            let _ = BaseCallFilter::contains(call);
        }
        assert!(sample
            .iter()
            .any(|c| matches!(c, BleavitCall::Wrapped(C::ProxyAnnounced(_)))));
        assert!(sample
            .iter()
            .any(|c| matches!(c, BleavitCall::Wrapped(C::MultisigAsMultiThreshold1(_)))));
        assert!(sample
            .iter()
            .any(|c| matches!(c, BleavitCall::Wrapped(C::Scheduler { .. }))));
    }
}

use bleavit_runtime_api::FutarchyApi;
use futarchy_primitives::{
    AccountId, BoundedVec, CohortSummaryView, DecisionStatsView, EpochStatusView, FixedU64,
    NavView as ContractNavView, OracleRoundView, ParamView, PositionId, PositionView, ProposalId,
    ProposalSummaryView, QueuedExecutionView, QuoteView, RuntimeVersionConstraint, TradeSide,
    WelfareView,
};

type Account = AccountId;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct RuntimeState {
    pub epoch: pallet_epoch::EpochState<Account>,
    pub ledger: pallet_conditional_ledger::LedgerState<Account>,
    pub market: pallet_market::MarketState<Account>,
    pub constitution: pallet_constitution::ConstitutionState,
    pub welfare: pallet_welfare::WelfareState,
    pub oracle: pallet_oracle::Oracle,
    pub execution_guard: pallet_execution_guard::ExecutionGuard,
    pub treasury: pallet_futarchy_treasury::Treasury,
    pub decision_stats_cache: Vec<DecisionStatsView>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            epoch: pallet_epoch::EpochState::new(),
            ledger: pallet_conditional_ledger::LedgerState::new(),
            market: pallet_market::MarketState::new(),
            constitution: pallet_constitution::ConstitutionState::genesis(),
            welfare: pallet_welfare::WelfareState::new(),
            oracle: pallet_oracle::Oracle::default(),
            execution_guard: pallet_execution_guard::ExecutionGuard::new(
                RuntimeVersionConstraint {
                    spec_name: BoundedVec::default(),
                    spec_version: RUNTIME_SPEC_VERSION,
                },
            ),
            treasury: pallet_futarchy_treasury::Treasury::default(),
            decision_stats_cache: Vec::new(),
        }
    }

    fn vault_state_for(&self, position: PositionId) -> futarchy_primitives::VaultState {
        match position {
            PositionId::Proposal { proposal, .. } => self
                .ledger
                .vaults
                .iter()
                .find(|v| v.proposal == proposal)
                .map(|v| v.info.state)
                .unwrap_or(futarchy_primitives::VaultState::Open),
            PositionId::Baseline { epoch, .. } => self
                .ledger
                .baseline_vaults
                .iter()
                .find(|v| v.epoch == epoch)
                .map(|v| match v.info.state {
                    pallet_conditional_ledger::BaselineState::Open => {
                        futarchy_primitives::VaultState::Open
                    }
                    pallet_conditional_ledger::BaselineState::Settled(s) => {
                        futarchy_primitives::VaultState::ScalarSettled {
                            winner: futarchy_primitives::Branch::Accept,
                            s,
                        }
                    }
                })
                .unwrap_or(futarchy_primitives::VaultState::Open),
        }
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl FutarchyApi for RuntimeState {
    fn epoch_status(&self) -> EpochStatusView {
        self.epoch.status_view()
    }

    fn proposal_summaries(&self) -> BoundedVec<ProposalSummaryView, 32> {
        self.epoch
            .proposals
            .iter()
            .map(|p| ProposalSummaryView {
                id: p.id,
                class: p.class,
                state: p.state,
                proposer: p.proposer,
                epoch: p.epoch,
                payload_hash: p.payload_hash,
                ask: p.ask,
                decision_market: p.markets.map(|m| (m.accept, m.reject)),
                gate_markets: p.markets.and_then(|m| m.gates),
                decide_at: p.decide_at,
                maturity: p.maturity,
                ratification: futarchy_primitives::RatificationStatus::NotRequired,
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_default()
    }

    fn quote(
        &self,
        market: futarchy_primitives::MarketId,
        side: TradeSide,
        amount: futarchy_primitives::Balance,
    ) -> QuoteView {
        self.market
            .quote_view(market, side, amount)
            .unwrap_or(QuoteView {
                cost: 0,
                fee: 0,
                p_after_1e9: FixedU64(0),
                max_trade: 0,
                within_domain: false,
            })
    }

    fn decision_stats(&self, pid: ProposalId) -> Option<DecisionStatsView> {
        self.decision_stats_cache
            .iter()
            .find(|s| s.pid == pid)
            .cloned()
    }

    fn account_positions(&self, who: AccountId) -> BoundedVec<PositionView, 64> {
        self.ledger
            .positions
            .iter()
            .filter(|p| p.owner == who)
            .map(|p| PositionView {
                position: p.id,
                balance: p.balance,
                vault_state: self.vault_state_for(p.id),
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_default()
    }

    fn execution_queue(&self) -> BoundedVec<QueuedExecutionView, 32> {
        self.execution_guard.view().try_into().unwrap_or_default()
    }

    fn welfare_current(&self) -> WelfareView {
        self.welfare
            .current_view(
                self.epoch.epoch.index,
                0,
                self.oracle.reserve_health.unhealthy,
            )
            .unwrap_or(WelfareView {
                epoch: self.epoch.epoch.index,
                spec_version: 0,
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
                reserve_flag: self.oracle.reserve_health.unhealthy,
            })
    }

    fn params(&self, keys: BoundedVec<ParamKey, 64>) -> BoundedVec<ParamView, 64> {
        keys.as_slice()
            .iter()
            .filter_map(|key| {
                self.constitution
                    .params
                    .iter()
                    .find(|p| &p.key == key)
                    .map(param_view)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_default()
    }

    fn nav(&self) -> ContractNavView {
        let nav = self.treasury.nav();
        ContractNavView {
            total: nav.nav,
            main: self.treasury.main_usdc,
            pol: 0,
            insurance: 0,
            keeper: 0,
            oracle: 0,
            rewards: 0,
            stream_remainders: 0,
            obligations: nav.nav.saturating_sub(nav.spendable_nav),
            haircut_flag: nav.reserve_impaired,
        }
    }

    fn recent_cohorts(&self) -> BoundedVec<CohortSummaryView, 32> {
        self.epoch.recent.clone().try_into().unwrap_or_default()
    }

    fn open_oracle_rounds(&self) -> BoundedVec<OracleRoundView, 192> {
        self.oracle
            .rounds
            .iter()
            .map(|r| OracleRoundView {
                component: r.component,
                epoch: r.epoch,
                round: r.round,
                reporter: r.reporter,
                value_1e9: r.value,
                evidence_hash: r.evidence_hash,
                bond: r.bond,
                challenge_deadline: r.challenge_deadline,
                acked_by_watchtowers: r.acks,
                escalated: r.round > 1,
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_default()
    }
}

fn param_view(p: &pallet_constitution::ParamRecord) -> ParamView {
    fn val(v: pallet_constitution::ParamValue) -> u128 {
        v.as_u128()
    }
    ParamView {
        key: p.key,
        value: val(p.value),
        min: val(p.min),
        max: val(p.max),
        max_delta: p
            .max_delta
            .map(|d| match d {
                pallet_constitution::MaxDelta::Absolute(v) => val(v),
                pallet_constitution::MaxDelta::Percent(x) => x as u128,
                pallet_constitution::MaxDelta::Factor(x) => x as u128,
            })
            .unwrap_or(0),
        cooldown_blocks: p.cooldown_epochs,
        last_change: p.last_changed_epoch,
        class: param_class(p.class),
    }
}

fn param_class(value: pallet_constitution::ParamClass) -> futarchy_primitives::ProposalClass {
    match value {
        pallet_constitution::ParamClass::Param => futarchy_primitives::ProposalClass::Param,
        pallet_constitution::ParamClass::Treasury => futarchy_primitives::ProposalClass::Treasury,
        pallet_constitution::ParamClass::Meta => futarchy_primitives::ProposalClass::Meta,
        pallet_constitution::ParamClass::Const
        | pallet_constitution::ParamClass::Entrenched
        | pallet_constitution::ParamClass::MetaAndValues => {
            futarchy_primitives::ProposalClass::Constitutional
        }
    }
}

#[cfg(test)]
mod b2_tests {
    use super::*;
    use bleavit_runtime_api::{FutarchyApi, FUTARCHY_API_METHODS};
    use futarchy_primitives::{bounds, ProposalClass, ProposalState};

    #[test]
    fn futarchy_api_declares_all_eleven_methods() {
        assert_eq!(FUTARCHY_API_METHODS.len(), 11);
        assert_eq!(FUTARCHY_API_METHODS[0], "epoch_status");
        assert_eq!(FUTARCHY_API_METHODS[10], "open_oracle_rounds");
    }

    #[test]
    fn runtime_api_returns_bounded_contract_views() {
        let mut rt = RuntimeState::new();
        rt.epoch.proposals.push(pallet_epoch::Proposal {
            id: 7,
            proposer: [1; 32],
            class: ProposalClass::Param,
            state: ProposalState::Queued,
            epoch: 2,
            submitted_at: 1,
            payload_hash: [9; 32],
            ask: 0,
            bond: 0,
            resources: Vec::new(),
            metric_spec: 0,
            decide_at: 10,
            rerun: false,
            extended: false,
            delayed_once: false,
            markets: Some(pallet_epoch::MarketSet {
                accept: 1,
                reject: 2,
                gates: Some([3, 4, 5, 6]),
                baseline: 9,
            }),
            maturity: Some(20),
            grace_end: Some(30),
            version_constraint: None,
            decision: None,
        });
        rt.decision_stats_cache.push(DecisionStatsView {
            pid: 7,
            twap_accept_1e9: FixedU64(1),
            twap_reject_1e9: FixedU64(2),
            twap_baseline_1e9: FixedU64(3),
            r_eff_1e9: FixedU64(4),
            trailing_accept_1e9: FixedU64(5),
            trailing_reject_1e9: FixedU64(6),
            coverage_pct: 99,
            traded_volume: 10,
            v_min_required: 11,
            converged: true,
            gate_twaps_1e9: Some([FixedU64(1); 4]),
            attack_cost_hat: 12,
            in_cap_prize: 4,
        });

        let summaries = rt.proposal_summaries();
        assert_eq!(summaries.as_slice().len(), 1);
        assert_eq!(summaries.as_slice()[0].decision_market, Some((1, 2)));
        assert_eq!(rt.decision_stats(7).unwrap().coverage_pct, 99);
        assert_eq!(rt.execution_queue().as_slice().len(), 0);
        assert_eq!(rt.open_oracle_rounds().as_slice().len(), 0);
        assert_eq!(
            BoundedVec::<ProposalSummaryView, { bounds::MAX_PROPOSAL_SUMMARIES }>::BOUND,
            32
        );
    }

    #[test]
    fn account_positions_are_filtered_and_annotated() {
        let who = [4; 32];
        let mut rt = RuntimeState::new();
        rt.ledger.create_vault(42, 0).unwrap();
        rt.ledger
            .positions
            .push(pallet_conditional_ledger::PositionRecord {
                id: futarchy_primitives::PositionId::Proposal {
                    proposal: 42,
                    branch: futarchy_primitives::Branch::Accept,
                    kind: futarchy_primitives::PositionKind::BranchUsdc,
                },
                owner: who,
                balance: 100,
                deposit: 0,
            });
        assert_eq!(rt.account_positions([9; 32]).as_slice().len(), 0);
        let positions = rt.account_positions(who);
        assert_eq!(positions.as_slice().len(), 1);
        assert_eq!(
            positions.as_slice()[0].vault_state,
            futarchy_primitives::VaultState::Open
        );
    }
}
