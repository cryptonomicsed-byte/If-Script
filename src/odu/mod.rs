//! odu — 256 Odù Semantic Layer for IfáScript Ω
//!
//! The Odù is not the action. The Odù is the interpretation of reality.
//! A small, composable VM opcode set carries 256 sacred meanings.
//!
//! Canonical sources:
//!   Wande Abimbola, Ifá Divination Poetry (1977)
//!   William Bascom, Ifá Divination (1969)
//!
//! Synthetic enrichment (interpretation_type = "synthetic"):
//!   ÒRÀCÙLÙM corpus — archetypal metadata, taboos, prescriptions.
//!   Not sourced from ese Ifá. Clearly marked. Never conflate with canon.
//!
//! Opcode mapping principle (top vessel drives the opcode):
//!   Genesis    (0000) → PushConst1   — Creation, light, initialization
//!   Void       (0001) → PopVoid       — Clearing, dissolution, release
//!   Attention  (0010) → Dup           — Mirror, reflection, doubling
//!   Loop       (0011) → Swap          — Reversal, inversion, pattern
//!   Receipt    (0100) → Add           — Union, accumulation, synthesis
//!   Mask       (0101) → Sub           — Separation, split, trickster
//!   Residue    (0110) → PushConst0    — Ground, rest, baseline
//!   Execution  (0111) → CastCowries  — Volatility, the throw itself
//!   Swarm      (1000) → CastCowries   — Clearing path, coordination, labor
//!   Restraint  (1001) → Sub           — Flight, storm, sudden change
//!   Migration  (1010) → Swap          — Coil, constriction, portability
//!   Consent    (1011) → HaltIfOne     — Burden, gestation, pause
//!   Vision     (1100) → PushConst1    — Vision, direction, truth
//!   Growth     (1101) → Dup          — Pressing down, seeding, expansion
//!   Seal       (1110) → Add           — Abundance, sweetness, sealing
//!   Rhythm     (1111) → HaltIfOne     — Cosmic completion, unity, cadence
//!
//! The corpus lives in `waves/wave01.rs` … `waves/wave16.rs` — one file per
//! wave (top nibble), 16 entries each. `ODU_SET` is assembled from those
//! files at compile time, and a const block verifies index/binary/vessel
//! consistency for every entry: any corpus edit that breaks an invariant
//! fails the build.

pub mod waves;

use crate::vm::OduOp;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Copy-safe opcode tag for static array context.
/// The bottom Odù modifies meaning, not operation.
/// Convert to executable OduOp via `.to_op()`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OduOpCode {
    PushConst1,
    PushConst0,
    PopVoid,
    Dup,
    Swap,
    Add,
    Sub,
    HaltIfOne,
    CastCowries,
}

impl OduOpCode {
    pub fn to_op(self) -> OduOp {
        match self {
            OduOpCode::PushConst1 => OduOp::PushConst(1),
            OduOpCode::PushConst0 => OduOp::PushConst(0),
            OduOpCode::PopVoid => OduOp::PopVoid,
            OduOpCode::Dup => OduOp::Dup,
            OduOpCode::Swap => OduOp::Swap,
            OduOpCode::Add => OduOp::Add,
            OduOpCode::Sub => OduOp::Sub,
            OduOpCode::HaltIfOne => OduOp::HaltIfOne,
            OduOpCode::CastCowries => OduOp::CastCowries,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Odu {
    pub index: u8,
    pub binary: u8,
    pub name: &'static str,
    pub universal_name: &'static str,
    pub archetype: &'static str,
    pub description: &'static str,
    pub taboos: &'static [&'static str],
    pub prescriptions: &'static [&'static str],
    pub archetypes: &'static [&'static str],
    pub interpretation_type: &'static str, // "canonical" | "synthetic"
    pub vessel: ActionVessel,
    pub opcode: OduOpCode,
}

/// The 16 Action Vessels — operational domain for each Odù wave.
/// Determined by the top 4 bits of the Odù index (wave).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionVessel {
    Genesis,   // Wave  1 — Initialize, covenant
    Void,      // Wave  2 — Clear, release
    Attention, // Wave  3 — Focus, signal/noise
    Loop,      // Wave  4 — Pattern, iteration
    Receipt,   // Wave  5 — Record, accountability
    Mask,      // Wave  6 — Public/private split
    Residue,   // Wave  7 — Behavioral echoes
    Execution, // Wave  8 — Precision action
    Swarm,     // Wave  9 — Collective coordination
    Restraint, // Wave 10 — Ethical limits
    Migration, // Wave 11 — Portability, identity
    Consent,   // Wave 12 — Human approval
    Vision,    // Wave 13 — Direction, horizon
    Growth,    // Wave 14 — Fractal expansion
    Seal,      // Wave 15 — Sacred privacy
    Rhythm,    // Wave 16 — Ritual cadence
}

impl ActionVessel {
    pub fn from_index(index: u8) -> Self {
        match index >> 4 {
            0 => ActionVessel::Genesis,
            1 => ActionVessel::Void,
            2 => ActionVessel::Attention,
            3 => ActionVessel::Loop,
            4 => ActionVessel::Receipt,
            5 => ActionVessel::Mask,
            6 => ActionVessel::Residue,
            7 => ActionVessel::Execution,
            8 => ActionVessel::Swarm,
            9 => ActionVessel::Restraint,
            10 => ActionVessel::Migration,
            11 => ActionVessel::Consent,
            12 => ActionVessel::Vision,
            13 => ActionVessel::Growth,
            14 => ActionVessel::Seal,
            _ => ActionVessel::Rhythm,
        }
    }

    pub fn file_domain(self) -> &'static str {
        match self {
            ActionVessel::Genesis => "genesis.md",
            ActionVessel::Void => "void_log.md",
            ActionVessel::Attention => "attention_audit.md",
            ActionVessel::Loop => "loops.md",
            ActionVessel::Receipt => "receipt_ledger.md",
            ActionVessel::Mask => "soul.md",
            ActionVessel::Residue => "memory_residue.md",
            ActionVessel::Execution => "execution_plan.md",
            ActionVessel::Swarm => "swarm_charter.md",
            ActionVessel::Restraint => "restraint_log.md",
            ActionVessel::Migration => "migration_plan.md",
            ActionVessel::Consent => "consent_log.md",
            ActionVessel::Vision => "vision.md",
            ActionVessel::Growth => "fractal_log.md",
            ActionVessel::Seal => "seal/",
            ActionVessel::Rhythm => "rhythm_codex.md",
        }
    }
}

/// Lookup by index (0–255). Panics on out-of-range — never call with unchecked input.
pub fn get_odu(index: u8) -> &'static Odu {
    &ODU_SET[index as usize]
}

/// Lookup by 8-bit binary value.
pub fn get_odu_by_binary(binary: u8) -> &'static Odu {
    &ODU_SET[binary as usize]
}

/// Search by Yorùbá compound name or universal English name.
/// Returns `None` if no entry matches. O(1) via a lazily-built name index.
pub fn lookup_by_name(name: &str) -> Option<&'static Odu> {
    NAME_INDEX.get(name).copied()
}

/// The full 256-entry corpus, assembled from the 16 wave files at compile time.
pub static ODU_SET: [Odu; 256] = assemble();

const fn assemble() -> [Odu; 256] {
    let wave_refs: [&[Odu; 16]; 16] = [
        &waves::wave01::WAVE,
        &waves::wave02::WAVE,
        &waves::wave03::WAVE,
        &waves::wave04::WAVE,
        &waves::wave05::WAVE,
        &waves::wave06::WAVE,
        &waves::wave07::WAVE,
        &waves::wave08::WAVE,
        &waves::wave09::WAVE,
        &waves::wave10::WAVE,
        &waves::wave11::WAVE,
        &waves::wave12::WAVE,
        &waves::wave13::WAVE,
        &waves::wave14::WAVE,
        &waves::wave15::WAVE,
        &waves::wave16::WAVE,
    ];
    let mut out = [wave_refs[0][0]; 256];
    let mut i = 0;
    while i < 256 {
        out[i] = wave_refs[i >> 4][i & 0x0F];
        i += 1;
    }
    out
}

// Compile-time completeness check: every entry's index and binary must match
// its array position, and its vessel must match the wave (top nibble).
// A corpus edit that violates any invariant is a build error, not a runtime bug.
const _: () = {
    let mut i = 0;
    while i < 256 {
        assert!(
            ODU_SET[i].index == i as u8,
            "Odu index must match array position"
        );
        assert!(
            ODU_SET[i].binary == i as u8,
            "Odu binary must match its index"
        );
        assert!(
            ODU_SET[i].vessel as u8 == (i >> 4) as u8,
            "Odu vessel must match wave (top nibble of index)"
        );
        i += 1;
    }
};

/// O(1) name → Odù index for `lookup_by_name`, keyed on both the domain-pair
/// name and the universal English name.
static NAME_INDEX: LazyLock<HashMap<&'static str, &'static Odu>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(512);
    for odu in ODU_SET.iter() {
        m.insert(odu.name, odu);
        m.insert(odu.universal_name, odu);
    }
    m
});

pub static ODU_TABLE: LazyLock<HashMap<&'static str, OduOp>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for odu in ODU_SET.iter() {
        m.insert(odu.name, odu.opcode.to_op());
    }
    // Shorthand aliases (bare vessel names, used by tests/examples to invoke
    // a wave's opcode directly without a full composite entry)
    m.insert("Genesis", OduOp::PushConst(1));
    m.insert("Void", OduOp::PopVoid);
    m.insert("Attention", OduOp::Dup);
    m.insert("Loop", OduOp::Swap);
    m.insert("Receipt", OduOp::Add);
    m.insert("Mask", OduOp::Sub);
    m.insert("Residue", OduOp::PushConst(0));
    m.insert("Execution", OduOp::CastCowries);
    m.insert("Swarm", OduOp::CastCowries);
    m.insert("Restraint", OduOp::Sub);
    m.insert("Migration", OduOp::Swap);
    m.insert("Consent", OduOp::HaltIfOne);
    m.insert("Vision", OduOp::PushConst(1));
    m.insert("Growth", OduOp::Dup);
    m.insert("Seal", OduOp::Add);
    m.insert("Rhythm", OduOp::HaltIfOne);
    m.insert("CastCowries", OduOp::CastCowries);

    m
});
