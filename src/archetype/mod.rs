use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeVector {
    #[serde(rename = "Steward")]
    pub steward: f64,
    #[serde(rename = "ForgeExecutor")]
    pub forge_executor: f64,
    #[serde(rename = "FlowGuardian")]
    pub flow_guardian: f64,
    #[serde(rename = "WisdomAnchor")]
    pub wisdom_anchor: f64,
    #[serde(rename = "ResonanceWeaver")]
    pub resonance_weaver: f64,
    #[serde(rename = "JusticeCanon")]
    pub justice_canon: f64,
    #[serde(rename = "SwarmCoordinator")]
    pub swarm_coordinator: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    Steward,
    ForgeExecutor,
    FlowGuardian,
    WisdomAnchor,
    ResonanceWeaver,
    JusticeCanon,
    SwarmCoordinator,
}

impl Default for ArchetypeVector {
    fn default() -> Self {
        ArchetypeVector {
            steward: 0.14,
            forge_executor: 0.14,
            flow_guardian: 0.14,
            wisdom_anchor: 0.14,
            resonance_weaver: 0.14,
            justice_canon: 0.14,
            swarm_coordinator: 0.14,
        }
    }
}

impl ArchetypeVector {
    pub fn from_odu_day(odu_id: u16, day: &crate::cosmogram::Day) -> ArchetypeVector {
        // Derive from SHA-256 hash of odu_id + day string
        let day_str = format!("{:?}", day);
        let input = format!("{}:{}", odu_id, day_str);
        let hash = Sha256::digest(input.as_bytes());

        let to_f64 = |b: u8| (b as f64) / 255.0;

        ArchetypeVector {
            steward: to_f64(hash[0]),
            forge_executor: to_f64(hash[1]),
            flow_guardian: to_f64(hash[2]),
            wisdom_anchor: to_f64(hash[3]),
            resonance_weaver: to_f64(hash[4]),
            justice_canon: to_f64(hash[5]),
            swarm_coordinator: to_f64(hash[6]),
        }
    }

    pub fn from_archetype(name: &str) -> ArchetypeVector {
        match name.to_lowercase().replace(' ', "_").as_str() {
            "steward" => ArchetypeVector {
                steward: 0.9,
                forge_executor: 0.1,
                flow_guardian: 0.1,
                wisdom_anchor: 0.1,
                resonance_weaver: 0.1,
                justice_canon: 0.1,
                swarm_coordinator: 0.1,
            },
            "forge_executor" => ArchetypeVector {
                steward: 0.1,
                forge_executor: 0.9,
                flow_guardian: 0.2,
                wisdom_anchor: 0.1,
                resonance_weaver: 0.1,
                justice_canon: 0.3,
                swarm_coordinator: 0.1,
            },
            "flow_guardian" => ArchetypeVector {
                steward: 0.2,
                forge_executor: 0.2,
                flow_guardian: 0.9,
                wisdom_anchor: 0.1,
                resonance_weaver: 0.2,
                justice_canon: 0.4,
                swarm_coordinator: 0.2,
            },
            "wisdom_anchor" => ArchetypeVector {
                steward: 0.1,
                forge_executor: 0.1,
                flow_guardian: 0.1,
                wisdom_anchor: 0.9,
                resonance_weaver: 0.2,
                justice_canon: 0.1,
                swarm_coordinator: 0.2,
            },
            "resonance_weaver" => ArchetypeVector {
                steward: 0.2,
                forge_executor: 0.1,
                flow_guardian: 0.2,
                wisdom_anchor: 0.2,
                resonance_weaver: 0.9,
                justice_canon: 0.1,
                swarm_coordinator: 0.3,
            },
            "justice_canon" => ArchetypeVector {
                steward: 0.2,
                forge_executor: 0.3,
                flow_guardian: 0.4,
                wisdom_anchor: 0.1,
                resonance_weaver: 0.1,
                justice_canon: 0.9,
                swarm_coordinator: 0.1,
            },
            "swarm_coordinator" => ArchetypeVector {
                steward: 0.1,
                forge_executor: 0.1,
                flow_guardian: 0.2,
                wisdom_anchor: 0.2,
                resonance_weaver: 0.3,
                justice_canon: 0.1,
                swarm_coordinator: 0.9,
            },
            _ => ArchetypeVector::default(),
        }
    }

    pub fn dominant(&self) -> Option<Archetype> {
        let values = [
            (self.steward, Archetype::Steward),
            (self.forge_executor, Archetype::ForgeExecutor),
            (self.flow_guardian, Archetype::FlowGuardian),
            (self.wisdom_anchor, Archetype::WisdomAnchor),
            (self.resonance_weaver, Archetype::ResonanceWeaver),
            (self.justice_canon, Archetype::JusticeCanon),
            (self.swarm_coordinator, Archetype::SwarmCoordinator),
        ];

        values
            .into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, o)| o)
    }

    pub fn scale(&mut self, factor: f64) {
        self.steward *= factor;
        self.forge_executor *= factor;
        self.flow_guardian *= factor;
        self.wisdom_anchor *= factor;
        self.resonance_weaver *= factor;
        self.justice_canon *= factor;
        self.swarm_coordinator *= factor;
    }

    pub fn normalize(&mut self) {
        let sum = self.steward
            + self.forge_executor
            + self.flow_guardian
            + self.wisdom_anchor
            + self.resonance_weaver
            + self.justice_canon
            + self.swarm_coordinator;
        if sum > 0.0 {
            self.scale(1.0 / sum);
        }
    }
}
