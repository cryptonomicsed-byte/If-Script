pub mod archetype;
pub mod calabash;
pub mod compiler;
pub mod cosmogram;
pub mod ebo;
pub mod entropy;
pub mod error;
pub mod field;
pub mod hermetic;
pub mod larql;
pub mod manifesto;
pub mod odu;
pub mod odu_ifa;
pub mod receipt;
pub mod ritual_codex;
pub mod soul;
pub mod vm;
pub mod zangbeto;

// Error type
pub use error::IfaError;

// Core VM
pub use vm::{CastResult, IfaVM};

// 16 Action Vessels — primary architectural concept of the Digital Calabash
pub use odu::ActionVessel;

// Full Odù corpus access (Hive/Steward tier) — Digital Calabash, agent-native
pub use odu::{get_odu, get_odu_by_binary, lookup_by_name, Odu, ODU_SET};

// The traditional Òdù Ifá corpus (same index/vessel/opcode structure as
// `odu::ODU_SET`, Yorùbá vocabulary) — for agent-to-human readings.
pub use odu_ifa::{get_odu_ifa, get_odu_ifa_by_binary, lookup_by_name_ifa, ODU_SET_IFA};

// Cosmogram — tiered-access engine (gates casts by tier ceiling, memory tier,
// access class, and governance metadata)
pub use compiler::{compile_invocations, compile_program, IfaParser, ParseError, ParsedInvocation};
pub use cosmogram::{tier_max_odu, ConsensusLevel, CosmogramEngine, CosmogramState};

// Digital Calabash scaling — 256 base Odù → 65,536 via composition,
// gated by experience and ratified by consensus.
pub use calabash::scaling::{tier_for_xp, AgentExperience, ConsensusLedger};
pub use calabash::{
    cast as cast_scaled, compose_id, decompose, resolve, AccessDenied, ComposedOdu,
};

// Living Manifesto — Odù-backed principles, ratified by consensus.
pub use manifesto::{Clause, Manifesto};

// LARQL — query language over the Digital Calabash (DESCRIBE/VERIFY/PREPARE).
pub use larql::{parse_query as parse_larql, LarqlAnswer, LarqlError, LarqlQuery};
