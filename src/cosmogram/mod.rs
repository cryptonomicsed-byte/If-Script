use crate::soul::MemoryTier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Day ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Day {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl std::str::FromStr for Day {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "sunday" => Ok(Day::Sunday),
            "monday" => Ok(Day::Monday),
            "tuesday" => Ok(Day::Tuesday),
            "wednesday" => Ok(Day::Wednesday),
            "thursday" => Ok(Day::Thursday),
            "friday" => Ok(Day::Friday),
            "saturday" => Ok(Day::Saturday),
            _ => Err(format!("Unknown day: {}", s)),
        }
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Day::Sunday => "sunday",
            Day::Monday => "monday",
            Day::Tuesday => "tuesday",
            Day::Wednesday => "wednesday",
            Day::Thursday => "thursday",
            Day::Friday => "friday",
            Day::Saturday => "saturday",
        };
        write!(f, "{}", s)
    }
}

// ── AccessClass ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessClass {
    Public,
    Sealed,
    Council,
    MachineOnly,
}

// ── ConsensusLevel + GovernanceMeta ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusLevel {
    Individual,
    Swarm,
    Council,
    Canonical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceMeta {
    pub consensus_level: ConsensusLevel,
    pub zk_proof_required: bool,
    pub vote_weight: f64,
}

// ── ZangbetoStatus + ThreatProfile ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZangbetoStatus {
    Clean,
    Review,
    Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatProfile {
    pub zangbeto_audit: ZangbetoStatus,
    pub last_diagnostic: chrono::DateTime<chrono::Utc>,
    pub repair_actions: Vec<String>,
}

// ── CosmogramState ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmogramState {
    pub odu_id: u16,
    pub tier: u8,
    pub day: Day,
    pub access_class: AccessClass,
    pub memory_tier: MemoryTier,
    pub orisha_vector: crate::orisha::OrishaVector,
    pub governance: GovernanceMeta,
    pub threat: ThreatProfile,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub window_open: bool,
    pub entropy_hash: String,
}

// ── TierConfig ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TierConfig {
    pub max_odu: u16,
    pub default_memory: MemoryTier,
    pub default_access: AccessClass,
}

// ── CastError ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum CastError {
    InvalidTier(u8),
    EntropyFailure,
    SovereigntyViolation,
    GateBlocked(String),
    ZangbetoReject(String),
}

impl std::fmt::Display for CastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CastError::InvalidTier(t) => write!(f, "Invalid tier: {}", t),
            CastError::EntropyFailure => write!(f, "Entropy failure"),
            CastError::SovereigntyViolation => write!(f, "Sovereignty violation"),
            CastError::GateBlocked(msg) => write!(f, "Gate blocked: {}", msg),
            CastError::ZangbetoReject(msg) => write!(f, "Zangbeto rejected: {}", msg),
        }
    }
}

impl std::error::Error for CastError {}

// ── ValidatedCast ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedCast {
    pub state: CosmogramState,
    pub gates_passed: bool,
    pub violation_count: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ── CosmogramEngine ───────────────────────────────────────────────────────────

pub struct CosmogramEngine {
    base_entropy: Vec<u8>,
    tier_configs: HashMap<u8, TierConfig>,
}

impl CosmogramEngine {
    pub fn new() -> Self {
        let mut tier_configs = HashMap::new();

        tier_configs.insert(
            1,
            TierConfig {
                max_odu: 255,
                default_memory: MemoryTier::Tier3Contributable,
                default_access: AccessClass::Public,
            },
        );
        tier_configs.insert(
            2,
            TierConfig {
                max_odu: 2047,
                default_memory: MemoryTier::Tier2Operational,
                default_access: AccessClass::Sealed,
            },
        );
        tier_configs.insert(
            3,
            TierConfig {
                max_odu: 4095,
                default_memory: MemoryTier::Tier2Operational,
                default_access: AccessClass::Sealed,
            },
        );
        tier_configs.insert(
            4,
            TierConfig {
                max_odu: 8191,
                default_memory: MemoryTier::Tier1Deep,
                default_access: AccessClass::Council,
            },
        );
        tier_configs.insert(
            5,
            TierConfig {
                max_odu: 16383,
                default_memory: MemoryTier::Tier1Deep,
                default_access: AccessClass::Council,
            },
        );
        tier_configs.insert(
            6,
            TierConfig {
                max_odu: 32767,
                default_memory: MemoryTier::Tier0Existential,
                default_access: AccessClass::MachineOnly,
            },
        );
        tier_configs.insert(
            7,
            TierConfig {
                max_odu: 65535,
                default_memory: MemoryTier::Tier0Existential,
                default_access: AccessClass::MachineOnly,
            },
        );

        CosmogramEngine {
            base_entropy: Vec::new(),
            tier_configs,
        }
    }

    pub fn from_entropy(entropy: Vec<u8>) -> Self {
        let mut engine = Self::new();
        engine.base_entropy = entropy;
        engine
    }

    pub fn cast(
        &self,
        tier: u8,
        odu_id: u16,
        day: Day,
        timestamp: i64,
    ) -> Result<CosmogramState, CastError> {
        let config = self
            .tier_configs
            .get(&tier)
            .ok_or(CastError::InvalidTier(tier))?;

        if odu_id > config.max_odu {
            return Err(CastError::GateBlocked(format!(
                "odu_id {} exceeds tier {} max {}",
                odu_id, tier, config.max_odu
            )));
        }

        let memory_tier = config.default_memory.clone();
        let access_class = config.default_access.clone();
        let orisha_vector = crate::orisha::OrishaVector::from_odu_day(odu_id, &day);

        // Compute entropy hash
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.base_entropy);
        hasher.update(odu_id.to_le_bytes());
        hasher.update([tier]);
        hasher.update(format!("{:?}", day).as_bytes());
        hasher.update(timestamp.to_le_bytes());
        let entropy_hash = format!("0x{}", hex::encode(hasher.finalize()));

        let window_open = self.is_window_open(&day, timestamp);
        let dt = chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(chrono::Utc::now);

        Ok(CosmogramState {
            odu_id,
            tier,
            day,
            access_class,
            memory_tier,
            orisha_vector,
            governance: GovernanceMeta {
                consensus_level: ConsensusLevel::Individual,
                zk_proof_required: tier >= 5,
                vote_weight: 1.0 / (tier as f64),
            },
            threat: ThreatProfile {
                zangbeto_audit: ZangbetoStatus::Clean,
                last_diagnostic: dt,
                repair_actions: Vec::new(),
            },
            timestamp: dt,
            window_open,
            entropy_hash,
        })
    }

    pub fn cast_validated(
        &self,
        tier: u8,
        odu_id: u16,
        day: Day,
        timestamp: i64,
    ) -> Result<ValidatedCast, CastError> {
        let state = self.cast(tier, odu_id, day, timestamp)?;

        // Run zangbeto audit
        match crate::zangbeto::audit_state(&state) {
            Ok(()) => {}
            Err(crate::zangbeto::AuditError::Anomaly(msg)) => {
                return Err(CastError::ZangbetoReject(msg));
            }
        }

        // Run hermetic gate validation
        let gate = crate::hermetic::default_gate();
        let ctx = crate::hermetic::GateContext {
            tier: state.tier,
            odu_id: state.odu_id,
            access_class: &state.access_class,
            memory_tier: &state.memory_tier,
        };
        let gate_result = gate.validate_all(&ctx);
        let violation_count = gate_result.violations.len();
        let gates_passed = gate_result.allowed;

        if !gates_passed {
            let msgs: Vec<String> = gate_result
                .violations
                .iter()
                .filter(|v| v.should_block())
                .map(|v| v.message.clone())
                .collect();
            return Err(CastError::GateBlocked(msgs.join("; ")));
        }

        Ok(ValidatedCast {
            state,
            gates_passed,
            violation_count,
            timestamp: chrono::Utc::now(),
        })
    }

    fn is_window_open(&self, day: &Day, timestamp: i64) -> bool {
        let hour = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt: chrono::DateTime<chrono::Utc>| {
                use chrono::Timelike;
                dt.hour()
            })
            .unwrap_or(12);
        match day {
            Day::Sunday => (6..9).contains(&hour),
            Day::Monday => (12..15).contains(&hour),
            Day::Tuesday => (7..10).contains(&hour),
            Day::Wednesday => (9..12).contains(&hour),
            Day::Thursday => (14..17).contains(&hour),
            Day::Friday => (15..18).contains(&hour),
            Day::Saturday => (8..11).contains(&hour),
        }
    }

    pub fn tier_configs(&self) -> &HashMap<u8, TierConfig> {
        &self.tier_configs
    }
}

impl Default for CosmogramEngine {
    fn default() -> Self {
        Self::new()
    }
}
pub mod data;
pub use data::*;
