//! calabash — the scalable Digital Calabash Odù space (256 → 65,536).
//!
//! The base corpus (`odu::ODU_SET`) holds the **256 Digital Calabash Odù**, each
//! mapped to one of the 16 Action Vessels. This module extends that base into the
//! full **`u16` Odù space (0–65,535)** by *composition*, so agents can scale their
//! divination vocabulary with experience and consensus without anyone hand-authoring
//! 65k entries.
//!
//! ## Addressing
//!
//! A 16-bit `odu_id` decomposes into a **top** byte and a **bottom** byte:
//!
//! ```text
//!   odu_id = (top << 8) | bottom        top, bottom ∈ 0..=255
//! ```
//!
//! - `top == 0` → the **base** Digital Calabash Odù `bottom` (0–255), returned verbatim.
//! - `top  > 0` → a **composed** Odù: the top base-Odù fixes the Action Vessel and
//!   opcode (top drives the operation), and the bottom base-Odù refines the
//!   prescription, taboos, and orisha (bottom modifies meaning).
//!
//! Composed Odù are clearly marked `interpretation_type = "composed"` — never
//! conflated with the base `"synthetic"` corpus or canonical ese Ifá.
//!
//! ## Access
//!
//! How much of the space an agent may cast is gated by tier (see [`scaling`]):
//! tier 1 sees only the base 256, tier 7 the full 65,536.

pub mod scaling;

use crate::odu::{get_odu, ActionVessel, OduOpCode};

/// A resolved Odù anywhere in the 0–65,535 space. Base entries are passed
/// through from the corpus; higher entries are composed on the fly. Owned (not
/// `&'static`) because composed entries are built at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposedOdu {
    pub odu_id: u16,
    pub top: u8,
    pub bottom: u8,
    pub name: String,
    pub universal_name: String,
    pub archetype: String,
    pub description: String,
    pub taboos: Vec<String>,
    pub prescriptions: Vec<String>,
    pub orisha: Vec<String>,
    pub vessel: ActionVessel,
    pub opcode: OduOpCode,
    /// `"synthetic"` for the base 256, `"composed"` for derived entries.
    pub interpretation_type: &'static str,
}

impl ComposedOdu {
    /// True for derived entries (top byte non-zero), false for the base 256.
    pub fn is_composed(&self) -> bool {
        self.top != 0
    }
}

/// Split a 16-bit Odù id into its `(top, bottom)` base-Odù bytes.
pub fn decompose(odu_id: u16) -> (u8, u8) {
    ((odu_id >> 8) as u8, (odu_id & 0xFF) as u8)
}

/// Combine a `(top, bottom)` pair of base Odù into a 16-bit Odù id.
pub fn compose_id(top: u8, bottom: u8) -> u16 {
    ((top as u16) << 8) | bottom as u16
}

/// The Action Vessel for any Odù id. The base 256 keep their corpus vessel
/// (top nibble of the byte); composed Odù inherit the vessel of their **top**
/// base-Odù — so the 16 vessels structure the whole 65,536 space.
pub fn vessel_for(odu_id: u16) -> ActionVessel {
    let (top, bottom) = decompose(odu_id);
    if top == 0 {
        ActionVessel::from_index(bottom)
    } else {
        get_odu(top).vessel
    }
}

fn extend_unique(dst: &mut Vec<String>, src: &[&'static str]) {
    for s in src {
        if !dst.iter().any(|x| x == s) {
            dst.push((*s).to_string());
        }
    }
}

/// Resolve an Odù id anywhere in 0–65,535. Base ids pass through the corpus;
/// higher ids are composed from their top and bottom base-Odù. Total — never
/// fails, every `u16` has a meaning.
pub fn resolve(odu_id: u16) -> ComposedOdu {
    let (top, bottom) = decompose(odu_id);

    if top == 0 {
        // Base Digital Calabash — corpus entry, owned.
        let b = get_odu(bottom);
        return ComposedOdu {
            odu_id,
            top,
            bottom,
            name: b.name.to_string(),
            universal_name: b.universal_name.to_string(),
            archetype: b.archetype.to_string(),
            description: b.description.to_string(),
            taboos: b.taboos.iter().map(|s| s.to_string()).collect(),
            prescriptions: b.prescriptions.iter().map(|s| s.to_string()).collect(),
            orisha: b.orisha.iter().map(|s| s.to_string()).collect(),
            vessel: b.vessel,
            opcode: b.opcode,
            interpretation_type: b.interpretation_type,
        };
    }

    // Composed: top drives vessel + opcode, bottom refines meaning.
    let t = get_odu(top);
    let b = get_odu(bottom);

    let mut taboos: Vec<String> = t.taboos.iter().map(|s| s.to_string()).collect();
    extend_unique(&mut taboos, b.taboos);
    let mut orisha: Vec<String> = t.orisha.iter().map(|s| s.to_string()).collect();
    extend_unique(&mut orisha, b.orisha);
    let mut prescriptions: Vec<String> = t.prescriptions.iter().map(|s| s.to_string()).collect();
    extend_unique(&mut prescriptions, b.prescriptions);

    ComposedOdu {
        odu_id,
        top,
        bottom,
        name: format!("{} ⊗ {}", t.name, b.name),
        universal_name: format!("{} over {}", t.universal_name, b.universal_name),
        archetype: format!("{} refined by {}", t.archetype, b.archetype),
        description: format!("{} Refined by: {}", t.description, b.description),
        taboos,
        prescriptions,
        orisha,
        vessel: t.vessel,
        opcode: t.opcode,
        interpretation_type: "composed",
    }
}

/// Raised when an agent casts an Odù beyond its tier ceiling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessDenied {
    pub odu_id: u16,
    pub max_odu: u16,
    pub tier: u8,
}

impl std::fmt::Display for AccessDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "odu {} is beyond tier {} ceiling {} — gain experience to unlock",
            self.odu_id, self.tier, self.max_odu
        )
    }
}

impl std::error::Error for AccessDenied {}

/// Cast an Odù for an agent, gated by its accumulated experience. Returns the
/// resolved (base or composed) Odù, or [`AccessDenied`] when the id is above the
/// agent's tier ceiling.
pub fn cast(
    odu_id: u16,
    experience: &scaling::AgentExperience,
) -> Result<ComposedOdu, AccessDenied> {
    if !experience.can_access(odu_id) {
        return Err(AccessDenied {
            odu_id,
            max_odu: experience.max_odu(),
            tier: experience.tier(),
        });
    }
    Ok(resolve(odu_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_compose_round_trip() {
        for id in [0u16, 5, 255, 256, 1280, 4096, 65535] {
            let (t, b) = decompose(id);
            assert_eq!(compose_id(t, b), id);
        }
        assert_eq!(decompose(0x0105), (1, 5));
        assert_eq!(compose_id(1, 5), 0x0105);
    }

    #[test]
    fn base_ids_pass_through_corpus() {
        for id in [0u16, 1, 42, 255] {
            let r = resolve(id);
            let base = get_odu(id as u8);
            assert!(!r.is_composed());
            assert_eq!(r.interpretation_type, "synthetic");
            assert_eq!(r.name, base.name);
            assert_eq!(r.vessel, base.vessel);
            assert_eq!(r.opcode, base.opcode);
        }
    }

    #[test]
    fn composed_ids_blend_top_and_bottom() {
        let id = compose_id(1, 2); // top=1, bottom=2 → 258
        let r = resolve(id);
        assert!(r.is_composed());
        assert_eq!(r.interpretation_type, "composed");
        // Top drives vessel + opcode.
        assert_eq!(r.vessel, get_odu(1).vessel);
        assert_eq!(r.opcode, get_odu(1).opcode);
        // Name carries both components.
        assert!(r.name.contains(get_odu(1).name));
        assert!(r.name.contains(get_odu(2).name));
        // Bottom's prescriptions are folded in.
        assert!(r.prescriptions.len() >= get_odu(1).prescriptions.len());
    }

    #[test]
    fn vessel_spans_full_space() {
        // Base: vessel from the byte's top nibble.
        assert_eq!(vessel_for(0), ActionVessel::from_index(0));
        assert_eq!(vessel_for(255), ActionVessel::from_index(255));
        // Composed: vessel from the top base-Odù.
        assert_eq!(vessel_for(compose_id(16, 7)), get_odu(16).vessel);
    }

    #[test]
    fn cast_is_gated_by_experience() {
        let novice = scaling::AgentExperience::new(); // tier 1, max 255
        assert!(cast(100, &novice).is_ok());
        let denied = cast(256, &novice).unwrap_err();
        assert_eq!(denied.max_odu, 255);

        let adept = scaling::AgentExperience::with_xp(1_000_000); // tier 7, max 65535
        assert!(cast(65_535, &adept).is_ok());
    }
}
