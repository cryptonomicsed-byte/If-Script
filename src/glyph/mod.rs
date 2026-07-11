//! Glyph residues — GlyphIndex memory woven into the Digital Calabash.
//!
//! The GlyphIndex layer (canonical reference: `Vantage/backend/glyph_index.py`;
//! key domain: BIPỌ̀N39 `glyphindex` module) stores an agent's sealed memories
//! as content-addressed Unicode glyphs. This module is If-Script's side of the
//! contract:
//!
//! * **GIX-FOLD-v1** — the same deterministic hash → glyph fold used
//!   everywhere in the ecosystem, so a residue computed here matches the
//!   address of the blob sealed in Vantage/OSOVM/Walrus.
//! * **Odù linkage** — every memory chunk maps onto a base Odù (`digest[0]`)
//!   and a composed Odù (`digest[0] << 8 | digest[1]`), tying the Digital
//!   Calabash's 256 → 65,536 scaling to lived memory.
//! * **`cast_with_memory`** — a memory-augmented cast: the cowries supply the
//!   base (bottom) Odù, while the agent's retrieved residues supply the top
//!   byte of a composed Odù. Divination stops being pure entropy and starts
//!   reflecting the agent's actual history — still gated by tier/experience.

use sha2::{Digest, Sha256};

use crate::calabash::{cast as cast_scaled, resolve, AccessDenied, ComposedOdu};
use crate::calabash::scaling::AgentExperience;
use crate::vm::{CastResult, IfaVM};

/// GIX-FOLD-v1 ranges (start, count): all valid, printable BMP scalars.
const FOLD_RANGES: [(u32, u32); 3] = [
    (0x0020, 0xD7FF - 0x0020 + 1),
    (0xE000, 0xFDCF - 0xE000 + 1),
    (0xFDF0, 0xFFFD - 0xFDF0 + 1),
];

/// SHA-256 of a memory chunk — its canonical GlyphIndex address.
pub fn content_hash(text: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    h.finalize().into()
}

/// Fold a content hash onto its display glyph (GIX-FOLD-v1).
pub fn glyph_fold(digest: &[u8; 32]) -> char {
    let total: u64 = FOLD_RANGES.iter().map(|(_, c)| *c as u64).sum();
    let mut rem: u64 = 0;
    for byte in digest {
        rem = (rem << 8 | *byte as u64) % total;
    }
    let mut idx = rem as u32;
    for (start, count) in FOLD_RANGES {
        if idx < count {
            return char::from_u32(start + idx).expect("fold ranges exclude invalid points");
        }
        idx -= count;
    }
    unreachable!()
}

/// (base Odù, composed Odù) linkage for a memory digest.
pub fn odu_link(digest: &[u8; 32]) -> (u8, u16) {
    (digest[0], (digest[0] as u16) << 8 | digest[1] as u16)
}

/// One retrieved memory chunk, as handed back by a GlyphIndex search.
#[derive(Debug, Clone)]
pub struct GlyphResidue {
    pub canonical_id: [u8; 32],
    pub glyph: char,
    pub odu_base: u8,
    pub odu_composed: u16,
    pub chunk: String,
}

impl GlyphResidue {
    pub fn from_chunk(chunk: &str) -> Self {
        let digest = content_hash(chunk);
        let (odu_base, odu_composed) = odu_link(&digest);
        Self {
            canonical_id: digest,
            glyph: glyph_fold(&digest),
            odu_base,
            odu_composed,
            chunk: chunk.to_string(),
        }
    }

    /// Verify the residue's content still matches its canonical address —
    /// the caller may have decrypted it from an untrusted store.
    pub fn verify(&self) -> bool {
        content_hash(&self.chunk) == self.canonical_id
    }
}

/// Deterministic entropy distilled from a set of residues: SHA-256 over the
/// domain tag and the *sorted* canonical ids, so retrieval order is irrelevant.
pub fn residue_entropy(residues: &[GlyphResidue]) -> [u8; 32] {
    let mut ids: Vec<&[u8; 32]> = residues.iter().map(|r| &r.canonical_id).collect();
    ids.sort_unstable();
    let mut h = Sha256::new();
    h.update(b"GIX-CAST/v1");
    for id in ids {
        h.update(id);
    }
    h.finalize().into()
}

/// Outcome of a memory-augmented cast.
#[derive(Debug)]
pub struct MemoryCast {
    /// The plain cowrie cast (always valid, tier-independent guidance).
    pub base: CastResult,
    /// The composed Odù reached by weaving memory into the cast, when the
    /// agent's tier permits it.
    pub composed: Result<ComposedOdu, AccessDenied>,
    /// The composed id that memory + cowries selected (even when gated).
    pub composed_id: u16,
    /// How many residues actually contributed (invalid ones are dropped).
    pub residues_used: usize,
}

/// Cast with the agent's memory in the calabash.
///
/// The cowries choose the bottom (base) Odù exactly as [`IfaVM::cast_odu`]
/// would; the top Odù comes from the residue entropy. Access to the composed
/// result stays gated by `experience` — an agent cannot divine above its tier
/// just because it remembers a lot.
pub fn cast_with_memory(
    vm: &mut IfaVM,
    residues: &[GlyphResidue],
    experience: &AgentExperience,
) -> MemoryCast {
    let base = vm.cast_odu();
    let valid: Vec<GlyphResidue> = residues.iter().filter(|r| r.verify()).cloned().collect();
    let composed_id = if valid.is_empty() {
        base.index as u16 // no memory → plain base cast
    } else {
        let entropy = residue_entropy(&valid);
        (entropy[0] as u16) << 8 | base.index as u16
    };
    let composed = if composed_id < 256 {
        Ok(resolve(composed_id))
    } else {
        cast_scaled(composed_id, experience)
    };
    MemoryCast {
        base,
        composed,
        composed_id,
        residues_used: valid.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calabash::scaling::AgentExperience;

    /// Frozen cross-language vectors from the canonical Python reference.
    const FOLD_VECTORS: &[(&str, u32, u8, u16)] = &[
        ("Àṣẹ", 21841, 227, 58152),
        ("hello", 23636, 44, 11506),
        ("GlyphIndex", 13726, 68, 17595),
        ("😊🚀 Unicode test", 64591, 189, 48626),
        ("Ọ̀rúnmìlà", 17963, 204, 52390),
    ];

    #[test]
    fn fold_matches_canonical_vectors() {
        for (text, codepoint, base, composed) in FOLD_VECTORS {
            let digest = content_hash(text);
            assert_eq!(glyph_fold(&digest) as u32, *codepoint);
            assert_eq!(odu_link(&digest), (*base, *composed));
        }
    }

    #[test]
    fn residue_roundtrip_and_verify() {
        let r = GlyphResidue::from_chunk("User: remember the river crossing");
        assert!(r.verify());
        let mut tampered = r.clone();
        tampered.chunk.push('!');
        assert!(!tampered.verify());
    }

    #[test]
    fn residue_entropy_is_order_independent() {
        let a = GlyphResidue::from_chunk("first memory");
        let b = GlyphResidue::from_chunk("second memory");
        assert_eq!(
            residue_entropy(&[a.clone(), b.clone()]),
            residue_entropy(&[b, a])
        );
    }

    #[test]
    fn memory_cast_respects_tier_gating() {
        let mut vm = IfaVM::with_intent("test intent");
        let residues = vec![GlyphResidue::from_chunk("a memory that shapes the cast")];
        let novice = AgentExperience::new();
        let cast = cast_with_memory(&mut vm, &residues, &novice);
        assert_eq!(cast.residues_used, 1);
        if cast.composed_id >= 256 {
            // A novice must not unlock composed Odù via memory alone…
            assert!(cast.composed.is_err());
        }
        // …but a Hive-tier agent may.
        let mut vm2 = IfaVM::with_intent("test intent");
        let hive = AgentExperience::with_xp(u64::MAX);
        let cast2 = cast_with_memory(&mut vm2, &residues, &hive);
        assert!(cast2.composed.is_ok());
    }

    #[test]
    fn tampered_residues_are_excluded() {
        let mut vm = IfaVM::with_intent("test intent");
        let mut bad = GlyphResidue::from_chunk("legit");
        bad.chunk = "forged".into();
        let cast = cast_with_memory(&mut vm, &[bad], &AgentExperience::new());
        assert_eq!(cast.residues_used, 0);
        assert!(cast.composed_id < 256, "forged residue must not steer the cast");
    }
}
