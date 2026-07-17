//! Runtime-call classification for the XCM rows of B1a's SafetyFilter (09 §6.1–§6.2).

use alloc::vec::Vec;
use frame_support::traits::Contains;
use staging_xcm::latest::{Asset, Junction, Location};

/// Origin disposition consumed by the runtime's outer SafetyFilter (09 §6.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XcmCallDisposition {
    DeniedAllOrigins,
    SignedAllowed,
    TreasuryOnly,
}

/// Canonical outbound user-exit asset/origin filter (09 §6.2). Destination is
/// independently pinned to Asset Hub by `classify_pallet_xcm_call`; this layer
/// admits only a signed local AccountId32 and the frozen DOT/USDC assets.
pub struct ReserveTransferFilter;

impl Contains<(Location, Vec<Asset>)> for ReserveTransferFilter {
    fn contains((origin, assets): &(Location, Vec<Asset>)) -> bool {
        let signed_origin = matches!(origin.unpack(), (0, [Junction::AccountId32 { .. }]));
        signed_origin
            && !assets.is_empty()
            && assets.iter().all(|asset| {
                asset.id.0 == crate::identity::usdc_location()
                    || asset.id.0 == crate::identity::dot_location()
            })
    }
}

/// Classifies every stable2606 `pallet_xcm::Call` variant (09 §6.1–§6.2).
///
/// The deprecated unlimited reserve-transfer call is denied: the limited form is the canonical
/// exit and forces the user to state a remote weight bound (09 §6.2). All arbitrary-message,
/// teleport, aliasing, generic-transfer, force and execution escape hatches are denied.
#[allow(deprecated)]
pub fn classify_pallet_xcm_call<T: pallet_xcm::Config>(
    call: &pallet_xcm::Call<T>,
) -> XcmCallDisposition {
    use XcmCallDisposition::{DeniedAllOrigins, SignedAllowed};

    match call {
        pallet_xcm::Call::limited_reserve_transfer_assets { dest, .. } => {
            // 01 §4 and 09 §6.2 expose one canonical user exit only: USDC
            // reserve-transfer to Asset Hub. Refuse stale/unconvertible versions
            // and every other sibling before the call reaches pallet-xcm.
            if staging_xcm::latest::Location::try_from((**dest).clone())
                .is_ok_and(|destination| destination == crate::identity::asset_hub_location())
            {
                SignedAllowed
            } else {
                DeniedAllOrigins
            }
        }
        // `claim_assets` is self-scoped by pallet-xcm: the dispatch origin is
        // converted to a Location and must match the trap key. Signed callers
        // therefore gain no authority over anyone else's trap; B1a maps the
        // TREASURY class into the same call for protocol-keyed traps (09 §6.1).
        pallet_xcm::Call::claim_assets { .. } => SignedAllowed,
        pallet_xcm::Call::send { .. }
        | pallet_xcm::Call::teleport_assets { .. }
        | pallet_xcm::Call::reserve_transfer_assets { .. }
        | pallet_xcm::Call::execute { .. }
        | pallet_xcm::Call::force_xcm_version { .. }
        | pallet_xcm::Call::force_default_xcm_version { .. }
        | pallet_xcm::Call::force_subscribe_version_notify { .. }
        | pallet_xcm::Call::force_unsubscribe_version_notify { .. }
        | pallet_xcm::Call::limited_teleport_assets { .. }
        | pallet_xcm::Call::force_suspension { .. }
        | pallet_xcm::Call::transfer_assets { .. }
        | pallet_xcm::Call::transfer_assets_using_type_and_then { .. }
        | pallet_xcm::Call::add_authorized_alias { .. }
        | pallet_xcm::Call::remove_authorized_alias { .. }
        | pallet_xcm::Call::remove_all_authorized_aliases { .. } => DeniedAllOrigins,
        // FRAME adds this unconstructible variant. Naming it keeps the match exhaustive, so a
        // future SDK call variant fails compilation instead of inheriting a permissive default.
        pallet_xcm::Call::__Ignore(..) => DeniedAllOrigins,
    }
}
