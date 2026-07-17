use crate::{mock::*, CumulativeDeposits};
use frame_support::assert_ok;

#[test]
fn mint_admits_exact_cap_and_refuses_cap_plus_one() {
    new_test_ext().execute_with(|| {
        UsdcIssuance::set(900);
        assert_ok!(InflowCaps::mint_admissible(100));
        assert_eq!(InflowCaps::mint_admissible(101), Err(()));
        assert_eq!(UsdcIssuance::get(), 900);
        assert_eq!(CumulativeDeposits::<Test>::iter().count(), 0);
    });
}

#[test]
fn zero_mint_is_always_admissible_and_sentinel_is_unbounded() {
    new_test_ext().execute_with(|| {
        TvlCap::set(0);
        UsdcIssuance::set(u128::MAX);
        assert_ok!(InflowCaps::mint_admissible(0));

        TvlCap::set(u128::MAX);
        assert_ok!(InflowCaps::mint_admissible(u128::MAX));
    });
}

#[test]
fn inflows_accumulate_to_the_exact_cap() {
    new_test_ext().execute_with(|| {
        DepositCap::set(10);
        assert_ok!(InflowCaps::note_inflow(&1, 4));
        assert_ok!(InflowCaps::note_inflow(&1, 6));
        assert_eq!(CumulativeDeposits::<Test>::get(1), 10);
        assert_ok!(InflowCaps::do_try_state());
    });
}

#[test]
fn refused_inflow_writes_nothing() {
    new_test_ext().execute_with(|| {
        DepositCap::set(10);
        assert_ok!(InflowCaps::note_inflow(&1, 4));
        assert_eq!(InflowCaps::note_inflow(&1, 7), Err(()));
        assert_eq!(CumulativeDeposits::<Test>::get(1), 4);

        DepositCap::set(u128::MAX);
        CumulativeDeposits::<Test>::insert(1, u128::MAX);
        assert_eq!(InflowCaps::note_inflow(&1, 1), Err(()));
        assert_eq!(CumulativeDeposits::<Test>::get(1), u128::MAX);
    });
}

#[test]
fn meters_are_isolated_per_account() {
    new_test_ext().execute_with(|| {
        DepositCap::set(10);
        assert_ok!(InflowCaps::note_inflow(&1, 10));
        assert_ok!(InflowCaps::note_inflow(&2, 7));
        assert_eq!(CumulativeDeposits::<Test>::get(1), 10);
        assert_eq!(CumulativeDeposits::<Test>::get(2), 7);
    });
}

#[test]
fn sentinel_makes_per_account_cap_unbounded_and_zero_is_not_stored() {
    new_test_ext().execute_with(|| {
        DepositCap::set(u128::MAX);
        assert_ok!(InflowCaps::note_inflow(&1, u128::MAX));
        assert_ok!(InflowCaps::note_inflow(&2, 0));
        assert_eq!(CumulativeDeposits::<Test>::get(1), u128::MAX);
        assert!(!CumulativeDeposits::<Test>::contains_key(2));
        assert_ok!(InflowCaps::do_try_state());
    });
}

#[test]
fn try_state_rejects_over_cap_and_zero_entries() {
    new_test_ext().execute_with(|| {
        DepositCap::set(10);
        CumulativeDeposits::<Test>::insert(1, 11);
        assert!(InflowCaps::do_try_state().is_err());

        CumulativeDeposits::<Test>::remove(1);
        CumulativeDeposits::<Test>::insert(2, 0);
        assert!(InflowCaps::do_try_state().is_err());
    });
}
