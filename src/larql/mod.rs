//! LARQL — a query language for reading the Digital Calabash / Òdù Ifá corpus.
//!
//! ## Scope (honest)
//!
//! This is a bounded v0.1, not the full speculative design in
//! `docs/consolidated/larql.md` (which was drafted against a fictional
//! `ActionVessel::Alignment` variant and an invented `confidence_baseline`/
//! `sensitivity_level` metadata schema that doesn't exist anywhere in this
//! crate — implementing it verbatim would have meant building a second,
//! disconnected corpus schema, the exact problem this project just spent
//! effort removing).
//!
//! Three query types, all grounded in the real corpus (`crate::odu`,
//! `crate::odu::ActionVessel`) with zero invented fields:
//!
//! - `DESCRIBE <index|"name"> [AT SCALE micro,meso,macro]` — look up an Odù
//!   and render it at increasing levels of detail (`micro`: universal name +
//!   archetype; `meso`: + prescriptions; `macro`: + taboos, archetypes,
//!   interpretation type).
//! - `VERIFY <Vessel> [WHERE <field> <op> <value>]` — check whether any Odù
//!   under a vessel matches a condition (`field` ∈ `universal_name`,
//!   `archetype`, `archetypes`, `prescription`, `name`; `op` ∈ `=`,
//!   `CONTAINS`).
//! - `PREPARE <action> CHECK: <Vessel>` — the vessel's file-domain-grounded
//!   step list for logging an action (write intent, log outcome).
//!
//! Deferred (not in this v0.1): `WALK` (time-series aggregation — no time
//! series data model exists yet) and `SYNTHESIZE`'s corpus-metadata
//! filtering (`confidence_baseline`, `sensitivity_level`, `larql_tags` —
//! fields the real `Odu` struct doesn't have and that 512 corpus entries
//! don't carry). Both are real follow-up work, not silently dropped.

pub mod ast;
pub mod engine;
pub mod error;
pub mod parser;

pub use ast::{
    Condition, DescribeQuery, LarqlQuery, OduRef, Operator, PrepareQuery, Scale, VerifyQuery,
};
pub use engine::{execute, LarqlAnswer};
pub use error::LarqlError;
pub use parser::parse_query;
