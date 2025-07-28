```rust
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum OriginType<AccountId, BlockNumber> {
    Signed(AccountId),
    Root,
    Collective {
        // Core identity
        id: u32,                // Collective identifier
        name: BoundedVec<u8, MaxNameLength>,
        
        // Rank structure (based on Fellowship model)
        rank_structure: RankStructure,
        
        // Authorization
        privilege_level: u8,    // 0-255 scale
        
        // Domain expertise (critical for governance)
        expertise: BoundedBTreeMap<ExpertiseArea, ExpertiseLevel, MaxExpertiseAreas>,
        
        // Fellowship-specific metrics
        fellowship_metrics: FellowshipMetrics<BlockNumber>,
    },
    None,
}

// Enhanced rank structure supporting all collective types
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct RankStructure {
    // Collective type
    collective_type: CollectiveType,
    
    // Rank configuration
    max_rank: u8,
    rank_thresholds: BoundedBTreeMap<RankTitle, u8, MaxRankTitles>,
    
    // Rank distribution statistics
    member_distribution: BoundedBTreeMap<u8, u32, MaxRanks>, // Maps rank to count
    
    // Voting weights (for rank-weighted voting)
    rank_weights: BoundedBTreeMap<u8, u32, MaxRanks>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord)]
pub enum CollectiveType {
    TechnicalFellowship,
    AmbassadorFellowship,
    ToolingCollective,
    JamDao,
    Other(BoundedVec<u8, MaxCollectiveTypeNameLength>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord)]
pub enum RankTitle {
    // Technical Fellowship titles
    Candidate,
    Member,
    Fellow,
    Architect,
    Master,
    
    // Ambassador Fellowship titles
    AdvocateAmbassador,    // Rank 0
    AssociateAmbassador,   // Rank I
    LeadAmbassador,        // Rank II
    SeniorAmbassador,      // Rank III
    PrincipalAmbassador,   // Rank IV
    GlobalAmbassador,      // Rank V
    GlobalHeadAmbassador,  // Rank VI
    
    // Tooling Collective titles
    Candidate,
    Contributor,
    Maintainer,
    Lead,
    
    // JAM DAO roles
    JamMember,
    JamModerator,
    
    // Custom title
    Custom(BoundedVec<u8, MaxRankTitleLength>),
}

// Comprehensive expertise areas from all collectives
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord)]
pub enum ExpertiseArea {
    // Technical Fellowship areas (from Technical Fellowship Manifesto)
    RuntimeDevelopment,
    ConsensusAlgorithms,
    CryptographicProtocols,
    ParachainConsensus,
    CrossChainMessaging,
    PeerNetworking,
    TopologyStrategies,
    ChainSynchronization,
    FramePallets,
    XcmSpecification,
    SecurityAuditing,
    
    // Ambassador Fellowship areas (from Ambassador Fellowship Manifesto)
    CommunityBuilding,
    Education,
    Translation,
    EventOrganization,
    ContentCreation,
    AdvocacyAndGovernance,
    TechnicalDevelopment,
    Partnerships,
    
    // Tooling Collective areas (from Tooling Collective document)
    // Fundamental dApp Tooling
    DappFundamentals,
    TransactionHandling,
    BlockSubscription,
    StorageInteraction,
    // Extended dApp Tooling
    DappExtensions,
    // General Tooling
    GeneralTooling,
    CliTools,
    DevExperience,
    
    // JAM DAO areas (from JAM DAO Code of Conduct)
    CommunityModeration,
    ConflictResolution,
    ProfessionalRepresentation,
    
    // Shared areas
    GovernanceProcess,
    EconomicSystems,
    DecentralizedCoordination,
    
    // Custom area
    Custom(BoundedVec<u8, MaxExpertiseAreaLength>),
}

// Three-level expertise scale as mentioned in the Technical Fellowship
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord)]
pub enum ExpertiseLevel {
    Member,      // Basic knowledge (equivalent to rank 1-2)
    Fellow,      // Significant experience (equivalent to rank 3-6)
    Master,      // Recognized authority (equivalent to rank 7+)
}

// Enhanced metrics tracking for all collective types
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct FellowshipMetrics<BlockNumber> {
    // Common governance metrics
    proposals_created: u32,
    proposals_approved: u32,
    proposals_executed: u32,
    
    // Technical Fellowship specific
    code_contributions: u32,
    security_reviews: u32,
    
    // Ambassador Fellowship specific
    community_events: u32,
    educational_materials: u32,
    translations: u32,
    
    // Tooling Collective specific
    tools_maintained: u32,
    libraries_developed: u32,
    documentation_contributions: u32,
    
    // JAM DAO specific
    moderation_actions: u32,
    conflict_resolutions: u32,
    
    // Activity tracking
    last_action_block: BlockNumber,
    induction_block: BlockNumber,
    
    // Principles assessment
    principles_assessment: PrinciplesAssessment,
}

// Comprehensive principles assessment incorporating values from all collectives
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct PrinciplesAssessment {
    // Technical Fellowship tenets and principles (from Technical Fellowship Manifesto)
    uphold_polkadot_interests: u8,     // 0-100
    respect_philosophy: u8,            // 0-100
    respect_procedures: u8,            // 0-100
    respect_members: u8,               // 0-100
    enlightened_liberalism: u8,        // 0-100
    critical_rationalism: u8,          // 0-100
    web3_values: u8,                   // 0-100
    decentralization: u8,              // 0-100
    
    // Ambassador Fellowship values (from Ambassador Fellowship Manifesto)
    inclusivity: u8,                   // 0-100
    collaboration: u8,                 // 0-100
    continuous_development: u8,        // 0-100
    integrity: u8,                     // 0-100
    innovation: u8,                    // 0-100
    
    // JAM DAO values (from JAM DAO Code of Conduct)
    active_participation: u8,          // 0-100
    professional_representation: u8,   // 0-100
    information_accuracy: u8,          // 0-100
    constructive_engagement: u8,       // 0-100
    
    // Tooling Collective values (from Tooling Collective document)
    foss_compliance: u8,               // 0-100
    code_accessibility: u8,            // 0-100
    transparent_governance: u8,        // 0-100
    developer_success: u8,             // 0-100
    
    // Shared governance values
    transparency: u8,                  // 0-100
    accountability: u8,                // 0-100
}
```