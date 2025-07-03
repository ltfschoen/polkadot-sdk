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

//! Unit tests that focus on testing individual pallet functions in isolation
//! rather than complex workflows covered by integration tests that are in ../tests

use super::*;
use crate::{self as pallet_origin_and_gate};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

// Import mock directly instead of through module import
#[path = "./mock.rs"]
mod mock;
use mock::*;

// TODO: Add unit tests
