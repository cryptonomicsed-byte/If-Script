use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EboTrigger {
    StackUnderflow,
    /// Defined for future arithmetic opcodes; not yet triggered.
    DivisionByZero,
    /// Defined for type-mismatch conditions; not yet triggered.
    InvalidCast,
    HeapOverflow,
    ForbiddenBranch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ebo {
    TimeDelay(Duration),
    ProofOfWork(u32),
    TokenBurn(String),
    /// A vow that must meet the minimum-length and keyword requirements checked
    /// by `EboTrigger::accepts()`. The string field is the *commitment hash*
    /// printed for auditability, not the raw vow text.
    IntentionString(String),
}

pub struct EboHistory {
    counts: HashMap<EboTrigger, u32>,
}

impl EboHistory {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn record(&mut self, trigger: EboTrigger) {
        *self.counts.entry(trigger).or_insert(0) += 1;
    }

    pub fn required_ebo(&self, trigger: &EboTrigger) -> Ebo {
        let count = self.counts.get(trigger).copied().unwrap_or(0);
        match (trigger, count) {
            (EboTrigger::StackUnderflow, 0..=2) => Ebo::TimeDelay(Duration::from_secs(1)),
            (EboTrigger::StackUnderflow, _) => Ebo::ProofOfWork(20),
            (EboTrigger::DivisionByZero, _) => Ebo::ProofOfWork(20),
            (EboTrigger::ForbiddenBranch, _) => {
                Ebo::IntentionString("I vow clarity and no harm".to_string())
            }
            _ => Ebo::TimeDelay(Duration::from_secs(5)),
        }
    }

    pub fn has_trigger(&self, trigger: &EboTrigger) -> bool {
        self.counts.contains_key(trigger)
    }
}

impl EboTrigger {
    /// Validate an Ebo offering for this trigger type.
    ///
    /// For `ForbiddenBranch`, the `IntentionString` vow must:
    /// 1. Be at least 20 characters long.
    /// 2. Contain at least one of the mandatory phrases: "i vow", "i pledge", "i commit".
    /// 3. Contain at least one of the ethical terms: "clarity", "no harm", "transparency".
    ///
    /// This prevents trivial bypass via incidental word matches (e.g. "no harm in trying").
    pub fn accepts(&self, ebo: &Ebo) -> bool {
        match (self, ebo) {
            (EboTrigger::StackUnderflow, Ebo::TimeDelay(d)) => d.as_secs() >= 1,
            (EboTrigger::DivisionByZero, Ebo::ProofOfWork(diff)) => *diff >= 20,
            (EboTrigger::ForbiddenBranch, Ebo::IntentionString(s)) => vow_is_valid(s),
            _ => false,
        }
    }
}

/// Validate a ForbiddenBranch vow string.
///
/// A valid vow must be substantive (≥ 20 chars), contain a commitment marker,
/// and contain an ethical term. This prevents incidental substring matches.
pub fn vow_is_valid(vow: &str) -> bool {
    if vow.len() < 20 {
        return false;
    }
    let lower = vow.to_lowercase();
    let has_commitment =
        lower.contains("i vow") || lower.contains("i pledge") || lower.contains("i commit");
    let has_ethical_term =
        lower.contains("clarity") || lower.contains("no harm") || lower.contains("transparency");
    has_commitment && has_ethical_term
}

/// Produce a SHA-256 commitment hash over the vow text for auditability.
pub fn commitment_hash(vow: &str) -> String {
    let hash = Sha256::digest(vow.as_bytes());
    hex::encode(&hash[..8])
}

impl Default for EboHistory {
    fn default() -> Self {
        Self::new()
    }
}
