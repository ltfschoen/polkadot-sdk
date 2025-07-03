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

//! Integration tests for origin-and-gate pallet focusing on verifying end-to-end
//! workflows and interactions between components rather than isolated functions,
//! testing the pallet's public API from an external perspective of real-world usage
//! patterns, and with complex workflows and edge cases handled in dedicated integration
//! test files.

use assert_matches::assert_matches;
use frame_support::assert_ok;
use frame_support::traits::EnsureOrigin;
use sp_core::H256;
use sp_runtime::DispatchError;

mod common;
use common::*;

#[test]
fn test_direct_and_gate_impossible_with_signed_origins() {
	new_test_ext().execute_with(|| {
		// Test that signed origins cannot satisfy AndGate directly
		// to represents the real-world scenario where a single account
		// cannot simultaneously satisfy multiple origin requirements
		assert!(AliceAndBob::try_origin(RuntimeOrigin::signed(ALICE)).is_err());
		assert!(AliceAndBob::try_origin(RuntimeOrigin::signed(BOB)).is_err());
	});
}

#[test]
fn test_direct_and_gate_impossible_with_root_origin() {
	new_test_ext().execute_with(|| {
		// Test that even root origin cannot bypass AndGate requirements
		assert!(AliceAndBob::try_origin(RuntimeOrigin::root()).is_err());
	});
}

#[test]
fn test_origin_type_with_proposal_workflow() {
	new_test_ext().execute_with(|| {
		// Set block number for event verification
		System::set_block_number(1);

		// Create a test call that will be used in the proposal
		let call = Call::Dummy { data: 42 }.into();
		let call_hash = <Test as Config>::Hashing::hash_of(&call);

		// Propose using Alice's origin and get origin ID dynamically
		let alice_origin_id = match AliceOrigin::origin_type() {
			CustomOriginType::Alice => ALICE_ORIGIN_ID,
			_ => panic!("Unexpected origin type"),
		};

		assert_ok!(OriginAndGate::propose(
			RuntimeOrigin::signed(ALICE),
			call.clone(),
			alice_origin_id,
			None
		));

		// Bob approves the proposal with dynamically determined origin ID
		let bob_origin_id = match BobOrigin::origin_type() {
			CustomOriginType::Bob => BOB_ORIGIN_ID,
			_ => panic!("Unexpected origin type"),
		};

		assert_ok!(OriginAndGate::approve(
			RuntimeOrigin::signed(BOB),
			call_hash,
			alice_origin_id,
			bob_origin_id,
		));

		// Verify the proposal exists and has both approvals
		let proposal = Proposals::<Test>::get(call_hash, alice_origin_id).unwrap();
		assert_eq!(proposal.approvals.len(), 2);

		// Verify both origin IDs are in the approvals
		assert!(proposal.approvals.contains(&alice_origin_id));
		assert!(proposal.approvals.contains(&bob_origin_id));

		// Verify approval event was emitted with correct origin IDs
		System::assert_has_event(<Test as Config>::Event::ProposalApproved(
			call_hash,
			alice_origin_id,
			bob_origin_id,
		));
	});
}

#[test]
fn test_andgate_origin_type() {
	new_test_ext().execute_with(|| {
		// Composite AndGate should return the default variant
		assert_eq!(AliceAndBob::origin_type(), CustomOriginType::None);

		// Individual origins should return their respective variants
		assert_eq!(AliceOrigin::origin_type(), CustomOriginType::Alice);
		assert_eq!(BobOrigin::origin_type(), CustomOriginType::Bob);
	});
}
