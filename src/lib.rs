pub mod ase_vault;
pub mod calabash;
pub mod compiler;
pub mod cosmogram;
pub mod ebo;
pub mod entropy;
pub mod error;
pub mod field;
pub mod hermetic;
pub mod manifesto;
pub mod odu;
pub mod orisha;
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

// Full Odù corpus access (Hive/Èṣù tier)
pub use odu::{get_odu, get_odu_by_binary, lookup_by_name, Odu, ODU_SET};

// Cosmogram — ese myth, sacred metadata, hermetic annotations
pub use compiler::{compile_invocations, compile_program, IfaParser, ParseError, ParsedInvocation};
pub use cosmogram::{
    get_cosmogram, tier_max_odu, ConsensusLevel, CosmogramEngine, CosmogramState, OduCosmos,
    COSMOGRAM,
};

// Digital Calabash scaling — 256 base Odù → 65,536 via composition,
// gated by experience and ratified by consensus.
pub use calabash::scaling::{tier_for_xp, AgentExperience, ConsensusLedger};
pub use calabash::{
    cast as cast_scaled, compose_id, decompose, resolve, AccessDenied, ComposedOdu,
};

// Living Manifesto — Odù-backed principles, ratified by consensus.
pub use manifesto::{Clause, Manifesto};
