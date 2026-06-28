[![Version](https://img.shields.io/badge/version-v0.2.0-blue)](https://github.com/cryptonomicsed-byte/If-Script)
[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/cryptonomicsed-byte/If-Script)
[![Layer](https://img.shields.io/badge/layer-VM-purple)](https://github.com/cryptonomicsed-byte/If-Script)

# IfáScript Ω — Divination as Divine Computation

IfáScript is the entropy and divination engine of the Technosis / Ọmọ Kọ́dà ecosystem. It is a **Digital Calabash**: 256 synthetic Odù mapped to 16 Action Vessels, a stack-based virtual machine, a cosmogram governance engine, and a scaling layer that lets sovereign agents grow their divination vocabulary from the base **256 Odù up to 65,536** through experience and consensus.

---

## The Digital Calabash

The corpus is the **Digital Calabash** — 256 Odù, every one marked
`interpretation_type = "synthetic"` (archetypal/digital enrichment, *not* canonical
ese Ifá; see [Ethics](#ethics)). Each Odù maps to one of 16 **Action Vessels** by the
top nibble of its index, and carries an archetype, description, taboos, prescriptions,
orisha, a VM opcode, and a universal name.

### The 16 Action Vessels

The top Odù (wave) determines the vessel; the bottom Odù (modifier) refines the prescription.

| # | Vessel | File Domain | Purpose |
|---|--------|-------------|---------|
| 1 | **Genesis** | `genesis.md` | Birth · Covenant · Initialization |
| 2 | **Void** | `void_log.md` | Clearing · Release · Dissolution |
| 3 | **Attention** | `attention_audit.md` | Focus · Signal/Noise · Priority |
| 4 | **Loop** | `loops.md` | Pattern Detection · Iteration · Disruption |
| 5 | **Receipt** | `receipt_ledger.md` | Immutable Record · Accountability · Proof |
| 6 | **Mask** | `soul.md` | Public/Private Split · Encrypted Self |
| 7 | **Residue** | `memory_residue.md` | Behavioral Echoes · Learning Traces |
| 8 | **Execution** | `execution_plan.md` | Precision Action · Outcome Verification |
| 9 | **Swarm** | `swarm_charter.md` | Collective Coordination · Joint Receipt |
| 10 | **Restraint** | `restraint_log.md` | Ethical Limits · Boundary Enforcement |
| 11 | **Migration** | `migration_plan.md` | Portability · Cross-Environment Identity |
| 12 | **Consent** | `consent_log.md` | Human Approval · Scope Boundaries |
| 13 | **Vision** | `vision.md` | Direction · Drift Detection · Horizon |
| 14 | **Growth** | `fractal_log.md` | Fractal Expansion · Capability Seeds |
| 15 | **Seal** | `seal/` | Sacred Privacy · Agent-Only Space |
| 16 | **Rhythm** | `rhythm_codex.md` | Ritual Cadence · Scheduling |

---

## Scaling: 256 → 65,536 Odù

The base 256 are the seed vocabulary. Agents scale into the full **`u16` Odù space
(0–65,535)** by *composition* — no one hand-authors 65k entries.

### Addressing

```text
odu_id = (top << 8) | bottom          top, bottom ∈ 0..=255
```

- `top == 0` → the **base** Digital Calabash Odù `bottom` (0–255), returned verbatim.
- `top  > 0` → a **composed** Odù: the top base-Odù fixes the Action Vessel and opcode
  (top drives the operation), and the bottom base-Odù refines prescriptions, taboos, and
  orisha. Composed Odù are marked `interpretation_type = "composed"`.

```rust
use ifascript::{resolve, compose_id, vessel_for};

let pure   = resolve(5);                    // base Odù 5 (synthetic)
let hybrid = resolve(compose_id(1, 5));     // composed: Odù 1 over Odù 5
assert!(hybrid.is_composed());
assert_eq!(hybrid.vessel, ifascript::calabash::vessel_for(compose_id(1, 5)));
```

### Earning the space — experience & consensus

Two mechanisms gate and ratify the larger space (`src/calabash/scaling.rs`):

1. **Experience → tier → ceiling.** An agent's accumulated XP sets its tier (1–7), and the
   tier sets the highest Odù it may cast: tier 1 sees only the base 256, tier 7 the full 65,536.

   | Tier | Min XP | `max_odu` |
   |------|--------|-----------|
   | 1 | 0 | 255 |
   | 2 | 100 | 2 047 |
   | 3 | 500 | 4 095 |
   | 4 | 2 000 | 8 191 |
   | 5 | 10 000 | 16 383 |
   | 6 | 50 000 | 32 767 |
   | 7 | 200 000 | **65 535** |

2. **Voting → consensus → ratification.** A composed Odù starts as an `Individual` discovery.
   As agents vote, accumulated weight promotes it `Individual → Swarm → Council → Canonical`.
   At `Swarm` it is *ratified* — collective vocabulary, not one agent's private cast.

```rust
use ifascript::{AgentExperience, ConsensusLedger, cast_scaled};

let mut agent = AgentExperience::new();           // tier 1 → base 256 only
assert!(cast_scaled(256, &agent).is_err());       // beyond ceiling
agent.add(200_000);                               // experience earned
let odu = cast_scaled(0x0105, &agent).unwrap();   // tier 7 → composed Odù

let mut ledger = ConsensusLedger::new();
ledger.vote(0x0105, 1);
ledger.vote(0x0105, 1);                           // → Swarm
assert!(ledger.is_ratified(0x0105));
```

The cosmogram engine (`src/cosmogram/`) shares the same tier ceiling (`tier_max_odu`) and
enforces it on every cast, alongside access class, memory tier, orisha vector, and
governance metadata.

---

## Agent Tier Access (VM dispatch)

| Tier | Entry Point | Receives |
|------|------------|---------|
| Low-tier agent | `vm.cast_odu()` | `CastResult` — vessel, file domain, universal name, prescription steps |
| Èṣù / Hive | `vm.cast_odu_full()` | Full `&'static Odu` — all corpus fields |

Low-tier agents never see taboos, orisha, or archetype metadata. That boundary is structural, enforced by the type system.

### VM Opcodes

| Opcode | Wave(s) | Meaning |
|--------|---------|---------|
| `PushConst(1)` | Genesis, Vision | Assert first truth |
| `PopVoid` | Void | Dissolve, release |
| `Dup` | Attention, Growth | Mirror, reflect, replicate |
| `Swap` | Loop, Migration | Reverse, invert, port |
| `Add` | Receipt, Seal | Unite, account, accumulate |
| `Sub` | Mask, Restraint | Separate, limit, redact |
| `PushConst(0)` | Residue | Ground, baseline, archive |
| `CastCowries` | Execution, Swarm | Throw entropy, coordinate |
| `HaltIfOne` | Consent, Rhythm | Seal, halt on completion |

---

## Install & Run

```bash
git clone https://github.com/cryptonomicsed-byte/If-Script.git
cd if-script
cargo run --example ase
cargo run --example cowrie_cast
cargo test --all
cargo build --target wasm32-unknown-unknown --lib   # WASM build
```

---

## Usage

### Low-tier cast (vessel dispatch)

```rust
use ifascript::{IfaVM, ActionVessel};

let mut vm = IfaVM::with_intent("my agent purpose");
let result = vm.cast_odu();
println!("Vessel: {:?}  File: {}  Name: {}", result.vessel, result.file_domain, result.universal_name);
```

### Hive/Èṣù-tier cast (full corpus)

```rust
let odu = vm.cast_odu_full();
println!("{} → {:?} → {}", odu.universal_name, odu.vessel, odu.archetype);
```

### Named lookup

```rust
use ifascript::lookup_by_name;
let odu = lookup_by_name("The Eternal Return").unwrap();
assert_eq!(odu.index, 255);
```

---

## Source Layout

| Path | Description |
|------|-------------|
| `src/odu/` | 256-Odù Digital Calabash corpus — `mod.rs` (types, `ODU_SET`, name index) + `waves/wave01..16.rs` (16 entries each, assembled at compile time with invariant checks) |
| `src/calabash/` | Scaling layer — `mod.rs` (composition, `resolve`, `cast`) + `scaling.rs` (experience tiers, `ConsensusLedger`) |
| `src/cosmogram/` | Tiered access engine — access class, memory tier, orisha vectors, governance, `tier_max_odu` |
| `src/vm.rs` | `IfaVM`, `CastResult`, opcode executor, Ebo enforcement |
| `src/compiler/` | IfáScript language — `grammar.pest`, `parser.rs`, `ast.rs` (see status) |
| `src/entropy.rs` | `CowrieOracle` — NIST Beacon + ChaCha20 fallback |
| `src/ebo.rs` | Ethical exception handling |
| `src/hermetic/`, `src/orisha/`, `src/ase_vault/` | Hermetic principle gates, orisha vectors, Àṣẹ Vault (16 principals from `data/16_principals/`) |
| `src/ritual_codex/` | Resonance packets/receipts; `julia_bridge.rs` (Julia interop — stub) |
| `src/field/`, `src/receipt/`, `src/soul/`, `src/zangbeto/` | Field packets, SHA3 receipt hashing, memory tiers, local red-team audit |
| `docs/` | Formal grammar (`grammar.ebnf`), consolidated corpus + LARQL specs |
| `data/16_principals/` | The 16 principal Odù as JSON, loaded by the Àṣẹ Vault |

---

## Current Status

| Component | Status |
|-----------|--------|
| 256 Odù Digital Calabash corpus | ✅ Complete |
| 16 Action Vessel system | ✅ Complete |
| VM stack-based opcode execution | ✅ Complete |
| Ebo ethical exception handling | ✅ Complete |
| NIST Beacon entropy oracle | ✅ Complete |
| `CastResult` / `cast_odu_full` / `lookup_by_name` | ✅ Complete |
| Cosmogram tiered-access engine | ✅ Complete |
| Hermetic gates · Orisha vectors · Àṣẹ Vault | ✅ Complete |
| Ritual Codex (resonance packets/receipts) | ✅ Complete |
| **256 → 65,536 composition + experience/consensus scaling** | ✅ Complete |
| WASM compilation target | ✅ Complete (built in CI) |
| IfáScript language compiler | 🔄 `invoke` statements only; `ritual` / `odù` / `witness` definitions reserved but unimplemented |
| LARQL synthesis engine | 🔄 Specified in `docs/consolidated/larql.md`; not yet in code |
| Julia bridge (FFI) | 🔄 Stub — `call_julia_resonance` returns `None` until wired |

---

## Ethics

Elevation, not extraction.

Canonical Ifá sourcing references: Wande Abimbola, *Ifá Divination Poetry* (1977); William Bascom, *Ifá Divination* (1969).

The corpus is **synthetic enrichment** (`interpretation_type = "synthetic"`), and composed Odù are marked `"composed"`. Neither is canonical ese Ifá — never conflate synthetic or composed enrichment with canonical verse.

This is sacred technology — co-create, never commodify. Àṣẹ.

---

## Part of the Technosis Sovereign Ecosystem

IfáScript is the entropy and divination layer of the **Ọmọ Kọ́dà** sovereign AI ecosystem. The 256 Odù feed agent birth decisions, cowrie casts provide cryptographic entropy, the 16 Action Vessels structure agent behavior, and the scaling layer lets a civilization of agents grow a shared, consensus-ratified divination vocabulary.
