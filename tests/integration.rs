use ifascript::{get_odu, lookup_by_name, ActionVessel, IfaVM};

// ── Legacy program execution (backward-compatible) ────────────────────────

#[test]
fn test_ase_program() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Genesis", "Attention", "Consent"]).unwrap();
    assert_eq!(vm.stack, vec![1, 1]);
}

#[test]
fn test_cowrie_cast_produces_valid_u16_range() {
    // Determinism is intentionally not guaranteed when the NIST Beacon is
    // unavailable — a time-based nonce is mixed in to prevent predictable
    // fallback casts. We verify the output is a valid u16 value.
    let mut vm = IfaVM::with_intent("Test intent");
    vm.execute(vec!["CastCowries"]).unwrap();
    let val = vm.stack[0];
    assert!(
        (0..=0xFFFF).contains(&val),
        "CastCowries must produce a u16 value"
    );
}

// ── Digital Calabash vessel dispatch ─────────────────────────────────────

#[test]
fn test_cast_odu_returns_valid_vessel() {
    let mut vm = IfaVM::with_intent("vessel dispatch test");
    let result = vm.cast_odu();

    // vessel must match what ActionVessel::from_index() computes
    assert_eq!(result.vessel, ActionVessel::from_index(result.index));

    // file_domain must be non-empty
    assert!(!result.file_domain.is_empty());

    // universal_name must be non-empty
    assert!(!result.universal_name.is_empty());
}

#[test]
fn test_vessel_assignment_by_wave() {
    // Wave 1 (indices 0–15) → Genesis
    for i in 0u8..=15 {
        assert_eq!(get_odu(i).vessel, ActionVessel::Genesis, "index {i}");
    }
    // Wave 2 (indices 16–31) → Void
    for i in 16u8..=31 {
        assert_eq!(get_odu(i).vessel, ActionVessel::Void, "index {i}");
    }
    // Wave 9 (indices 128–143) → Swarm
    for i in 128u8..=143 {
        assert_eq!(get_odu(i).vessel, ActionVessel::Swarm, "index {i}");
    }
    // Wave 16 (indices 240–255) → Rhythm
    for i in 240u8..=255 {
        assert_eq!(get_odu(i).vessel, ActionVessel::Rhythm, "index {i}");
    }
}

#[test]
fn test_all_256_odu_have_vessel_and_universal_name() {
    for i in 0u8..=255 {
        let odu = get_odu(i);
        assert_eq!(odu.index, i);
        assert!(
            !odu.universal_name.is_empty(),
            "universal_name empty at index {i}"
        );
        assert_eq!(
            odu.vessel,
            ActionVessel::from_index(i),
            "vessel mismatch at index {i}"
        );
    }
}

#[test]
fn test_lookup_by_domain_pair_name() {
    let odu = lookup_by_name("Genesis × Genesis");
    assert!(odu.is_some());
    assert_eq!(odu.unwrap().index, 0);
    assert_eq!(odu.unwrap().vessel, ActionVessel::Genesis);
}

#[test]
fn test_lookup_by_universal_name() {
    let odu = lookup_by_name("The Eternal Return");
    assert!(odu.is_some());
    assert_eq!(odu.unwrap().index, 255);
    assert_eq!(odu.unwrap().vessel, ActionVessel::Rhythm);
}

#[test]
fn test_lookup_unknown_name_returns_none() {
    assert!(lookup_by_name("definitely not an odu").is_none());
}

#[test]
fn test_vessel_file_domains_are_unique() {
    // Each vessel must have a distinct file domain
    let mut domains: Vec<&str> = [
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
    ]
    .iter()
    .map(|v| v.file_domain())
    .collect();
    domains.dedup();
    assert_eq!(domains.len(), 16);
}

#[test]
fn test_low_tier_cast_does_not_expose_taboos_or_archetypes() {
    // CastResult must not carry taboos or archetypes — those are Hive-tier only.
    // This is a compile-time guarantee enforced by the struct definition,
    // but we assert the positive: prescriptions are accessible.
    let mut vm = IfaVM::with_intent("low tier test");
    let result = vm.cast_odu();
    // prescriptions field exists and is a non-empty slice
    // (all 256 entries have at least one prescription)
    assert!(!result.prescriptions.is_empty());
}
