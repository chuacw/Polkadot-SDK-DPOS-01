use std::collections::BTreeMap;

use frame_support::assert_ok;

use crate::{
	constants::{AccountId, Balance, TestAccount},
	mock::{self, *},
	pallet,
	tests::ros,
	types::{
		CandidateDelegationSet, CandidateDetail, DelegationInfo, EpochSnapshot, ValidatorStatus,
	},
	BalanceOf, CandidateDelegators, CandidatePool, DelegateCountMap, DelegationInfos, Event,
	HoldReason,
};
use frame_support::traits::fungible::InspectHold;

pub fn register_new_candidate(
	candidate: AccountId,
	balance: BalanceOf<Test>,
	hold_amount: BalanceOf<Test>,
) {
	assert_ok!(Dpos::register_as_candidate(ros(candidate), hold_amount));
	assert_eq!(
		CandidatePool::<Test>::get(candidate),
		Some(CandidateDetail {
			bond: hold_amount,
			total_delegations: 0,
			status: ValidatorStatus::Online
		})
	);
	assert_eq!(Balances::free_balance(candidate), balance - hold_amount);
	assert_eq!(Balances::total_balance_on_hold(&candidate), hold_amount);
	assert_eq!(
		Balances::balance_on_hold(&HoldReason::CandidateBondReserved.into(), &candidate),
		hold_amount
	);

	assert_eq!(CandidateDelegators::<Test>::get(&candidate), vec![]);

	// Assert that the correct event was deposited
	System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateRegistered {
		candidate_id: candidate,
		initial_bond: hold_amount,
	}));
}

pub fn get_genesis_epoch_snapshot(
	active_validator_set: CandidateDelegationSet<Test>,
) -> EpochSnapshot<Test> {
	EpochSnapshot {
		validators: BTreeMap::from_iter(
			active_validator_set.iter().map(|(v, b, _)| (*v, *b)).collect::<Vec<_>>(),
		),
		delegations: BTreeMap::default(),
	}
}

pub fn delegate_candidate(
	delegator: &TestAccount,
	candidate: &TestAccount,
	delegated_amount: Balance,
) {
	let delegator_balance = Balances::free_balance(delegator.id);
	let total_balance_on_hold = Balances::total_balance_on_hold(&delegator.id);
	let count = DelegateCountMap::<Test>::get(delegator.id);
	let delegators = CandidateDelegators::<Test>::get(candidate.id);
	let delegation_info = DelegationInfos::<Test>::get(delegator.id, candidate.id);
	let no_delegator = delegation_info.is_none();

	assert_ok!(Dpos::delegate_candidate(ros(delegator.id), candidate.id, delegated_amount));

	if no_delegator {
		assert_eq!(DelegateCountMap::<Test>::get(delegator.id), count + 1);
		assert_eq!(
			CandidateDelegators::<Test>::get(candidate.id),
			[delegators.to_vec(), vec![delegator.id]].concat()
		);
		assert_eq!(
			DelegationInfos::<Test>::get(delegator.id, candidate.id),
			Some(DelegationInfo { amount: delegated_amount })
		);
	} else {
		assert_eq!(
			DelegationInfos::<Test>::get(delegator.id, candidate.id),
			Some(DelegationInfo { amount: delegation_info.unwrap().amount + delegated_amount })
		);
	}
	assert_eq!(Balances::free_balance(delegator.id), delegator_balance - delegated_amount);
	assert_eq!(
		Balances::total_balance_on_hold(&delegator.id),
		total_balance_on_hold + delegated_amount
	);

	let candidate_detail = Dpos::get_candidate(&candidate.id).unwrap();
	System::assert_last_event(RuntimeEvent::Dpos(Event::CandidateDelegated {
		candidate_id: candidate.id,
		delegated_by: delegator.id,
		amount: delegated_amount,
		total_delegated_amount: candidate_detail.total_delegations,
	}));
}

pub fn get_author_commission() -> u32 {
	<mock::Test as pallet::Config>::AuthorCommission::get()
}

pub fn get_delegator_commission() -> u32 {
	<mock::Test as pallet::Config>::DelegatorCommission::get()
}
