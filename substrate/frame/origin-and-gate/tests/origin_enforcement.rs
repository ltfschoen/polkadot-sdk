// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Integration tests for origin-and-gate pallet focusing on origin enforcement mechanisms.

use assert_matches::assert_matches;
use frame_support::assert_ok;
use frame_support::traits::EnsureOrigin;
use pallet_origin_and_gate::{Config, ProposalStatus, Proposals};

// Import common test module
mod common;
use common::*;

#[test]
fn ensure_origin_works_with_and_gate() {
	new_test_ext().execute_with(|| {
		// Proceed past genesis block so events get deposited
		System::set_block_number(1);

		// Generate call hash
		let call = make_remark_call("1000").unwrap();
		let call_hash = <<Test as Config>::Hashing as sp_runtime::traits::Hash>::hash_of(&call);

		// Proposal by Alice dispatching a signed extrinsic
		assert_ok!(OriginAndGate::propose(
			RuntimeOrigin::signed(ALICE),
			call.clone(),
			ALICE_ORIGIN_ID,
			None,
		));

		// Prior to Bob's approval, test AliceAndBob origin directly and expect it to fail
		assert_matches!(AliceAndBob::ensure_origin(RuntimeOrigin::signed(ALICE)), Err(_));

		// Approval by Bob dispatching a signed extrinsic
		assert_ok!(OriginAndGate::approve(
			RuntimeOrigin::signed(BOB),
			call_hash,
			ALICE_ORIGIN_ID,
			BOB_ORIGIN_ID,
		));

		// Read pallet storage to verify the proposal is marked as executed as the
		// AliceAndBob origin should now pass for this call
		assert!(Proposals::<Test>::contains_key(call_hash, ALICE_ORIGIN_ID));
		let proposal = Proposals::<Test>::get(call_hash, ALICE_ORIGIN_ID).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Executed);

		// Verify ProposalExecuted event was emitted
		System::assert_has_event(RuntimeEvent::OriginAndGate(
			pallet_origin_and_gate::Event::ProposalExecuted {
				proposal_hash: call_hash,
				origin_id: ALICE_ORIGIN_ID,
				result: Ok(()),
			},
		));

		// assert!(System::events().iter().any(|r| {
		// 	matches!(
		// 		r.event,
		// 		RuntimeEvent::OriginAndGate(
		// 			pallet_origin_and_gate::Event::ProposalExecuted {
		// 				proposal_hash: ph,
		// 				origin_id: ALICE_ORIGIN_ID,
		// 				result: Ok(()),
		// 			}
		// 		) if ph == call_hash
		// 	)
		// }))
	});
}

#[test]
fn test_direct_and_gate_impossible_with_signed_origins() {
	new_test_ext().execute_with(|| {
		// Test that signed origins cannot satisfy AndGate directly
		// to represent real-world scenario where a single account
		// cannot simultaneously satisfy multiple origin requirements
		assert_matches!(AliceAndBob::ensure_origin(RuntimeOrigin::signed(ALICE)), Err(_));
		assert_matches!(AliceAndBob::ensure_origin(RuntimeOrigin::signed(BOB)), Err(_));
	});
}

#[test]
fn test_direct_and_gate_impossible_with_root_origin() {
	new_test_ext().execute_with(|| {
		// Test that even root origin cannot bypass AndGate requirements
		assert_matches!(AliceAndBob::ensure_origin(RuntimeOrigin::root()), Err(_));
	});
}

#[test]
fn ensure_different_origin_ids_must_be_used() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Generate call hash
		let call = make_remark_call("1000").unwrap();
		let call_hash = <<Test as Config>::Hashing as sp_runtime::traits::Hash>::hash_of(&call);

		// Proposal by Alice
		assert_ok!(OriginAndGate::propose(
			RuntimeOrigin::signed(ALICE),
			call.clone(),
			ALICE_ORIGIN_ID,
			None,
		));

		// Try to approve with the same origin ID which should fail
		let result = OriginAndGate::approve(
			RuntimeOrigin::signed(ALICE),
			call_hash,
			ALICE_ORIGIN_ID,
			ALICE_ORIGIN_ID,
		);
		assert!(result.is_err());

		// The same should apply for Bob
		let result = OriginAndGate::approve(
			RuntimeOrigin::signed(BOB),
			call_hash,
			ALICE_ORIGIN_ID,
			ALICE_ORIGIN_ID,
		);
		assert!(result.is_err());

		// Read pallet storage to verify the proposal is still pending
		let proposal = Proposals::<Test>::get(call_hash, ALICE_ORIGIN_ID).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Pending);
	});
}

#[test]
fn ensure_proposals_cannot_be_executed_by_other_means() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Generate call hash
		let call = make_remark_call("1000").unwrap();
		let call_hash = <<Test as Config>::Hashing as sp_runtime::traits::Hash>::hash_of(&call);

		// Proposal by Alice
		assert_ok!(OriginAndGate::propose(
			RuntimeOrigin::signed(ALICE),
			call.clone(),
			ALICE_ORIGIN_ID,
			None,
		));

		// Try to execute the call directly which should fail
		assert!(AliceAndBob::ensure_origin(RuntimeOrigin::signed(ALICE)).is_err());

		// Even with root origin, direct execution should fail
		assert!(AliceAndBob::ensure_origin(RuntimeOrigin::root()).is_err());

		// Read pallet storage to verify the proposal is still pending
		let proposal = Proposals::<Test>::get(call_hash, ALICE_ORIGIN_ID).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Pending);
	});
}
