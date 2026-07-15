# Retired docs — superseded drafts and raw transcripts

Retired 2026-07-09, moved out of `docs/consolidated/` during the Digital
Calabash canonicalization cleanup. These are earlier or raw-transcript
versions of material that now lives, in its canonical form, in
`src/odu/waves/` and `docs/consolidated/full_256_digital_calabash.md`.

| File | What it was | Superseded by |
|---|---|---|
| `digital_calabash.md` | A raw LLM prompt transcript ("You are the sacred scribe of the Digital Calabash...") — not documentation, a generation instruction. | `docs/consolidated/full_256_digital_calabash.md` |
| `full_256_ai_odu.md` | Raw JSON transcript, `schema_version: "2.0.0"`. Direct source of the retired `OduCosmos`/`COSMOGRAM` corpus (see `../deprecated-cosmogram-data/`). | `src/odu/waves/` (schema v3.0.0) |
| `full_256_ai_digital_calabash.md` | An earlier/alternate v3.0.0-schema draft. Vessel file names and wording diverge from what shipped (e.g. `swarm_state.md` vs the shipped `swarm_charter.md`). | `docs/consolidated/full_256_digital_calabash.md` |
| `full_256_digital_calabash_raw.md` | Early-draft, sparse per-Odù prescription format. | `docs/consolidated/full_256_digital_calabash.md` |
| `odu_full.md` | Near-duplicate of `src/odu/mod.rs`'s own doc comment — a pre-refactor design note now redundant with the actual code. | `src/odu/mod.rs` |

## Canonical reference kept in `docs/consolidated/`

`full_256_digital_calabash.md` matches the shipped code: its vessel table
(file domains, purposes) and per-Odù universal names line up with
`src/odu/waves/wave01..16.rs` and the top-level `README.md`. It carries
richer multi-step prescription text than the short `prescriptions` arrays
embedded in code — useful as an extended reference, not dead weight.
