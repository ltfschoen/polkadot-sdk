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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode};
use core::marker::PhantomData;
use frame_support::{
	dispatch::{
		ClassifyDispatch, DispatchClass, DispatchResult, GetDispatchInfo, Pays, PaysFee, WeighData,
	},
	pallet_prelude::*,
	traits::{EnsureOrigin, Get, IsSubType},
	weights::Weight,
};
use frame_system::{
	self, ensure_signed,
	pallet_prelude::{BlockNumberFor, *},
	Origin as RuntimeOrigin,
};
use log::info;
use scale_info::TypeInfo;
use sp_runtime::{
	impl_tx_ext_default,
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, DispatchInfoOf, DispatchOriginOf, Dispatchable,
		Member, One, SaturatedConversion, Saturating, TransactionExtension, ValidateResult, Zero,
	},
	transaction_validity::{InvalidTransaction, ValidTransaction},
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

/// Type alias for balance type from balances pallet.
// TODO: Remove use of balance pallet since it does not appear to be required
pub type BalanceOf<T> = <T as pallet_balances::Config>::Balance;

/// Helper struct that requires approval from two origins.
pub struct AndGate<A, B>(PhantomData<(A, B)>);

/// Implementation of `EnsureOrigin` that requires approval from two different origins
/// to succeed. It creates a compound origin check where both origin A and origin B
/// must approve for overall check to pass. Used in asynchronous approval flow where
/// multiple origins need to independently approve a proposal over time.
impl<Origin, A, B> frame_support::traits::EnsureOrigin<Origin> for AndGate<A, B>
where
	Origin: Into<Result<frame_system::RawOrigin<Origin::AccountId>, Origin>>
		+ From<frame_system::RawOrigin<Origin::AccountId>>
		+ Clone,
	Origin: frame_support::traits::OriginTrait,
	A: EnsureOrigin<Origin, Success = ()>,
	B: EnsureOrigin<Origin, Success = ()>,
{
	type Success = ();

	fn try_origin(origin: Origin) -> Result<Self::Success, Origin> {
		let origin_clone = origin.clone();
		match A::try_origin(origin) {
			Ok(_) => B::try_origin(origin_clone),
			Err(_) => Err(origin_clone),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<Origin, ()> {
		// Placeholder implementation for benchmarking to create a successful origin from A
		A::try_successful_origin()
	}
}

struct WeightForSetDummy<T: pallet_balances::Config>(BalanceOf<T>);

impl<T: pallet_balances::Config> WeighData<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
	fn weigh_data(&self, _target: (&BalanceOf<T>,)) -> Weight {
		Weight::from_parts(100_000, 0)
	}
}

impl<T: pallet_balances::Config> ClassifyDispatch<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
	fn classify_dispatch(&self, _target: (&BalanceOf<T>,)) -> DispatchClass {
		DispatchClass::Normal
	}
}

impl<T: pallet_balances::Config> PaysFee<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
	fn pays_fee(&self, _target: (&BalanceOf<T>,)) -> Pays {
		Pays::Yes
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::{WeightInfo, *};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{Dispatchable, Hash, One};
	use sp_std::{fmt::Debug, marker::PhantomData};

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;

		/// The hashing implementation.
		type Hashing: sp_runtime::traits::Hash;

		/// Identifier type for different origins that must maintain uniqueness and comparability.
		type OriginId: Parameter + Member + TypeInfo + Copy + Ord + MaxEncodedLen;

		/// The maximum number of approvals for a single proposal.
		/// The original specification by Dr Gavin Wood requires exactly two approvals to satisfy
		/// the "AND Gate" pattern for two origins.
		#[pallet::constant]
		type MaxApprovals: Get<u32> + Clone;

		/// How long a proposal is valid for measured in blocks before it expires.
		#[pallet::constant]
		type ProposalLifetime: Get<BlockNumberFor<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Governance origin type.
		type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			Weight::zero()
		}

		fn on_finalize(_n: BlockNumberFor<T>) {
			// TODO
		}

		fn offchain_worker(_n: BlockNumberFor<T>) {
			// TODO
		}
	}

	impl<T: Config> Pallet<T> {
		/// Helper function to get error index of specific error variant
		fn error_index(error: Error<T>) -> u8 {
			match error {
				Error::ProposalAlreadyExists => 0,
				Error::ProposalNotFound => 1,
				Error::CannotApproveOwnProposalUsingDifferentOrigin => 2,
				Error::TooManyApprovals => 3,
				Error::NotAuthorized => 4,
				Error::ProposalAlreadyExecuted => 5,
				Error::ProposalExpired => 6,
				Error::ProposalNotExpired => 7,
				Error::ProposalCancelled => 8,
				Error::OriginAlreadyApproved => 9,
				Error::InsufficientApprovals => 10,
				Error::ProposalNotPending => 11,
				Error::OriginApprovalNotFound => 12,
				Error::FailedToExecute => 13,
			}
		}

		/// Helper function to attempt execution of a proposal with sufficient approvals
		fn maybe_execute(
			proposal_hash: &T::Hash,
			origin_id: &T::OriginId,
			mut proposal: ProposalInfo<T::Hash, BlockNumberFor<T>, T::OriginId, T::AccountId, T::MaxApprovals>,
		) -> Result<(), DispatchError> {
			// Only attempt to execute if proposal is still Pending
			if proposal.status != ProposalStatus::Pending {
				return Ok(());
			}

			// Check if we have enough approvals
			if proposal.approvals.len() as u32 >= T::MaxApprovals::get() {
				// Create timepoint for this execution
				let now = <frame_system::Pallet<T>>::block_number();
				let current_timepoint = <CurrentTimepoint<T>>::get().unwrap_or(TimePoint {
					height: BlockNumberFor::<T>::zero(),
					index: 0
				});
				let index = current_timepoint.index + 1;
				let time = TimePoint { height: now, index };

				// Get the call that was stored
				let maybe_call = <ProposalCalls<T>>::get(proposal_hash);
				if let Some(call) = maybe_call {
					// Store timepoint and update proposal
					<CurrentTimepoint<T>>::put(time);
					<ExecutedCalls<T>>::insert(time, *proposal_hash);

					// Update proposal status
					proposal.status = ProposalStatus::Executed;
					<Proposals<T>>::insert(proposal_hash, origin_id, proposal);

					// Dispatch the call with root origin
					let result = call.dispatch(RuntimeOrigin::root());

					// Clean up approvals storage
					<Approvals<T>>::remove_prefix((proposal_hash, origin_id.clone()), None);

					// Clear call storage
					<ProposalCalls<T>>::remove(proposal_hash);

					// Emit event for proposal execution
					Self::deposit_event(Event::ProposalExecuted {
						proposal_hash: *proposal_hash,
						origin_id: origin_id.clone(),
						result: result.map(|_| ()).map_err(|e| e.error),
					});

					// Return result
					if result.is_ok() {
						Ok(())
					} else {
						Err(Error::<T>::FailedToExecute.into())
					}
				} else {
					Ok(())
				}
			} else {
				Ok(())
			}
		}

		/// Check if proposal expired and if so update its storage status
		/// Returns true if expired, false otherwise
		fn is_expired(
			proposal_hash: &T::Hash,
			origin_id: &T::OriginId,
			proposal: &mut ProposalInfo<T::Hash, BlockNumberFor<T>, T::OriginId, T::AccountId, T::MaxApprovals>,
		) -> bool {
			if let Some(expiry) = proposal.expiry {
				let current_block = <frame_system::Pallet<T>>::block_number();
				if current_block > expiry {
					// Update status in proposal info
					proposal.status = ProposalStatus::Expired;

					// Update proposal in storage
					<Proposals<T>>::insert(proposal_hash, origin_id, proposal.clone());

					// Emit expiry event
					Self::deposit_event(Event::ProposalExpired {
						proposal_hash: *proposal_hash,
						origin_id: origin_id.clone(),
					});

					return true;
				}
			}

			false
		}
	}

	#[pallet::call(weight(<T as Config>::WeightInfo))]
	impl<T: Config> Pallet<T> {
		/// Submit a proposal for approval, recording the first origin's approval.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::propose())]
		pub fn propose(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
			origin_id: T::OriginId,
			expiry: Option<BlockNumberFor<T>>,
		) -> DispatchResultWithPostInfo {
			// Check extrinsic was signed
			let who = ensure_signed(origin)?;

			// Compute hash of call for storage using system hashing implementation
			let proposal_hash = <T as frame_system::Config>::Hashing::hash_of(&call);

			// Check if given proposal already exists
			ensure!(
				!<Proposals<T>>::contains_key(&proposal_hash, &origin_id),
				Error::<T>::ProposalAlreadyExists
			);

			// Get current block number
			let current_block = frame_system::Pallet::<T>::block_number();

			// Determine expiration block number if provided or otherwise use default
			let expiry_block = match expiry {
				Some(expiry_block) => {
					// Check expiry not in the past
					ensure!(current_block <= expiry_block, Error::<T>::ProposalExpired);
					Some(expiry_block)
				},
				None => {
					// If no expiry was provided then use proposal lifetime config
					Some(current_block.saturating_add(T::ProposalLifetime::get()))
				},
			};

			// Create an empty bounded vec for approvals
			let mut approvals = BoundedVec::<T::OriginId, T::MaxApprovals>::default();

			// Add proposer as first approval
			if let Err(_) = approvals.try_push(origin_id.clone()) {
				return Err(Error::<T>::TooManyApprovals.into());
			}

			// Create and store proposal metadata (bounded storage)
			let proposal_info = ProposalInfo {
				call_hash: proposal_hash,
				expiry: expiry_block,
				approvals,
				status: ProposalStatus::Pending,
				proposer: who.clone(),
			};

			// Store proposal metadata (bounded storage)
			<Proposals<T>>::insert(proposal_hash, origin_id.clone(), proposal_info);

			// Mark first approval in approvals storage (bounded)
			<Approvals<T>>::insert(
				(proposal_hash, origin_id.clone()),
				origin_id.clone(),
				who.clone(),
			);

			// Store actual call data (unbounded)
			<ProposalCalls<T>>::insert(proposal_hash, call);

			// Emit event
			Self::deposit_event(Event::ProposalCreated { proposal_hash, origin_id });

			Ok(().into())
		}

		/// Approve a previously submitted proposal.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_approval())]
		pub fn add_approval(
			origin: OriginFor<T>,
			call_hash: T::Hash,
			proposal_origin_id: T::OriginId,
			approval_origin_id: T::OriginId,
		) -> DispatchResult {
			// Check call sender is authorized
			let who = ensure_signed(origin)?;

			// Ensure proposal exists
			let mut proposal_info = <Proposals<T>>::get(&call_hash, &proposal_origin_id)
				.ok_or(Error::<T>::ProposalNotFound)?;

			// Check if proposal is expired
			if Self::is_expired(&call_hash, &proposal_origin_id, &mut proposal_info) {
				return Err(Error::<T>::ProposalExpired.into());
			}

			// Ensure proposal is in Pending state
			ensure!(
				proposal_info.status == ProposalStatus::Pending,
				Error::<T>::ProposalNotPending
			);

			// Prevent self-approval with different origin
			if proposal_info.proposer == who && proposal_origin_id.clone() != approval_origin_id.clone() {
				return Err(Error::<T>::CannotApproveOwnProposalUsingDifferentOrigin.into());
			}

			// Check if the origin already approved this proposal
			ensure!(
				!<Approvals<T>>::contains_key((&call_hash, &proposal_origin_id), &approval_origin_id),
				Error::<T>::OriginAlreadyApproved,
			);

			// Add to storage to mark this origin as approved and who approved
			<Approvals<T>>::insert(
				(&call_hash, &proposal_origin_id),
				&approval_origin_id,
				who.clone(),
			);

			// Try to add approval to list of approvals of the proposal
			if !proposal_info.approvals.contains(&approval_origin_id) {
				if proposal_info.approvals.try_push(approval_origin_id).is_err() {
					return Err(Error::<T>::TooManyApprovals.into());
				}
			}

			// Update proposal in storage with new origin approval
			<Proposals<T>>::insert(call_hash, origin_id.clone(), &proposal_info);

			// Emit approval event
			Self::deposit_event(Event::OriginApprovalAdded {
				proposal_hash: call_hash,
				origin_id: proposal_origin_id.clone(),
				approval_origin_id: approval_origin_id.clone(),
				who: who.clone(),
			});

			// Pass clone of proposal info so original not modified if execution attempt fails
			Self::maybe_execute(&call_hash, &proposal_origin_id, proposal_info)
				.map_err(|e| e)?;

			Ok(().into())
		}

		/// Cancel pending proposal is only callable by original proposer
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_proposal())]
		pub fn cancel_proposal(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
		) -> DispatchResultWithPostInfo {
			// Check extrinsic was signed
			let who = ensure_signed(origin)?;

			// Get proposal info
			let mut proposal = Proposals::<T>::get(&proposal_hash, &origin_id)
				.ok_or(Error::<T>::ProposalNotFound)?;

			// Ensure proposal is in a pending state
			ensure!(proposal.status == ProposalStatus::Pending, Error::<T>::ProposalNotPending);

			// Ensure caller is original proposer
			ensure!(who == proposal.proposer, Error::<T>::NotAuthorized);

			// Update proposal status to Cancelled
			// TODO: Potentially remove since this is just for completeness
			// and potentially unnecessary since status will be removed from storage
			// shortly after and proposal cancelled event will be emitted
			// at lower cost
			proposal.status = ProposalStatus::Cancelled;

			// Remove all approvals from Approvals storage efficiently
			<Approvals<T>>::remove_prefix((proposal_hash, origin_id.clone()), None);

			// Update storage with cancelled status
			Proposals::<T>::insert(&proposal_hash, &origin_id, proposal);

			// Remove actual call data (unbounded) to save storage
			<ProposalCalls<T>>::remove(proposal_hash);

			// Remove proposal from storage to remove all approvals
			Proposals::<T>::remove(&proposal_hash, &origin_id);

			// Emit event
			Self::deposit_event(Event::ProposalCancelled { proposal_hash, origin_id });

			Ok(().into())
		}

		/// Remove an expired proposal from storage.
		///
		/// Dispatch origin of call must be signed by original proposer
		/// or governance. Use batch call to remove multiple expired proposals.
		///
		/// - `call_hash`: Hash of call to be executed.
		/// - `origin_id`: Origin identifier used to create proposal.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_proposal())]
		pub fn remove(
			origin: OriginFor<T>,
			call_hash: T::Hash,
			origin_id: T::OriginId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check proposal exists
			let proposal_info =
				<Proposals<T>>::get(&call_hash, &origin_id).ok_or(Error::<T>::ProposalNotFound)?;

			// Check if proposal expired
			match proposal_info.status {
				ProposalStatus::Expired => {},
				_ => return Err(Error::<T>::ProposalNotExpired.into()),
			}

			// Check authorization to remove expired proposal
			let is_proposer = who == proposal_info.proposer;
			let is_governance = T::GovernanceOrigin::try_origin(origin.clone()).is_ok();

			ensure!(is_proposer || is_governance, Error::<T>::NotAuthorized);

			// Remove proposal from storage
			<ProposalCalls<T>>::remove(&call_hash);
			<Approvals<T>>::remove_prefix((&call_hash, &origin_id), None);
			<Proposals<T>>::remove(&call_hash, &origin_id);

			// Deposit event
			Self::deposit_event(Event::ProposalRemoved {
				proposal_hash: call_hash,
				origin_id,
				who,
			});

			Ok(().into())
		}

		/// Withdraw an approval for the proposal associated with an origin.
		///
		/// Only callable by an origin that has approved the proposal.
		///
		/// - `origin`: Must be a valid authority (i.e. entity) that approves the proposal.
		/// - `proposal_hash`: The proposal hash to withdraw approval for.
		/// - `origin_id`: The origin id that the proposal belongs to.
		/// - `withdrawing_origin_id`: The origin id to withdraw the approval for since the account
		///   might need to specify which of their multiple origin authorities they approved with
		///   that they are now withdrawing approval for.
		#[pallet::call_index(4)]
		#[pallet::weight((<T as pallet::Config>::WeightInfo::withdraw_approval(), DispatchClass::Normal))]
		pub fn withdraw_approval(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
			withdrawing_origin_id: T::OriginId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Get proposal info
			let mut proposal = Proposals::<T>::get(&proposal_hash, &origin_id)
				.ok_or(Error::<T>::ProposalNotFound)?;

			// Check if proposal expired and update its status if needed
			if Self::is_expired(&proposal_hash, &origin_id, &mut proposal) {
				return Err(Error::<T>::ProposalExpired.into());
			}

			ensure!(proposal.status == ProposalStatus::Pending, Error::<T>::ProposalNotPending);

			// Verify approval exists and check authorisation such that only original approver can
			// withdraw their approval using our mapping from OriginId to AccountId where only
			// the account that originally granted approval can withdraw it
			let approval_account = <Approvals<T>>::get(
				(proposal_hash, origin_id.clone()),
				withdrawing_origin_id.clone(),
			)
			.ok_or(Error::<T>::OriginApprovalNotFound)?;

			ensure!(approval_account == who, Error::<T>::NotAuthorized);

			// Find position of withdrawing_origin_id in approvals vector
			let pos = proposal
				.approvals
				.iter()
				.position(|a| a == &withdrawing_origin_id)
				.ok_or(Error::<T>::OriginApprovalNotFound)?;

			// Remove approval at found position
			proposal.approvals.swap_remove(pos);

			// Update proposal in storage
			Proposals::<T>::insert(&proposal_hash, &origin_id, &proposal);

			// Remove approval from Approvals storage
			<Approvals<T>>::remove((proposal_hash, origin_id), withdrawing_origin_id);

			// Emit event
			Self::deposit_event(Event::OriginApprovalWithdrawn {
				proposal_hash,
				origin_id,
				withdrawing_origin_id,
			});

			Ok(().into())
		}

		/// A privileged call; in this case it resets our dummy value to something new.
		/// Implementation of a privileged call. The `origin` parameter is ROOT because
		/// it's not (directly) from an extrinsic, but rather the system as a whole has decided
		/// to execute it. Different runtimes have different reasons for allow privileged
		/// calls to be executed - we don't need to care why. Because it's privileged, we can
		/// assume it's a one-off operation and substantial processing/storage/memory can be used
		/// without worrying about gameability or attack scenarios.
		///
		/// The weight for this extrinsic we use our own weight object `WeightForSetDummy`
		/// or set_dummy() extrinsic to determine its weight
		#[pallet::call_index(5)]
		// #[pallet::weight(WeightForSetDummy::<T>(<BalanceOf<T>>::from(100u64.into())))]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_dummy())]
		pub fn set_dummy(
			origin: OriginFor<T>,
			#[pallet::compact] new_value: T::Balance,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Print out log or debug message in the console via log::{error, warn, info, debug,
			// trace}, accepting format strings similar to `println!`.
			// https://paritytech.github.io/substrate/master/sp_io/logging/fn.log.html
			// https://paritytech.github.io/substrate/master/frame_support/constant.LOG_TARGET.html
			info!("New value is now: {:?}", new_value);

			// Put the new value into storage.
			<Dummy<T>>::put(new_value);

			Self::deposit_event(Event::SetDummy { balance: new_value });

			// All good, no refund.
			Ok(())
		}

		// /// A dummy function for use in tests and benchmarks
		// #[pallet::call_index(3)]
		// #[pallet::weight(<T as pallet::Config>::WeightInfo::set_dummy())]
		// pub fn dummy_benchmark(
		//     origin: OriginFor<T>,
		//     remark: Vec<u8>,
		// ) -> DispatchResultWithPostInfo {
		//     ensure_signed(origin)?;
		//     Ok(().into())
		// }
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A proposal has been created.
		ProposalCreated {
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
		},
		/// An origin has added their approval of a proposal.
		OriginApprovalAdded {
			/// Hash of the proposal.
			proposal_hash: T::Hash,
			/// Origin identifier.
			origin_id: T::OriginId,
			/// Origin identifier of the approval origin.
			approval_origin_id: T::OriginId,
			/// Account that added the approval.
			who: T::AccountId,
		},
		/// A proposal has been executed.
		ProposalExecuted {
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
			result: DispatchResult,
		},
		/// A proposal has expired.
		ProposalExpired {
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
		},
		/// A proposal has been cancelled.
		ProposalCancelled {
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
		},
		/// An origin has withdrawn their approval of a proposal.
		OriginApprovalWithdrawn {
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
			withdrawing_origin_id: T::OriginId,
		},
		/// A proposal has been removed.
		ProposalRemoved {
			proposal_hash: T::Hash,
			origin_id: T::OriginId,
			who: T::AccountId,
		},
		SetDummy {
			balance: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A proposal with these parameters already exists
		ProposalAlreadyExists,
		/// The proposal could not be found
		ProposalNotFound,
		/// The proposer cannot approve their own proposal with different origin ID
		CannotApproveOwnProposalUsingDifferentOrigin,
		/// The proposal has too many approvals
		TooManyApprovals,
		/// The caller is not authorized to approve
		NotAuthorized,
		/// The proposal is not pending
		ProposalNotPending,
		/// The proposal has already been executed
		ProposalAlreadyExecuted,
		/// The proposal has expired
		ProposalExpired,
		/// The proposal is not expired
		ProposalNotExpired,
		/// The proposal was cancelled
		ProposalCancelled,
		/// The proposal was already approved by the origin
		OriginAlreadyApproved,
		/// The proposal does not have enough approvals
		InsufficientApprovals,
		/// The origin approval could not be found
		OriginApprovalNotFound,
		/// Failed to execute proposal
		FailedToExecute,
	}

	/// Status of proposal
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum ProposalStatus {
		/// Proposal is pending and awaiting approvals
		Pending,
		/// Proposal has been executed
		Executed,
		/// Proposal has expired
		Expired,
		/// Proposal has been cancelled
		Cancelled,
	}

	/// Info about specific proposal
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(MaxApprovals))]
	pub struct ProposalInfo<Hash, BlockNumber, OriginId, AccountId, MaxApprovals: Get<u32>> {
		/// The call hash of this proposal to execute
		pub call_hash: Hash,
		/// The block number after which this proposal expires
		pub expiry: Option<BlockNumber>,
		/// List of origins that have approved this proposal
		pub approvals: BoundedVec<OriginId, MaxApprovals>,
		/// The current status of this proposal
		pub status: ProposalStatus,
		/// The original proposer of this proposal
		pub proposer: AccountId,
	}

	/// Timepoint is combination of block number and extrinsic index in that block
	#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct TimePoint<BlockNumber> {
		/// Block number
		pub height: BlockNumber,
		/// Extrinsic index
		pub index: u32,
	}

	/// Storage for proposals
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::Hash,
		Blake2_128Concat,
		T::OriginId,
		ProposalInfo<T::Hash, BlockNumberFor<T>, T::OriginId, T::AccountId, T::MaxApprovals>,
		OptionQuery,
	>;

	/// Storage for approvals by `OriginId`
	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type Approvals<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(T::Hash, T::OriginId), // e.g. (proposal_hash, origin_id)
		Blake2_128Concat,
		T::OriginId,  // e.g. approval_origin_id or withdrawing_origin_id
		T::AccountId, // e.g. account that added the approval
		OptionQuery,
	>;

	/// Storage for calls themselves that is unbounded since
	/// `RuntimeCall` does not implement `MaxEncodedLen`
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn proposal_calls)]
	pub type ProposalCalls<T: Config> =
		StorageMap<_, Identity, T::Hash, Box<<T as Config>::RuntimeCall>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_timepoint)]
	pub(super) type CurrentTimepoint<T: Config> = StorageValue<_, TimePoint<BlockNumberFor<T>>>;

	/// Record of timepoints including block number and extrinsic index that is used for
	/// proposal execution or expiry
	#[pallet::storage]
	#[pallet::getter(fn executed_calls)]
	pub(super) type ExecutedCalls<T: Config> =
		StorageMap<_, Twox64Concat, TimePoint<BlockNumberFor<T>>, T::Hash>;

	#[pallet::storage]
	pub(super) type Dummy<T: Config> = StorageValue<_, T::Balance>;

	// The genesis config type.
	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		// TODO
		pub key: Option<T::AccountId>,
	}

	// The build of genesis for the pallet.
	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// TODO
		}
	}
}
