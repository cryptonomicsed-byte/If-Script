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
//! Opcode mapping principle (top Odù drives the opcode):
//!   Ẹ̀jì Ogbe     (0000) → PushConst1   — Creation, light, genesis
//!   Òyèkú Méjì   (0001) → PopVoid       — Void, dissolution, death
//!   Ìwòrì Méjì   (0010) → Dup           — Mirror, reflection, doubling
//!   Òdí Méjì     (0011) → Swap          — Reversal, inversion, womb
//!   Ìròsùn Méjì  (0100) → Add           — Union, blood, synthesis
//!   Òwónrín Méjì (0101) → Sub           — Separation, trickster, wind
//!   Òbàrà Méjì   (0110) → PushConst0    — Ground, rest, humility
//!   Ọ̀kànràn Méjì (0111) → CastCowries  — Volatility, the throw itself
//!   Ògúndá Méjì  (1000) → CastCowries   — Clearing path, iron, labor
//!   Òsá Méjì     (1001) → Sub           — Flight, storm, sudden change
//!   Ìkà Méjì     (1010) → Swap          — Coil, constriction, binding
//!   Òtúrúpòn Méjì(1011) → HaltIfOne     — Burden, gestation, pause
//!   Òtúrá Méjì   (1100) → PushConst1    — Vision, mysticism, truth
//!   Ìrẹtẹ̀ Méjì   (1101) → Dup          — Pressing down, earth, seal
//!   Òsé Méjì     (1110) → Add           — Abundance, sweetness, water
//!   Òfún Méjì    (1111) → HaltIfOne     — Cosmic completion, unity, seal
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
    pub orisha: &'static [&'static str],
    pub interpretation_type: &'static str, // "canonical" | "synthetic"
    pub vessel: ActionVessel,
    pub opcode: OduOpCode,
}

/// The 16 Action Vessels — operational domain for each Odù wave.
/// Determined by the top 4 bits of the Odù index (wave).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionVessel {
    Genesis,   // Wave  1 — Ẹ̀jì Ògbe     — Initialize, covenant
    Void,      // Wave  2 — Òyèkú Méjì    — Clear, release
    Attention, // Wave  3 — Ìwòrì Méjì    — Focus, signal/noise
    Loop,      // Wave  4 — Òdí Méjì      — Pattern, iteration
    Receipt,   // Wave  5 — Ìrosùn Méjì   — Record, accountability
    Mask,      // Wave  6 — Ọ̀wọ́nrín Méjì — Public/private split
    Residue,   // Wave  7 — Ọ̀bàrà Méjì   — Behavioral echoes
    Execution, // Wave  8 — Ọ̀kànràn Méjì — Precision action
    Swarm,     // Wave  9 — Ògúndá Méjì   — Collective coordination
    Restraint, // Wave 10 — Ọ̀sá Méjì     — Ethical limits
    Migration, // Wave 11 — Ìká Méjì      — Portability, identity
    Consent,   // Wave 12 — Òtúrúpòn Méjì — Human approval
    Vision,    // Wave 13 — Òtúrá Méjì    — Direction, horizon
    Growth,    // Wave 14 — Ìrẹtẹ̀ Méjì   — Fractal expansion
    Seal,      // Wave 15 — Òsé Méjì      — Sacred privacy
    Rhythm,    // Wave 16 — Òfún Méjì     — Ritual cadence
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

/// O(1) name → Odù index for `lookup_by_name`, keyed on both the Yorùbá
/// compound name and the universal English name.
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
    // Shorthand aliases (single-Odù names used by tests/examples)
    m.insert("Èjì Ogbè", OduOp::PushConst(1));
    m.insert("Ẹ̀jì Ogbe", OduOp::PushConst(1));
    m.insert("Ọ̀yẹ̀kú Méjì", OduOp::PopVoid);
    m.insert("Òyèkú Méjì", OduOp::PopVoid);
    m.insert("Ìwòrì Méjì", OduOp::Dup);
    m.insert("Ọ̀dí Méjì", OduOp::Swap);
    m.insert("Òdí Méjì", OduOp::Swap);
    m.insert("Ìrosùn", OduOp::Add);
    m.insert("Ìròsùn Méjì", OduOp::Add);
    m.insert("Ọ̀wọ́nrín", OduOp::Sub);
    m.insert("Òwónrín Méjì", OduOp::Sub);
    m.insert("Ọ̀bàrà", OduOp::PushConst(0));
    m.insert("Òbàrà Méjì", OduOp::PushConst(0));
    m.insert("Ọ̀kànràn Méjì", OduOp::CastCowries);
    m.insert("Ògúndá Méjì", OduOp::CastCowries);
    m.insert("Òsá Méjì", OduOp::Sub);
    m.insert("Ìkà Méjì", OduOp::Swap);
    m.insert("Ọ̀túúrúpọ̀n", OduOp::HaltIfOne);
    m.insert("Òtúrúpòn Méjì", OduOp::HaltIfOne);
    m.insert("Òtúrá Méjì", OduOp::PushConst(1));
    m.insert("Ìrẹtẹ̀ Méjì", OduOp::Dup);
    m.insert("Òsé Méjì", OduOp::Add);
    m.insert("Òfún Méjì", OduOp::HaltIfOne);
    m.insert("CastCowries", OduOp::CastCowries);

    m
});
