# Retired: `OduCosmos` / `COSMOGRAM`

Retired 2026-07-09. This was a second, parallel 256-entry Odù corpus
(`src/cosmogram/data.rs`), independent of the canonical corpus in
`src/odu/mod.rs` + `src/odu/waves/`.

## Why it was retired

- **Different, older schema** (`sacred_name`, `orisha_primary`/`orisha_secondary`,
  `domain`, `core_theme`, `ese_myth`, `hermetic_gate`) matching the raw
  `schema_version: "2.0.0"` JSON transcript now at
  `../deprecated-docs/full_256_ai_odu.md`, not the `Odu` struct's current
  v3.0.0 schema (`taboos`, `prescriptions`, `orisha`, `opcode`, `vessel`).
- **Incomplete** — only 123 of 256 entries had real data; the rest were
  blank placeholder structs (`sacred_name: ""`, etc.).
- **Disconnected from the runtime.** `CosmogramEngine::cast()`
  (`src/cosmogram/mod.rs`) — the actual tiered-access cast logic — never
  read `COSMOGRAM`/`OduCosmos`. It was dead data, exercised only by tests
  that asserted its own incompleteness (`test_cosmogram_123_entries_have_data`).

## Canonical replacement

`src/odu/mod.rs` (assembler + invariant checks) + `src/odu/waves/wave01..16.rs`
(256 entries, 16 per wave) is the one corpus wired into `IfaVM`,
`calabash::resolve`/`cast_scaled`, `manifesto`, and `lookup_by_name`. It is
complete (compile-time-asserted: every index/binary/vessel checked against
its array position) and matches the vessel table and universal names in
`../deprecated-docs/../../docs/consolidated/full_256_digital_calabash.md`
and the top-level `README.md`.

## Salvageable content

The 123 populated entries here have `ese_myth` narrative prose that the
canonical `Odu` struct doesn't carry (it has `taboos`/`prescriptions`/`orisha`
instead). Porting that prose into the canonical corpus — and writing the
missing 133 — is a follow-up content task, intentionally out of scope for
the cleanup that retired this file.
