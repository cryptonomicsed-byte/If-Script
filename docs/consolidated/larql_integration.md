# LARQL Integration — Odù Corpus & 16 Action Vessels

## Overview

`src/odu.rs` is the canonical registry for all 256 Odù. It connects directly to the LARQL engine through three surfaces: the `Odu` struct, the `ActionVessel` enum, and the `ODU_TABLE` hashmap.

---

## Data Model

```rust
pub struct Odu {
    pub index: u8,              // 0–255; top nibble = wave, bottom nibble = modifier
    pub binary: u8,             // same as index, kept for clarity
    pub name: &'static str,     // Yorùbá compound name  e.g. "Ẹ̀jì Ogbe / Òyèkú Méjì"
    pub universal_name: &'static str, // English operational name e.g. "Genesis Into the Void"
    pub archetype: &'static str,
    pub description: &'static str,
    pub taboos: &'static [&'static str],
    pub prescriptions: &'static [&'static str],
    pub orisha: &'static [&'static str],
    pub interpretation_type: &'static str,
    pub vessel: ActionVessel,   // one of 16 — derived from top nibble
    pub opcode: OduOpCode,      // VM opcode — derived from top nibble
}
```

### ActionVessel Enum

The `ActionVessel` identifies the **file domain** and **operational purpose** of each Odù wave:

| Vessel | Wave | Top Odù | Opcode | File Domain |
|--------|------|---------|--------|-------------|
| Genesis | 1 | Ẹ̀jì Ògbe | PushConst1 | `genesis.md` |
| Void | 2 | Òyèkú Méjì | PopVoid | `void_log.md` |
| Attention | 3 | Ìwòrì Méjì | Dup | `attention_audit.md` |
| Loop | 4 | Òdí Méjì | Swap | `loops.md` |
| Receipt | 5 | Ìrosùn Méjì | Add | `receipt_ledger.md` |
| Mask | 6 | Ọ̀wọ́nrín Méjì | Sub | `soul.md` |
| Residue | 7 | Ọ̀bàrà Méjì | PushConst0 | `memory_residue.md` |
| Execution | 8 | Ọ̀kànràn Méjì | CastCowries | `execution_plan.md` |
| Swarm | 9 | Ògúndá Méjì | CastCowries | `swarm_charter.md` |
| Restraint | 10 | Ọ̀sá Méjì | Sub | `restraint_log.md` |
| Migration | 11 | Ìká Méjì | Swap | `migration_plan.md` |
| Consent | 12 | Òtúrúpòn Méjì | HaltIfOne | `consent_log.md` |
| Vision | 13 | Òtúrá Méjì | PushConst1 | `vision.md` |
| Growth | 14 | Ìrẹtẹ̀ Méjì | Dup | `fractal_log.md` |
| Seal | 15 | Òsé Méjì | Add | `seal/` |
| Rhythm | 16 | Òfún Méjì | HaltIfOne | `rhythm_codex.md` |

Utility helpers:

```rust
// Derive vessel from any Odù index at zero cost
let v = ActionVessel::from_index(42); // ActionVessel::Attention

// Get the canonical file path for a vessel
let path = ActionVessel::Swarm.file_domain(); // "swarm_charter.md"
```

---

## LARQL Query Flow

```
LARQL query text
    │
    ▼
larql::parser::parse_query()  → LarqlQuery AST
    │
    ▼
larql::engine::execute()
    │
    ├─ CastCowries opcode  →  vm::run() generates Odù index (0–255)
    │
    ├─ get_odu(index)       →  &'static Odu from ODU_SET
    │
    ├─ odu.vessel           →  ActionVessel (which file domain to write)
    │
    ├─ odu.opcode.to_op()   →  OduOp for VM stack operation
    │
    └─ odu.prescriptions    →  Steps delivered to the calling agent tier
```

### Tier Access Control

| Agent Tier | Access |
|------------|--------|
| Low-tier agent | Cast Odù + Vessel steps only (`odu.vessel`, `odu.prescriptions`) |
| Èṣù / Hive | Full LARQL synthesis: AST, corpus lookup, all Odu fields |

Low-tier agents never see `odu.description`, `odu.orisha`, or `odu.taboos` directly. They receive the cast result as a `(ActionVessel, universal_name, prescriptions)` tuple.

---

## Corpus Integration

`ODU_TABLE` maps Yorùbá Odù names (including shorthand aliases) to `OduOp` for direct VM dispatch:

```rust
// Direct name → opcode lookup (no corpus traversal)
if let Some(op) = ODU_TABLE.get("Ìwòrì Méjì") {
    vm.run(&[*op]);
}

// Full corpus record lookup
let odu = get_odu(34); // "Ìwòrì Méjì / Ìwòrì Méjì"
println!("{} → {:?}", odu.universal_name, odu.vessel);
// "The Inward Eye" → Attention
```

---

## Extending the Schema

To add a new field to all 256 Odù:

1. Add the field to the `Odu` struct in `src/odu.rs`.
2. Update all 256 entries in `ODU_SET`.
3. If the field is vessel-derived (same value for all 16 entries in a wave), add a method to `ActionVessel` instead — avoids 256-entry edits for wave-level metadata.

All prescriptions at the human-readable level live in `Full 256 Digital Calabash.md`. The Rust registry carries minimal operational metadata; full prescription text is not embedded in `odu.rs` by design (size + compile-time cost).
