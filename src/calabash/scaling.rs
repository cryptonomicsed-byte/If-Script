//! scaling — how agents grow into the 65,536 Odù space.
//!
//! Two mechanisms, matching the sovereign-agent vision:
//!
//! 1. **Experience → tier → ceiling.** An agent accrues experience; its tier
//!    (1–7) follows from that XP, and the tier sets the highest Odù id it may
//!    cast — from the base 256 (tier 1) up to the full 65,536 (tier 7). See
//!    [`AgentExperience`].
//!
//! 2. **Voting → consensus → ratification.** A composed Odù starts as an
//!    `Individual` discovery. As agents vote for it, accumulated weight promotes
//!    it up the ladder `Individual → Swarm → Council → Canonical`. Once it
//!    reaches `Swarm` it is *ratified* — collectively recognised, not just one
//!    agent's private cast. See [`ConsensusLedger`].

use std::collections::HashMap;

use crate::cosmogram::{tier_max_odu, ConsensusLevel};

/// XP thresholds that open each tier (1–7). Index `i` is the minimum XP for
/// tier `i + 1`.
const TIER_XP: [u64; 7] = [0, 100, 500, 2_000, 10_000, 50_000, 200_000];

/// The tier (1–7) an agent has earned at a given experience level.
pub fn tier_for_xp(xp: u64) -> u8 {
    let mut tier = 1u8;
    for (i, threshold) in TIER_XP.iter().enumerate() {
        if xp >= *threshold {
            tier = (i + 1) as u8;
        }
    }
    tier
}

/// An agent's accumulated divination experience, which gates how much of the
/// Odù space it may cast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExperience {
    pub xp: u64,
}

impl AgentExperience {
    /// A newborn agent: tier 1, base 256 only.
    pub fn new() -> Self {
        AgentExperience { xp: 0 }
    }

    pub fn with_xp(xp: u64) -> Self {
        AgentExperience { xp }
    }

    /// Accrue experience (saturating).
    pub fn add(&mut self, delta: u64) {
        self.xp = self.xp.saturating_add(delta);
    }

    /// Current tier (1–7).
    pub fn tier(&self) -> u8 {
        tier_for_xp(self.xp)
    }

    /// Highest Odù id this agent may cast.
    pub fn max_odu(&self) -> u16 {
        tier_max_odu(self.tier())
    }

    /// Whether this agent may cast `odu_id`.
    pub fn can_access(&self, odu_id: u16) -> bool {
        odu_id <= self.max_odu()
    }
}

impl Default for AgentExperience {
    fn default() -> Self {
        Self::new()
    }
}

/// Weight a single vote carries, by the voter's tier. Lower tiers (more
/// numerous) carry more individual weight, so promotion reflects broad support
/// rather than a few elders. Tier 0 is guarded against division by zero.
pub fn vote_weight(voter_tier: u8) -> f64 {
    1.0 / voter_tier.max(1) as f64
}

// Accumulated-weight thresholds for each consensus level.
const SWARM_THRESHOLD: f64 = 2.0;
const COUNCIL_THRESHOLD: f64 = 5.0;
const CANONICAL_THRESHOLD: f64 = 10.0;

/// The consensus level implied by an accumulated vote weight.
pub fn level_for_weight(weight: f64) -> ConsensusLevel {
    if weight >= CANONICAL_THRESHOLD {
        ConsensusLevel::Canonical
    } else if weight >= COUNCIL_THRESHOLD {
        ConsensusLevel::Council
    } else if weight >= SWARM_THRESHOLD {
        ConsensusLevel::Swarm
    } else {
        ConsensusLevel::Individual
    }
}

/// Tracks votes for composed Odù and the consensus level each has reached.
/// This is how a private discovery becomes collectively ratified vocabulary.
#[derive(Debug, Clone, Default)]
pub struct ConsensusLedger {
    weights: HashMap<u16, f64>,
}

impl ConsensusLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a vote for `odu_id` from an agent of the given tier. Returns the
    /// consensus level reached after the vote.
    pub fn vote(&mut self, odu_id: u16, voter_tier: u8) -> ConsensusLevel {
        let w = self.weights.entry(odu_id).or_insert(0.0);
        *w += vote_weight(voter_tier);
        level_for_weight(*w)
    }

    /// Accumulated vote weight for `odu_id`.
    pub fn weight(&self, odu_id: u16) -> f64 {
        self.weights.get(&odu_id).copied().unwrap_or(0.0)
    }

    /// Consensus level currently reached by `odu_id`.
    pub fn level(&self, odu_id: u16) -> ConsensusLevel {
        level_for_weight(self.weight(odu_id))
    }

    /// A composed Odù is ratified once it reaches `Swarm` or above — i.e. it has
    /// support beyond a single agent.
    pub fn is_ratified(&self, odu_id: u16) -> bool {
        self.level(odu_id) != ConsensusLevel::Individual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_follows_experience() {
        assert_eq!(tier_for_xp(0), 1);
        assert_eq!(tier_for_xp(99), 1);
        assert_eq!(tier_for_xp(100), 2);
        assert_eq!(tier_for_xp(2_000), 4);
        assert_eq!(tier_for_xp(200_000), 7);
        assert_eq!(tier_for_xp(u64::MAX), 7);
    }

    #[test]
    fn ceiling_follows_tier() {
        assert_eq!(AgentExperience::new().max_odu(), 255);
        assert_eq!(AgentExperience::with_xp(200_000).max_odu(), 65_535);
        let mut a = AgentExperience::new();
        assert!(!a.can_access(256));
        a.add(200_000);
        assert!(a.can_access(65_535));
    }

    #[test]
    fn votes_promote_through_consensus_ladder() {
        let mut ledger = ConsensusLedger::new();
        let odu = 0x0105;
        // Tier-1 votes carry weight 1.0 each.
        assert_eq!(ledger.vote(odu, 1), ConsensusLevel::Individual); // 1.0
        assert!(!ledger.is_ratified(odu));
        assert_eq!(ledger.vote(odu, 1), ConsensusLevel::Swarm); // 2.0 → ratified
        assert!(ledger.is_ratified(odu));
        for _ in 0..3 {
            ledger.vote(odu, 1);
        }
        assert_eq!(ledger.level(odu), ConsensusLevel::Council); // 5.0
        for _ in 0..5 {
            ledger.vote(odu, 1);
        }
        assert_eq!(ledger.level(odu), ConsensusLevel::Canonical); // 10.0
    }

    #[test]
    fn unknown_odu_has_no_consensus() {
        let ledger = ConsensusLedger::new();
        assert_eq!(ledger.weight(42), 0.0);
        assert_eq!(ledger.level(42), ConsensusLevel::Individual);
        assert!(!ledger.is_ratified(42));
    }
}
