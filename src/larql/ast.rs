//! LARQL — a small query language for reading the Digital Calabash / Òdù Ifá
//! corpus. Bounded, honest scope (see `src/larql/mod.rs` for what's in and
//! what's deferred): three query types, all resolved against the real
//! `crate::odu`/`crate::odu_ifa` corpora and `crate::odu::ActionVessel` —
//! no invented metadata schema.

use crate::odu::ActionVessel;

#[derive(Debug, Clone, PartialEq)]
pub enum LarqlQuery {
    Describe(DescribeQuery),
    Verify(VerifyQuery),
    Prepare(PrepareQuery),
}

/// Level of detail requested from a `DESCRIBE` query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Universal name + archetype — a one-line gloss.
    Micro,
    /// Micro + prescriptions.
    Meso,
    /// Meso + taboos, archetypes/Òrìṣà, and interpretation type.
    Macro,
}

/// Which Odù a `DESCRIBE` targets.
#[derive(Debug, Clone, PartialEq)]
pub enum OduRef {
    Index(u8),
    Name(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DescribeQuery {
    pub target: OduRef,
    /// Defaults to `[Scale::Micro]` when the query omits `AT SCALE ...`.
    pub scales: Vec<Scale>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Eq,
    Contains,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    /// One of: `universal_name`, `archetype`, `archetypes`, `prescription`.
    pub field: String,
    pub operator: Operator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifyQuery {
    pub vessel: ActionVessel,
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepareQuery {
    pub action: String,
    pub vessel: ActionVessel,
}

/// Parse a bare vessel name (`"Genesis"`, `"Consent"`, ...) into `ActionVessel`.
pub fn parse_vessel(name: &str) -> Option<ActionVessel> {
    Some(match name {
        "Genesis" => ActionVessel::Genesis,
        "Void" => ActionVessel::Void,
        "Attention" => ActionVessel::Attention,
        "Loop" => ActionVessel::Loop,
        "Receipt" => ActionVessel::Receipt,
        "Mask" => ActionVessel::Mask,
        "Residue" => ActionVessel::Residue,
        "Execution" => ActionVessel::Execution,
        "Swarm" => ActionVessel::Swarm,
        "Restraint" => ActionVessel::Restraint,
        "Migration" => ActionVessel::Migration,
        "Consent" => ActionVessel::Consent,
        "Vision" => ActionVessel::Vision,
        "Growth" => ActionVessel::Growth,
        "Seal" => ActionVessel::Seal,
        "Rhythm" => ActionVessel::Rhythm,
        _ => return None,
    })
}
