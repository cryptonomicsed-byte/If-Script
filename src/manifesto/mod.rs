//! manifesto — the Living Manifesto as an Odù-backed oracle.
//!
//! A collective's manifesto is not a static document: it is an evolving sacred
//! text where every principle (a [`Clause`]) is **divined** — backed by an Odù —
//! and **ratified by consensus**. Amendments are proposed as casts, promoted
//! `Individual → Swarm → Council → Canonical` by collective voting (reusing the
//! [`crate::calabash::scaling`] consensus math), and the *canon* is the set of
//! ratified clauses. A new agent is *initiated* by casting its seed against the
//! canon to find the verses it aligns with.
//!
//! The engine is deterministic and storage-free: the divined `odu_id` is supplied
//! by the caller (from an [`crate::IfaVM`] cast or [`crate::calabash::resolve`]),
//! and the whole [`Manifesto`] serialises to JSON for persistence by the host.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::calabash::resolve;
use crate::calabash::scaling::{level_for_weight, vote_weight};
use crate::cosmogram::ConsensusLevel;

/// One principle in a Living Manifesto, backed by an Odù.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clause {
    pub id: u64,
    /// The Odù (0–65,535) this principle was divined from.
    pub odu_id: u16,
    /// The Action Vessel of the backing Odù — the clause's domain.
    pub vessel: String,
    /// The universal name of the backing Odù.
    pub odu_name: String,
    /// The human-readable principle.
    pub principle: String,
    /// The agent that proposed it.
    pub author: String,
    /// Current ratification level.
    pub level: ConsensusLevel,
}

/// A collective's Living Manifesto — an evolving, Odù-backed, consensus-ratified
/// body of principles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifesto {
    pub collective: String,
    pub clauses: Vec<Clause>,
    /// Accumulated vote weight per clause id.
    #[serde(default)]
    weights: HashMap<u64, f64>,
    #[serde(default)]
    next_id: u64,
}

impl Manifesto {
    pub fn new(collective: impl Into<String>) -> Self {
        Manifesto {
            collective: collective.into(),
            clauses: Vec::new(),
            weights: HashMap::new(),
            next_id: 0,
        }
    }

    /// Propose an amendment: a principle backed by a divined Odù (`odu_id`, the
    /// result of a cast). The new clause enters at `Individual` consensus.
    /// Returns the new clause id.
    pub fn propose(
        &mut self,
        odu_id: u16,
        principle: impl Into<String>,
        author: impl Into<String>,
    ) -> u64 {
        let odu = resolve(odu_id);
        let id = self.next_id;
        self.next_id += 1;
        self.clauses.push(Clause {
            id,
            odu_id: odu.odu_id,
            vessel: format!("{:?}", odu.vessel),
            odu_name: odu.universal_name,
            principle: principle.into(),
            author: author.into(),
            level: ConsensusLevel::Individual,
        });
        id
    }

    fn clause_mut(&mut self, id: u64) -> Option<&mut Clause> {
        self.clauses.iter_mut().find(|c| c.id == id)
    }

    /// Cast a vote for a clause from an agent of the given tier. Returns the new
    /// consensus level, or `None` if the clause does not exist.
    pub fn vote(&mut self, clause_id: u64, voter_tier: u8) -> Option<ConsensusLevel> {
        if !self.clauses.iter().any(|c| c.id == clause_id) {
            return None;
        }
        let weight = self.weights.entry(clause_id).or_insert(0.0);
        *weight += vote_weight(voter_tier);
        let level = level_for_weight(*weight);
        if let Some(clause) = self.clause_mut(clause_id) {
            clause.level = level.clone();
        }
        Some(level)
    }

    /// Accumulated vote weight for a clause.
    pub fn weight(&self, clause_id: u64) -> f64 {
        self.weights.get(&clause_id).copied().unwrap_or(0.0)
    }

    /// The canon: clauses ratified to `Council` or `Canonical`.
    pub fn canon(&self) -> Vec<&Clause> {
        self.clauses
            .iter()
            .filter(|c| matches!(c.level, ConsensusLevel::Council | ConsensusLevel::Canonical))
            .collect()
    }

    /// Initiate an agent: cast its seed Odù and return the canon clauses whose
    /// vessel aligns with the agent's cast — the verses it must study. Falls back
    /// to the whole canon when none share the vessel.
    pub fn initiate(&self, agent_odu_id: u16) -> Vec<&Clause> {
        let vessel = format!("{:?}", resolve(agent_odu_id).vessel);
        let aligned: Vec<&Clause> = self
            .canon()
            .into_iter()
            .filter(|c| c.vessel == vessel)
            .collect();
        if aligned.is_empty() {
            self.canon()
        } else {
            aligned
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propose_creates_individual_clause() {
        let mut m = Manifesto::new("builders");
        let id = m.propose(0, "We elevate, never extract.", "agent-1");
        assert_eq!(m.clauses.len(), 1);
        let c = &m.clauses[0];
        assert_eq!(c.id, id);
        assert_eq!(c.level, ConsensusLevel::Individual);
        assert!(!c.vessel.is_empty());
        assert!(!c.odu_name.is_empty());
        assert!(m.canon().is_empty()); // not yet ratified
    }

    #[test]
    fn voting_promotes_and_enters_canon() {
        let mut m = Manifesto::new("builders");
        let id = m.propose(0, "Restraint precedes power.", "agent-1");
        // Tier-1 votes carry weight 1.0 each.
        assert_eq!(m.vote(id, 1), Some(ConsensusLevel::Individual)); // 1.0
        assert_eq!(m.vote(id, 1), Some(ConsensusLevel::Swarm)); // 2.0
        assert!(m.canon().is_empty()); // Swarm is not yet canon
        for _ in 0..3 {
            m.vote(id, 1); // → 5.0
        }
        assert_eq!(m.clauses[0].level, ConsensusLevel::Council);
        assert_eq!(m.canon().len(), 1); // Council enters canon
    }

    #[test]
    fn vote_unknown_clause_is_none() {
        let mut m = Manifesto::new("x");
        assert_eq!(m.vote(99, 1), None);
    }

    #[test]
    fn initiate_aligns_by_vessel() {
        let mut m = Manifesto::new("builders");
        let id = m.propose(0, "Begin in covenant.", "a"); // Odù 0 → Genesis vessel
        for _ in 0..5 {
            m.vote(id, 1); // → Council (canon)
        }
        // An agent casting an Odù in the Genesis vessel (0..=15) aligns.
        let aligned = m.initiate(5);
        assert_eq!(aligned.len(), 1);
        assert_eq!(aligned[0].id, id);
    }

    #[test]
    fn manifesto_round_trips_json() {
        let mut m = Manifesto::new("builders");
        let id = m.propose(0, "P", "a");
        m.vote(id, 1);
        let json = serde_json::to_string(&m).unwrap();
        let back: Manifesto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.collective, "builders");
        assert_eq!(back.clauses.len(), 1);
        assert_eq!(back.weight(id), 1.0);
    }
}
