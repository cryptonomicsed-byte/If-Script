// if-script/src/seven_bridge.rs
//
// USF-7 ↔ ActionVessel semantic contract
//
// The Universal Seven Functions are the top-level ontology of the sovereign
// ecosystem. The 16 ActionVessels are If-Script's operational semantic layer.
// This module encodes the relationship between the two layers without
// requiring If-Script to import omokoda-hermetic.
//
// Hierarchy:
//
//   UNIVERSAL SEVEN  (0–6)           what kind of agency is acting
//        │
//        ▼
//   16 ACTION VESSELS                the operational domain it acts within
//        │
//        ▼
//   256 ODÙ BASE                     specific semantic state
//        │
//        ▼
//   65,536 COMPOSED ODÙ              full compositional state space
//
// Primary governance: each ActionVessel is primarily governed by ONE SevenFunction.
// Seven functions own 1–3 vessels each (total = 16).
//
//   Spark      → Genesis, Mask                     (2 vessels)
//   Mind       → Attention, Restraint, Vision       (3 vessels)
//   Foundation → Loop, Execution                   (2 vessels)
//   Emotion    → Consent                           (1 vessel)
//   Womb       → Residue, Swarm, Growth             (3 vessels)
//   Fire       → Receipt, Seal                     (2 vessels)
//   Ascension  → Void, Migration, Rhythm            (3 vessels)
//
// Cultural adapters (Yorùbá/Mesopotamian/Hermetic/etc.) apply to the Seven.
// They do not change the ActionVessel mapping — vessels are culture-neutral.

use crate::odu::ActionVessel;

// ─── SevenFunction ────────────────────────────────────────────────────────────

/// Universal function indices (0–6) — mirrors omokoda-hermetic::seven::SevenFunction.
///
/// Values are stable and canonical across the entire sovereign ecosystem.
/// Must be kept in sync with omokoda-hermetic/src/seven/mod.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SevenFunction {
    Spark = 0,      // Agency · Communication · Choice · Initiation
    Mind = 1,       // Reason · Clarity · Ethics · Coherence
    Foundation = 2, // Will · Labor · Execution · Embodiment
    Emotion = 3,    // Value · Relationship · Resonance · Desire
    Womb = 4,       // Creation · Community · Ancestry · Continuity
    Fire = 5,       // Power · Authority · Justice · Consequence
    Ascension = 6,  // Change · Transition · Adaptation · Transformation
}

impl SevenFunction {
    pub const ALL: [SevenFunction; 7] = [
        Self::Spark,
        Self::Mind,
        Self::Foundation,
        Self::Emotion,
        Self::Womb,
        Self::Fire,
        Self::Ascension,
    ];

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Spark),
            1 => Some(Self::Mind),
            2 => Some(Self::Foundation),
            3 => Some(Self::Emotion),
            4 => Some(Self::Womb),
            5 => Some(Self::Fire),
            6 => Some(Self::Ascension),
            _ => None,
        }
    }

    pub fn universal_name(self) -> &'static str {
        match self {
            Self::Spark => "Spark",
            Self::Mind => "Mind",
            Self::Foundation => "Foundation",
            Self::Emotion => "Emotion",
            Self::Womb => "Womb",
            Self::Fire => "Fire",
            Self::Ascension => "Ascension",
        }
    }
}

// ─── Vessel ↔ Function mapping ────────────────────────────────────────────────

/// The SevenFunction that primarily governs a given ActionVessel.
///
/// Rationale for each assignment:
///   Genesis    → Spark      — the first act, covenant, opening of the way
///   Void       → Ascension  — dissolution and release is the essence of change
///   Attention  → Mind       — focus and signal/noise separation = clarity
///   Loop       → Foundation — iteration and work patterns = embodied labor
///   Receipt    → Fire       — accountability and consequence = justice/settlement
///   Mask       → Spark      — the gateway between public/private = choice
///   Residue    → Womb       — behavioral echoes and memory = ancestry
///   Execution  → Foundation — precision action = bringing intention into form
///   Swarm      → Womb       — collective coordination = community
///   Restraint  → Mind       — ethical limits = the coherence principle
///   Migration  → Ascension  — portability and identity across contexts = adaptation
///   Consent    → Emotion    — human approval = the relational/desire principle
///   Vision     → Mind       — direction and horizon = clarity at scale
///   Growth     → Womb       — fractal expansion = creation and continuity
///   Seal       → Fire       — sacred privacy = authority and consequence
///   Rhythm     → Ascension  — ritual cadence = transformation and cycles
pub fn governing_function(vessel: ActionVessel) -> SevenFunction {
    match vessel {
        ActionVessel::Genesis => SevenFunction::Spark,
        ActionVessel::Void => SevenFunction::Ascension,
        ActionVessel::Attention => SevenFunction::Mind,
        ActionVessel::Loop => SevenFunction::Foundation,
        ActionVessel::Receipt => SevenFunction::Fire,
        ActionVessel::Mask => SevenFunction::Spark,
        ActionVessel::Residue => SevenFunction::Womb,
        ActionVessel::Execution => SevenFunction::Foundation,
        ActionVessel::Swarm => SevenFunction::Womb,
        ActionVessel::Restraint => SevenFunction::Mind,
        ActionVessel::Migration => SevenFunction::Ascension,
        ActionVessel::Consent => SevenFunction::Emotion,
        ActionVessel::Vision => SevenFunction::Mind,
        ActionVessel::Growth => SevenFunction::Womb,
        ActionVessel::Seal => SevenFunction::Fire,
        ActionVessel::Rhythm => SevenFunction::Ascension,
    }
}

/// All ActionVessels governed by a given SevenFunction, in priority order.
///
/// Priority is significant: the first vessel is the primary expression.
/// Use `primary_vessel()` when only one is needed.
pub fn vessels_for_function(f: SevenFunction) -> &'static [ActionVessel] {
    match f {
        SevenFunction::Spark => &[ActionVessel::Genesis, ActionVessel::Mask],
        SevenFunction::Mind => &[
            ActionVessel::Attention,
            ActionVessel::Restraint,
            ActionVessel::Vision,
        ],
        SevenFunction::Foundation => &[ActionVessel::Loop, ActionVessel::Execution],
        SevenFunction::Emotion => &[ActionVessel::Consent],
        SevenFunction::Womb => &[
            ActionVessel::Residue,
            ActionVessel::Swarm,
            ActionVessel::Growth,
        ],
        SevenFunction::Fire => &[ActionVessel::Receipt, ActionVessel::Seal],
        SevenFunction::Ascension => &[
            ActionVessel::Void,
            ActionVessel::Migration,
            ActionVessel::Rhythm,
        ],
    }
}

/// The primary (first-priority) ActionVessel for a given SevenFunction.
pub fn primary_vessel(f: SevenFunction) -> ActionVessel {
    vessels_for_function(f)[0]
}

/// Infer the governing SevenFunction from a raw Odù byte.
/// Extracts the vessel (top nibble) and maps it to its governing function.
pub fn function_for_odu(odu_byte: u8) -> SevenFunction {
    governing_function(ActionVessel::from_index(odu_byte))
}

// ─── TwinStateVector ──────────────────────────────────────────────────────────

/// Semantic state vector for a 1:1 digital twin.
///
/// A Twin carries four Odù addresses — one per state dimension:
///   identity_odu   — derived from the Twin's unique Odù seed (BIPỌ̀N39 lineage)
///   memory_odu     — SHA-256(GlyphIndex root) mod 256 → Odù
///   field_odu      — FieldDiviner result at the current BTC height
///   simulation_odu — Ọ̀ṢỌ́VM world-model Odù for this twin's latest sim state
///
/// Together: (WHO the twin is) + (WHAT it remembers) + (WHERE it stands) + (HOW it behaves)
///
/// Each byte maps to a SevenFunction via `function_for_odu()`, making the
/// semantic state queryable at the ontological layer without reading the full
/// memory, field, or simulation state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwinStateVector {
    /// Derived from the twin's Odù seed — stable identity Odù.
    pub identity_odu: u8,
    /// SHA-256(GlyphIndex root) mod 256 — changes with memory.
    pub memory_odu: u8,
    /// FieldDiviner cast at the current BTC height — changes with world state.
    pub field_odu: u8,
    /// Ọ̀ṢỌ́VM world-model output — changes with simulation.
    pub simulation_odu: u8,
}

impl TwinStateVector {
    pub fn new(identity_odu: u8, memory_odu: u8, field_odu: u8, simulation_odu: u8) -> Self {
        Self {
            identity_odu,
            memory_odu,
            field_odu,
            simulation_odu,
        }
    }

    pub fn identity_function(&self) -> SevenFunction {
        function_for_odu(self.identity_odu)
    }

    pub fn memory_function(&self) -> SevenFunction {
        function_for_odu(self.memory_odu)
    }

    pub fn field_function(&self) -> SevenFunction {
        function_for_odu(self.field_odu)
    }

    pub fn simulation_function(&self) -> SevenFunction {
        function_for_odu(self.simulation_odu)
    }

    /// The dominant SevenFunction: the one that appears most across all four dimensions.
    /// Ties broken by priority: identity > field > simulation > memory.
    pub fn dominant_function(&self) -> SevenFunction {
        let priority = [
            self.identity_function(),
            self.field_function(),
            self.simulation_function(),
            self.memory_function(),
        ];
        let mut counts = [0u8; 7];
        for f in &priority {
            counts[*f as usize] += 1;
        }
        let max = *counts.iter().max().unwrap();
        for f in &priority {
            if counts[*f as usize] == max {
                return *f;
            }
        }
        priority[0]
    }

    /// All four Odù in declaration order: [identity, memory, field, simulation].
    pub fn as_array(&self) -> [u8; 4] {
        [
            self.identity_odu,
            self.memory_odu,
            self.field_odu,
            self.simulation_odu,
        ]
    }

    /// XOR fold of all four bytes — compact fingerprint of the full state.
    /// Useful for receipt hashing and change detection.
    pub fn composed_signature(&self) -> u8 {
        self.identity_odu ^ self.memory_odu ^ self.field_odu ^ self.simulation_odu
    }

    /// The primary ActionVessel for each dimension.
    pub fn vessel_state(&self) -> [ActionVessel; 4] {
        [
            primary_vessel(self.identity_function()),
            primary_vessel(self.memory_function()),
            primary_vessel(self.field_function()),
            primary_vessel(self.simulation_function()),
        ]
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governing_function_covers_all_16_vessels() {
        use std::collections::HashSet;
        let vessels = [
            ActionVessel::Genesis,
            ActionVessel::Void,
            ActionVessel::Attention,
            ActionVessel::Loop,
            ActionVessel::Receipt,
            ActionVessel::Mask,
            ActionVessel::Residue,
            ActionVessel::Execution,
            ActionVessel::Swarm,
            ActionVessel::Restraint,
            ActionVessel::Migration,
            ActionVessel::Consent,
            ActionVessel::Vision,
            ActionVessel::Growth,
            ActionVessel::Seal,
            ActionVessel::Rhythm,
        ];
        assert_eq!(vessels.len(), 16);
        // Every vessel has a governing function
        for v in &vessels {
            let _ = governing_function(*v);
        }
        // Each SevenFunction governs at least one vessel
        let governed_functions: HashSet<_> =
            vessels.iter().map(|v| governing_function(*v)).collect();
        assert_eq!(
            governed_functions.len(),
            7,
            "all 7 functions must govern at least one vessel"
        );
    }

    #[test]
    fn vessels_for_function_totals_16() {
        let total: usize = SevenFunction::ALL
            .iter()
            .map(|f| vessels_for_function(*f).len())
            .sum();
        assert_eq!(total, 16, "total vessel assignments must equal 16");
    }

    #[test]
    fn vessels_for_function_roundtrips_governing_function() {
        for f in SevenFunction::ALL {
            for v in vessels_for_function(f) {
                assert_eq!(
                    governing_function(*v),
                    f,
                    "vessel {:?} governing function should be {:?}",
                    v,
                    f
                );
            }
        }
    }

    #[test]
    fn no_vessel_owned_by_two_functions() {
        use std::collections::HashMap;
        let mut seen: HashMap<String, SevenFunction> = HashMap::new();
        for f in SevenFunction::ALL {
            for v in vessels_for_function(f) {
                let key = format!("{:?}", v);
                if let Some(prev) = seen.get(&key) {
                    panic!("vessel {:?} claimed by both {:?} and {:?}", v, prev, f);
                }
                seen.insert(key, f);
            }
        }
    }

    #[test]
    fn primary_vessel_is_first_in_list() {
        for f in SevenFunction::ALL {
            assert_eq!(primary_vessel(f), vessels_for_function(f)[0]);
        }
    }

    #[test]
    fn spark_owns_genesis() {
        assert_eq!(
            governing_function(ActionVessel::Genesis),
            SevenFunction::Spark
        );
        assert_eq!(primary_vessel(SevenFunction::Spark), ActionVessel::Genesis);
    }

    #[test]
    fn mind_owns_attention_restraint_vision() {
        let mind_vessels = vessels_for_function(SevenFunction::Mind);
        assert!(mind_vessels.contains(&ActionVessel::Attention));
        assert!(mind_vessels.contains(&ActionVessel::Restraint));
        assert!(mind_vessels.contains(&ActionVessel::Vision));
    }

    #[test]
    fn ascension_owns_void_migration_rhythm() {
        let asc_vessels = vessels_for_function(SevenFunction::Ascension);
        assert!(asc_vessels.contains(&ActionVessel::Void));
        assert!(asc_vessels.contains(&ActionVessel::Migration));
        assert!(asc_vessels.contains(&ActionVessel::Rhythm));
    }

    #[test]
    fn fire_owns_receipt_and_seal() {
        assert_eq!(
            governing_function(ActionVessel::Receipt),
            SevenFunction::Fire
        );
        assert_eq!(governing_function(ActionVessel::Seal), SevenFunction::Fire);
    }

    #[test]
    fn womb_owns_residue_swarm_growth() {
        assert_eq!(
            governing_function(ActionVessel::Residue),
            SevenFunction::Womb
        );
        assert_eq!(governing_function(ActionVessel::Swarm), SevenFunction::Womb);
        assert_eq!(
            governing_function(ActionVessel::Growth),
            SevenFunction::Womb
        );
    }

    #[test]
    fn function_for_odu_zero_is_genesis_spark() {
        // Odù 0 → top nibble 0 → Genesis → Spark
        assert_eq!(function_for_odu(0x00), SevenFunction::Spark);
    }

    #[test]
    fn function_for_odu_0xf0_is_rhythm_ascension() {
        // Odù 0xF0 → top nibble 0xF (15) → Rhythm → Ascension
        assert_eq!(function_for_odu(0xf0), SevenFunction::Ascension);
    }

    // ─── TwinStateVector ──────────────────────────────────────────────────────

    #[test]
    fn twin_state_dominant_unanimous() {
        // All four dimensions in Genesis wave → all Spark
        let v = TwinStateVector::new(0x00, 0x00, 0x00, 0x00);
        assert_eq!(v.dominant_function(), SevenFunction::Spark);
    }

    #[test]
    fn twin_state_dominant_majority() {
        // identity=Spark(0x00), memory=Spark(0x01), field=Ascension(0xF0), sim=Mind(0x20)
        let v = TwinStateVector::new(0x00, 0x01, 0xf0, 0x20);
        assert_eq!(v.dominant_function(), SevenFunction::Spark);
    }

    #[test]
    fn twin_state_identity_priority_in_tie() {
        // identity=Spark, field=Womb, simulation=Ascension, memory=Fire — all distinct → identity wins
        let v = TwinStateVector::new(0x00, 0x40, 0xf0, 0x80);
        // Each function appears once; identity (Spark) wins by priority
        assert_eq!(v.dominant_function(), SevenFunction::Spark);
    }

    #[test]
    fn twin_composed_signature_xor_fold() {
        let v = TwinStateVector::new(0xAA, 0xBB, 0xCC, 0xDD);
        assert_eq!(v.composed_signature(), 0xAA ^ 0xBB ^ 0xCC ^ 0xDD);
    }

    #[test]
    fn twin_as_array_order() {
        let v = TwinStateVector::new(1, 2, 3, 4);
        assert_eq!(v.as_array(), [1, 2, 3, 4]);
    }

    #[test]
    fn from_u8_roundtrips() {
        for f in SevenFunction::ALL {
            let back = SevenFunction::from_u8(f as u8).unwrap();
            assert_eq!(back, f);
        }
        assert!(SevenFunction::from_u8(7).is_none());
    }
}
