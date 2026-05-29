use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    Tier0Existential,
    Tier1Deep,
    Tier2Operational,
    Tier3Contributable,
}
