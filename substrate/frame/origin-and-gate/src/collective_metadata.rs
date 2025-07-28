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

//! Collective metadata module for the origin-and-gate pallet.
//!
//! This module provides comprehensive metadata structures for collective origins
//! used in the origin-and-gate pallet. It includes definitions for origin types,
//! rank structures, expertise areas, and principles assessment derived from
//! official governance documents of various collectives.
//!
//! # Migration Path
//!
//! This module is currently housed within the origin-and-gate pallet due to its
//! tight integration with the pallet's collective origin handling mechanisms. The
//! origin-and-gate pallet specifically needs this metadata to:
//!
//! 1. Distinguish between standard and whitelisted collective origins
//! 2. Apply appropriate privilege levels based on collective identity
//! 3. Support the CompositeOriginId structure with collective_id and role fields
//!
//! In the future, this module is designed to be extracted into a standalone pallet
//! when:
//! - Multiple pallets require access to collective metadata
//! - The metadata grows in complexity beyond what's needed for origin authorization
//! - Separate governance mechanisms are needed for metadata management
//!
//! The module is structured with clear boundaries and minimal dependencies on the
//! host pallet to facilitate this future migration.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedBTreeMap, BoundedVec, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// Constants and type definitions
// -----------------------------

/// Maximum length for names
pub type MaxNameLength = frame_support::traits::ConstU32<64>;

/// Maximum length for rank titles
pub type MaxRankTitleLength = frame_support::traits::ConstU32<64>;

/// Maximum number of rank titles
pub type MaxRankTitles = frame_support::traits::ConstU32<20>;

/// Maximum number of ranks
pub type MaxRanks = frame_support::traits::ConstU32<10>;

/// Maximum length for collective type names
pub type MaxCollectiveTypeNameLength = frame_support::traits::ConstU32<64>;

/// Maximum length for expertise areas
pub type MaxExpertiseAreaLength = frame_support::traits::ConstU32<64>;

/// Maximum number of expertise areas
pub type MaxExpertiseAreas = frame_support::traits::ConstU32<10>;

// Collective IDs
pub const TECH_FELLOWSHIP: u32 = 4;
pub const SOCIETY: u32 = 5;
pub const AMBASSADOR_FELLOWSHIP: u32 = 6;

// Core types
// -----------------------------

/// Origin type with comprehensive metadata
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum OriginType<AccountId, BlockNumber> {
    /// A signed origin with an account ID
    Signed(AccountId),
    /// The root origin
    Root,
    /// A collective origin with detailed metadata
    Collective {
        /// Collective identifier
        id: u32,
        /// Collective name
        name: BoundedVec<u8, MaxNameLength>,
        /// Rank structure (based on Fellowship model)
        rank_structure: RankStructure,
        /// Authorization privilege level (0-255 scale)
        privilege_level: u8,
        /// Domain expertise (critical for governance)
        expertise: BoundedBTreeMap<ExpertiseArea, ExpertiseLevel, MaxExpertiseAreas>,
        /// Fellowship-specific metrics
        fellowship_metrics: FellowshipMetrics<BlockNumber>,
    },
    /// No origin
    None,
}

/// Enhanced rank structure supporting all collective types
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RankStructure {
    /// Collective type
    pub collective_type: CollectiveType,
    /// Maximum rank in this collective
    pub max_rank: u8,
    /// Maps rank titles to their numerical values
    pub rank_thresholds: BoundedBTreeMap<RankTitle, u8, MaxRankTitles>,
    /// Maps rank to count of members at that rank
    pub member_distribution: BoundedBTreeMap<u8, u32, MaxRanks>,
    /// Maps rank to voting weight
    pub rank_weights: BoundedBTreeMap<u8, u32, MaxRanks>,
}

/// Types of collectives supported in the system
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub enum CollectiveType {
    /// Technical Fellowship collective
    TechnicalFellowship,
    /// Ambassador Fellowship collective
    AmbassadorFellowship,
    /// Other collective types
    Other(BoundedVec<u8, MaxCollectiveTypeNameLength>),
}

/// Rank titles across different collectives
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub enum RankTitle {
    // Technical Fellowship titles
    /// Candidate rank in Technical Fellowship
    Candidate,
    /// Member rank in Technical Fellowship
    Member,
    /// Fellow rank in Technical Fellowship
    Fellow,
    /// Architect rank in Technical Fellowship
    Architect,
    /// Master rank in Technical Fellowship
    Master,

    // Ambassador Fellowship titles
    /// Advocate Ambassador (Rank 0)
    AdvocateAmbassador,
    /// Associate Ambassador (Rank I)
    AssociateAmbassador,
    /// Lead Ambassador (Rank II)
    LeadAmbassador,
    /// Senior Ambassador (Rank III)
    SeniorAmbassador,
    /// Principal Ambassador (Rank IV)
    PrincipalAmbassador,
    /// Global Ambassador (Rank V)
    GlobalAmbassador,
    /// Global Head Ambassador (Rank VI)
    GlobalHeadAmbassador,

    // Society titles
    /// Candidate member of Society
    SocietyCandidate,
    /// Suspended member of Society
    SocietySuspended,
    /// Regular member of Society
    SocietyMember,
    /// Defender role in Society (skeptic)
    SocietyDefender,
    /// Head of Society (elected leader)
    SocietyHead,

    // Custom title
    /// Custom rank title
    Custom(BoundedVec<u8, MaxRankTitleLength>),
}

// Domain expertise
// -----------------------------

/// Comprehensive expertise areas from all collectives
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub enum ExpertiseArea {
    // Technical Fellowship areas (from Technical Fellowship Manifesto)
    /// Runtime Development expertise
    RuntimeDevelopment,
    /// Consensus Algorithms expertise
    ConsensusAlgorithms,
    /// Cryptographic Protocols expertise
    CryptographicProtocols,
    /// Parachain Consensus expertise
    ParachainConsensus,
    /// Cross-Chain Messaging expertise
    CrossChainMessaging,
    /// Peer Networking expertise
    PeerNetworking,
    /// Topology Strategies expertise
    TopologyStrategies,
    /// Chain Synchronization expertise
    ChainSynchronization,
    /// FRAME Pallets expertise
    FramePallets,
    /// XCM Specification expertise
    XcmSpecification,
    /// Security Auditing expertise
    SecurityAuditing,

    // Ambassador Fellowship areas (from Ambassador Fellowship Manifesto)
    /// Community Building expertise
    CommunityBuilding,
    /// Education expertise
    Education,
    /// Translation expertise
    Translation,
    /// Event Organization expertise
    EventOrganization,
    /// Content Creation expertise
    ContentCreation,
    /// Advocacy And Governance expertise
    AdvocacyAndGovernance,
    /// Technical Development expertise
    TechnicalDevelopment,
    /// Partnerships expertise
    Partnerships,

    // Shared areas
    /// Governance Process expertise
    GovernanceProcess,
    /// Economic Systems expertise
    EconomicSystems,
    /// Decentralized Coordination expertise
    DecentralizedCoordination,

    // Custom area
    /// Custom expertise area
    Custom(BoundedVec<u8, MaxExpertiseAreaLength>),
}

/// Three-level expertise scale as mentioned in the Technical Fellowship
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub enum ExpertiseLevel {
    /// Basic knowledge (equivalent to rank 1-2)
    Member,
    /// Significant experience (equivalent to rank 3-6)
    Fellow,
    /// Recognized authority (equivalent to rank 7+)
    Master,
}

// Metrics and assessment
// -----------------------------

/// Enhanced metrics tracking for all collective types
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct FellowshipMetrics<BlockNumber> {
    // Common governance metrics
    /// Number of proposals created
    pub proposals_created: u32,
    /// Number of proposals approved
    pub proposals_approved: u32,
    /// Number of proposals executed
    pub proposals_executed: u32,

    // Technical Fellowship specific
    /// Number of code contributions
    pub code_contributions: u32,
    /// Number of security reviews
    pub security_reviews: u32,

    // Ambassador Fellowship specific
    /// Number of community events
    pub community_events: u32,
    /// Number of educational materials
    pub educational_materials: u32,
    /// Number of translations
    pub translations: u32,

    // Activity tracking
    /// Block number of last action
    pub last_action_block: BlockNumber,
    /// Block number of induction
    pub induction_block: BlockNumber,

    // Principles assessment
    /// Assessment of adherence to principles
    pub principles_assessment: PrinciplesAssessment,
}

/// Comprehensive principles assessment incorporating values from all collectives
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct PrinciplesAssessment {
    // Technical Fellowship tenets and principles (from Technical Fellowship Manifesto)
    /// Assessment of upholding Polkadot interests (0-100)
    pub uphold_polkadot_interests: u8,
    /// Assessment of respecting philosophy (0-100)
    pub respect_philosophy: u8,
    /// Assessment of respecting procedures (0-100)
    pub respect_procedures: u8,
    /// Assessment of respecting members (0-100)
    pub respect_members: u8,
    /// Assessment of enlightened liberalism (0-100)
    pub enlightened_liberalism: u8,
    /// Assessment of critical rationalism (0-100)
    pub critical_rationalism: u8,
    /// Assessment of web3 values (0-100)
    pub web3_values: u8,
    /// Assessment of decentralization (0-100)
    pub decentralization: u8,

    // Ambassador Fellowship values (from Ambassador Fellowship Manifesto)
    /// Assessment of inclusivity (0-100)
    pub inclusivity: u8,
    /// Assessment of collaboration (0-100)
    pub collaboration: u8,
    /// Assessment of continuous development (0-100)
    pub continuous_development: u8,
    /// Assessment of integrity (0-100)
    pub integrity: u8,
    /// Assessment of innovation (0-100)
    pub innovation: u8,

    // Shared governance values
    /// Assessment of transparency (0-100)
    pub transparency: u8,
    /// Assessment of accountability (0-100)
    pub accountability: u8,
}

// Helper functions and trait implementations
// -----------------------------

/// Helper functions for working with collective metadata
pub mod helpers {
    use super::*;

    /// Determines if a collective is a whitelisted collective with elevated privileges
    pub fn is_whitelisted_collective(collective_id: u32) -> bool {
        // Technical Fellowship, Society, and Ambassador Fellowship are whitelisted collectives
        collective_id == TECH_FELLOWSHIP || collective_id == SOCIETY || collective_id == AMBASSADOR_FELLOWSHIP
    }

    /// Gets the privilege level for a collective
    pub fn get_collective_privilege_level(collective_type: &CollectiveType) -> u8 {
        match collective_type {
            CollectiveType::TechnicalFellowship => 255, // Highest privilege
            CollectiveType::AmbassadorFellowship => 200,
            CollectiveType::Other(_) => 100, // Default privilege level
        }
    }

    /// Check if the collective has permission to execute a specific call index
    /// This enforces that only whitelisted collectives can perform certain privileged operations
    pub fn collective_can_execute_call(collective_id: u32, call_index: u8) -> bool {
        match call_index {
            // propose - index 0: All collectives can propose
            0 => true,
            // add_approval - index 1: All collectives can approve
            1 => true,
            // amend_remark - index 2: All collectives can amend remarks
            2 => true,
            // execute_proposal - index 3: All collectives can execute their own proposals
            3 => true,
            // cancel_proposal - index 4: All collectives can cancel their own proposals
            4 => true,
            // withdraw_proposal - index 5: All collectives can withdraw their own approvals
            5 => true,
            // clean - index 7: All collectives can clean their own proposals
            7 => true,
            // add_storage_id - index 8: All collectives can add storage IDs to their own proposals
            8 => true,
            // remove_storage_id - index 9: Only whitelisted collectives can remove any storage ID,
            // non-whitelisted collectives can only remove storage IDs they added themselves
            9 => is_whitelisted_collective(collective_id),
            // Any other call index: Only whitelisted collectives can execute
            _ => is_whitelisted_collective(collective_id),
        }
    }

    /// Check if a collective can remove a specific storage ID
    /// Non-whitelisted collectives can only remove storage IDs they added themselves
    pub fn collective_can_remove_storage_id(
        collective_id: u32,
        storage_id_adder: &[u8],
        collective_account: &[u8]
    ) -> bool {
        // Whitelisted collectives can remove any storage ID
        if is_whitelisted_collective(collective_id) {
            return true;
        }

        // Non-whitelisted collectives can only remove storage IDs they added themselves
        storage_id_adder == collective_account
    }
}

// Public API for external use
// -----------------------------

/// Public API for collective metadata
pub mod api {
    use super::*;

    /// Creates a new collective origin type
    pub fn create_collective_origin<AccountId, BlockNumber>(
        id: u32,
        name: BoundedVec<u8, MaxNameLength>,
        collective_type: CollectiveType,
    ) -> OriginType<AccountId, BlockNumber> {
        // This is a simplified implementation - in a real system you would
        // populate all the fields with appropriate values
        OriginType::Collective {
            id,
            name,
            rank_structure: RankStructure {
                collective_type,
                max_rank: 0,
                rank_thresholds: BoundedBTreeMap::new(),
                member_distribution: BoundedBTreeMap::new(),
                rank_weights: BoundedBTreeMap::new(),
            },
            privilege_level: super::helpers::get_collective_privilege_level(&collective_type),
            expertise: BoundedBTreeMap::new(),
            fellowship_metrics: FellowshipMetrics {
                proposals_created: 0,
                proposals_approved: 0,
                proposals_executed: 0,
                code_contributions: 0,
                security_reviews: 0,
                community_events: 0,
                educational_materials: 0,
                translations: 0,
                last_action_block: BlockNumber::default(),
                induction_block: BlockNumber::default(),
                principles_assessment: PrinciplesAssessment {
                    uphold_polkadot_interests: 0,
                    respect_philosophy: 0,
                    respect_procedures: 0,
                    respect_members: 0,
                    enlightened_liberalism: 0,
                    critical_rationalism: 0,
                    web3_values: 0,
                    decentralization: 0,
                    inclusivity: 0,
                    collaboration: 0,
                    continuous_development: 0,
                    integrity: 0,
                    innovation: 0,
                    transparency: 0,
                    accountability: 0,
                },
            },
        }
    }

    /// Extracts the collective ID from an origin type
    pub fn get_collective_id<AccountId, BlockNumber>(
        origin: &OriginType<AccountId, BlockNumber>,
    ) -> Option<u32> {
        match origin {
            OriginType::Collective { id, .. } => Some(*id),
            _ => None,
        }
    }
}
