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

use frame_support::assert_ok;
use frame_support::traits::EnsureOrigin;
use pallet_origin_and_gate::{Config, Event, Proposals};
use sp_runtime::traits::Hash;

mod common;
use common::*;

#[test]
fn test_create_and_retrieve_proposal() {
	new_test_ext().execute_with(|| {
		// Set block number for event verification
		System::set_block_number(1);

		// Create test call for use in proposal
		let call = Box::new(RuntimeCall::OriginAndGate(pallet_origin_and_gate::Call::set_dummy { new_value: 1000 }));
		let call_hash = <<Test as Config>::Hashing as Hash>::hash_of(&call);

		// Create proposal with Alice's origin
		assert_ok!(OriginAndGate::propose(
			RuntimeOrigin::signed(ALICE),
			call,
			ALICE_ORIGIN_ID,
			None,
		));

		// Verify proposal exists
		let proposal = Proposals::<Test>::get(call_hash, ALICE_ORIGIN_ID);
		assert!(proposal.is_some());

		// Verify event emitted
		System::assert_has_event(RuntimeEvent::OriginAndGate(Event::ProposalCreated {
			proposal_hash: call_hash,
			origin_id: ALICE_ORIGIN_ID,
		}));
	});
}

#[test]
fn test_proposal_workflow() {
	new_test_ext().execute_with(|| {
		// Set block number for event verification
		System::set_block_number(1);

		// Create test call to be used in proposal
		let call = Box::new(RuntimeCall::OriginAndGate(pallet_origin_and_gate::Call::set_dummy { new_value: 42 }));
		let call_hash = <<Test as Config>::Hashing as Hash>::hash_of(&call);

		// Propose using Alice's origin
		assert_ok!(OriginAndGate::propose(
			RuntimeOrigin::signed(ALICE),
			call.clone(),
			ALICE_ORIGIN_ID,
			None
		));

		// Bob approves proposal
		assert_ok!(OriginAndGate::approve(
			RuntimeOrigin::signed(BOB),
			call_hash,
			ALICE_ORIGIN_ID,
			BOB_ORIGIN_ID,
		));

		// Verify proposal exists and has both approvals
		let proposal = Proposals::<Test>::get(call_hash, ALICE_ORIGIN_ID).unwrap();
		assert_eq!(proposal.approvals.len(), 2);

		// Verify both origin IDs are in approvals
		assert!(proposal.approvals.contains(&ALICE_ORIGIN_ID));
		assert!(proposal.approvals.contains(&BOB_ORIGIN_ID));

		// Verify appropriate events were emitted
		System::assert_has_event(RuntimeEvent::OriginAndGate(Event::ProposalApproved {
			proposal_hash: call_hash,
			origin_id: ALICE_ORIGIN_ID,
			approving_origin_id: BOB_ORIGIN_ID,
		}));
	});
}

#[test]
fn test_origin_ids() {
	new_test_ext().execute_with(|| {
		// Verify origin ID constants match expected values
		assert_eq!(ALICE_ORIGIN_ID, 10);
		assert_eq!(BOB_ORIGIN_ID, 20);
		assert_eq!(CHARLIE_ORIGIN_ID, 30);
		assert_eq!(ROOT_ORIGIN_ID, 0);
	});
}
