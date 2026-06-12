use crate::cosmogram::Day;
use crate::hermetic::HermeticPrinciple;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod julia_bridge;

// ── ResonancePacket ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonancePacket {
    pub odu_id: u16,
    pub tier: u8,
    pub day: Day,
    pub timestamp: i64,
    /// Gate bias: HermeticPrinciple key → weight (stored as string keys for JSON compat)
    pub gate_bias_raw: HashMap<String, f64>,
    pub intent: String,
    pub archetype: Option<String>,
}

impl ResonancePacket {
    pub fn new(odu_id: u16, tier: u8, day: Day, timestamp: i64, intent: impl Into<String>) -> Self {
        ResonancePacket {
            odu_id,
            tier,
            day,
            timestamp,
            gate_bias_raw: HashMap::new(),
            intent: intent.into(),
            archetype: None,
        }
    }

    /// Get the gate bias as typed HermeticPrinciple keys
    pub fn gate_bias(&self) -> HashMap<HermeticPrinciple, f64> {
        self.gate_bias_raw
            .iter()
            .filter_map(|(k, v)| k.parse::<HermeticPrinciple>().ok().map(|p| (p, *v)))
            .collect()
    }

    pub fn set_gate_bias(&mut self, principle: HermeticPrinciple, weight: f64) {
        self.gate_bias_raw.insert(principle.to_string(), weight);
    }
}

// ── ResonanceReceipt ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceReceipt {
    pub packet: ResonancePacket,
    pub entropy_hash: String,
    pub orisha_dominant: Option<String>,
    pub gates_passed: bool,
    pub violation_count: usize,
    pub receipt_hash: Option<String>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

impl ResonanceReceipt {
    pub fn new(
        packet: ResonancePacket,
        entropy_hash: String,
        orisha_dominant: Option<String>,
        gates_passed: bool,
        violation_count: usize,
    ) -> Self {
        ResonanceReceipt {
            packet,
            entropy_hash,
            orisha_dominant,
            gates_passed,
            violation_count,
            receipt_hash: None,
            issued_at: chrono::Utc::now(),
        }
    }
}

// ── RitualCodex ───────────────────────────────────────────────────────────────

pub struct RitualCodex {
    pub data_dir: Option<std::path::PathBuf>,
}

impl RitualCodex {
    pub fn new() -> Self {
        RitualCodex { data_dir: None }
    }

    pub fn with_data_dir(path: impl Into<std::path::PathBuf>) -> Self {
        RitualCodex {
            data_dir: Some(path.into()),
        }
    }

    /// Load day configuration from JSON file, if available
    pub fn load_day(&self, day: &Day) -> Option<serde_json::Value> {
        let data_dir = self.data_dir.as_ref()?;
        let filename = format!("{}.json", day);
        let path = data_dir.join(filename);
        let contents = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Cast a resonance packet into a receipt using the cosmogram engine
    pub fn cast_resonance(
        &self,
        packet: ResonancePacket,
        engine: &crate::cosmogram::CosmogramEngine,
    ) -> Result<ResonanceReceipt, crate::cosmogram::CastError> {
        let validated = engine.cast_validated(
            packet.tier,
            packet.odu_id,
            packet.day.clone(),
            packet.timestamp,
        )?;

        let orisha_dominant = crate::orisha::OrishaVector::from_odu_day(packet.odu_id, &packet.day)
            .dominant()
            .map(|o| format!("{:?}", o));

        let receipt = ResonanceReceipt::new(
            packet,
            validated.state.entropy_hash.clone(),
            orisha_dominant,
            validated.gates_passed,
            validated.violation_count,
        );

        Ok(receipt)
    }
}

impl Default for RitualCodex {
    fn default() -> Self {
        Self::new()
    }
}
