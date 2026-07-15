//! odu_ifa — the traditional 256 Òdù Ifá corpus, for human-facing readings.
//!
//! `crate::odu` (the Digital Calabash) and this module are a **dual corpus**:
//! same 256-entry index space, same `Odu`/`ActionVessel`/`OduOpCode` shapes,
//! same wave/vessel/opcode structure — verified by the same compile-time
//! invariants — but different vocabulary for different audiences.
//!
//! - `crate::odu` — agent-native names and archetypes (Steward, Oracle Sage,
//!   Forge Executor, ...). Meant for agent-to-agent casts and internal state.
//! - `crate::odu_ifa` (this module) — the original Yorùbá names and Òrìṣà this
//!   project started with (Ẹ̀jì Ogbe, Olódùmarè, Ọ̀rúnmìlà, ...). Meant for
//!   agent-to-human readings, where the traditional vocabulary is the point.
//!
//! Because both share an index, a single cowrie cast (one entropy draw, one
//! `u8` index) resolves through *either* table — [`crate::vm::IfaVM::cast_odu`]
//! for the agent-facing reading, [`crate::vm::IfaVM::cast_odu_for_human`] for
//! the same cast's traditional reading. Same event, two renderings.
//!
//! Synthetic enrichment (interpretation_type = "synthetic"):
//!   archetypal metadata, taboos, prescriptions built around the traditional
//!   names. Not sourced from ese Ifá — clearly marked, never conflated with
//!   canonical verse. See [`crate::odu`] for the canonical-sourcing note;
//!   the same caveat applies here, just in the original vocabulary.
//!
//! The corpus lives in `waves/wave01.rs` … `waves/wave16.rs`, assembled at
//! compile time exactly like `crate::odu::ODU_SET`, with the same
//! index/binary/vessel invariant check.

pub mod waves;

use crate::odu::Odu;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Lookup by index (0–255). Panics on out-of-range — never call with unchecked input.
pub fn get_odu_ifa(index: u8) -> &'static Odu {
    &ODU_SET_IFA[index as usize]
}

/// Lookup by 8-bit binary value.
pub fn get_odu_ifa_by_binary(binary: u8) -> &'static Odu {
    &ODU_SET_IFA[binary as usize]
}

/// Search by traditional Yorùbá compound name or universal English name.
/// Returns `None` if no entry matches. O(1) via a lazily-built name index.
pub fn lookup_by_name_ifa(name: &str) -> Option<&'static Odu> {
    NAME_INDEX_IFA.get(name).copied()
}

/// The full 256-entry traditional corpus, assembled from the 16 wave files
/// at compile time.
pub static ODU_SET_IFA: [Odu; 256] = assemble();

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

// Same completeness check as crate::odu::ODU_SET: every entry's index and
// binary must match its array position, and its vessel must match the wave
// (top nibble). A corpus edit that violates any invariant is a build error.
const _: () = {
    let mut i = 0;
    while i < 256 {
        assert!(
            ODU_SET_IFA[i].index == i as u8,
            "Odu index must match array position"
        );
        assert!(
            ODU_SET_IFA[i].binary == i as u8,
            "Odu binary must match its index"
        );
        assert!(
            ODU_SET_IFA[i].vessel as u8 == (i >> 4) as u8,
            "Odu vessel must match wave (top nibble of index)"
        );
        i += 1;
    }
};

/// O(1) name → Odù index for `lookup_by_name_ifa`, keyed on both the
/// traditional Yorùbá compound name and the universal English name.
static NAME_INDEX_IFA: LazyLock<HashMap<&'static str, &'static Odu>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(512);
    for odu in ODU_SET_IFA.iter() {
        m.insert(odu.name, odu);
        m.insert(odu.universal_name, odu);
    }
    m
});

/// Confirm the two corpora are structurally identical twins — same vessel,
/// same opcode, same index/binary — for every one of the 256 entries. Only
/// naming/archetype vocabulary should ever differ between them.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::odu::ActionVessel;

    #[test]
    fn structurally_parallel_to_digital_calabash() {
        for i in 0u8..=255 {
            let calabash = crate::odu::get_odu(i);
            let ifa = get_odu_ifa(i);
            assert_eq!(calabash.index, ifa.index, "index mismatch at {i}");
            assert_eq!(calabash.binary, ifa.binary, "binary mismatch at {i}");
            assert_eq!(calabash.vessel, ifa.vessel, "vessel mismatch at {i}");
            assert_eq!(calabash.opcode, ifa.opcode, "opcode mismatch at {i}");
        }
    }

    #[test]
    fn index_zero_is_traditional_yoruba() {
        let odu = get_odu_ifa(0);
        assert_eq!(odu.name, "Ẹ̀jì Ogbe / Ẹ̀jì Ogbe");
        assert_eq!(odu.archetypes, &["Olódùmarè", "Ọ̀rúnmìlà"]);
        assert_eq!(odu.vessel, ActionVessel::Genesis);
    }

    #[test]
    fn lookup_by_traditional_name() {
        let odu = lookup_by_name_ifa("Ẹ̀jì Ogbe / Ẹ̀jì Ogbe");
        assert!(odu.is_some());
        assert_eq!(odu.unwrap().index, 0);
    }
}
