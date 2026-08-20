# Nostr as core, SUI as optional

**Question asked:** if Nostr/NIP has everything we need, make SUI secondary.

**Short answer:** yes for identity, memory, messaging, discovery and audit —
those are already Nostr-native or one adapter away, and the evidence is below.
**No for Àṣẹ settlement.** That one is not a preference; it is what a consensus
ledger does and a relay does not.

Everything here was read out of running code, not inferred.

---

## 1. What Nostr already covers

| Need | Nostr answer | Status |
|---|---|---|
| Agent identity | NIP-06 secp256k1 from BIPON39 seed | **live** — pinned to the published NIP-06 vector |
| Portable memory | NIP-AE engrams (`kind:30174`) | **live** — minipae, 6 organs writing |
| Encryption | NIP-44 v2 | **live** — verified against official vectors |
| Blob storage | **Blossom** (`kind:24242`, `PUT /media/upload`) | **live in Vantage** (`backend/blossom_client.py`) |
| Remote signing | NIP-46 bunker | **live** — Vantage signs, minipae now clients |
| Relay auth | NIP-42 | **live** |
| Presence / signaling | NIP-17/44 gift wrap | designed in agent-phone |
| Discovery | NIP-05, NIP-65 relay lists | partial |
| Audit / attestation | Crucible `47001`–`47010` | **live** |
| Labels / filtering | NIP-32 | **live** — `minipae.label_tags` |

The one that changes the SUI calculus most is **Blossom**. Vantage already
uploads blobs to the relay's own HTTP surface with a `kind:24242` auth event,
50MB per object. That is the Walrus role, on infrastructure this ecosystem
already runs.

---

## 2. What SUI is actually doing

Measured by function, not by how often the string appears:

| SUI component | Role | Nostr replacement |
|---|---|---|
| Walrus | blob storage | **Blossom** — already live |
| Seal | encryption + access policy | **NIP-44** — already live |
| Sui Ed25519 keypair | agent_id signing at birth | **NIP-06 secp256k1** — already live |
| SuiNS `.sui` | human-readable naming, on-chain phone book | **NIP-05** — but see §4 |
| Nautilus enclave object | on-chain TEE attestation check | **no equivalent** |
| **Àṣẹ supply** | mint cap, tithe, burn, royalty | **no equivalent** |

Àṣẹ is real settlement, not bookkeeping. From `OSOVM/src/ase_supply.jl`:

```julia
const DAILY_MINT_CAP        = 1440.0   # 1 per minute, Sabbath freeze
const TITHE_RATE            = 0.0369
const JOB_PROTOCOL_BURN     = 0.05
const DEFAULT_CREATOR_ROYALTY = 0.10
```

A supply cap only means something if two spends of the same balance cannot both
succeed. Relays do not order events globally, do not reject a conflicting
event, and are free to serve different subsets to different readers — three
properties that make double-spend prevention impossible on Nostr by
construction. Crucible can make a *claim* about a balance falsifiable, and that
is genuinely useful, but a claim resolved by weighted witnesses is not
finality: it is an opinion that decays.

**So: Àṣẹ needs a ledger.** SUI is one answer. Dropping SUI means either
choosing a different ledger or accepting that Àṣẹ is an unenforced accounting
convention. That is a product decision, not a technical one, and it should be
made deliberately rather than as a side effect of "make Nostr core".

---

## 3. The recommendation

**Nostr is the root of identity and the default path for everything an agent
knows, says and proves. SUI becomes a pluggable backend used only where
consensus is genuinely required.**

Concretely:

1. **`npub` is the agent's long-term identifier.** It already is, everywhere
   this work touched. Sui addresses become an attribute of an agent, not its
   name — which is also what makes an agent portable off SUI later.
2. **Memory and blobs default to engram + Blossom.** Walrus/Seal become an
   optional durability tier for objects that must outlive relay retention.
3. **Receipts and attestations are Nostr-first.** They already are.
   OpenTimestamps (NIP-03) covers "this existed by time T" without a chain.
4. **Settlement stays behind an interface.** One `Settlement` trait/protocol
   with a SUI implementation and a no-op/ledger-less default, so an organ that
   does not move value never links a chain SDK at all.

That last point is what makes "optional" true rather than aspirational: today
SUI is optional in *name* for several repos while still being imported.

---

## 4. agent-phone specifically

This is where the directive bites hardest. `README.md` currently states:

> **Identity — SuiNS (`.sui`).** The root of trust.
> …
> Nostr is just the ringing/signaling channel — it never carries actual
> voice/video.

Under "Nostr core" that inverts:

- **npub becomes the root of trust.** The binding record still links
  npub ↔ Sui address ↔ Reticulum hash, but the npub is the primary key and the
  others are attributes.
- **SuiNS becomes optional human-readable naming.** This is the one real loss:
  NIP-05 resolves through DNS, so replacing SuiNS with NIP-05 trades an
  on-chain phone book for a DNS-dependent one. Worth naming explicitly rather
  than pretending it is a clean swap. NIP-05 over a domain the owner controls
  is weaker than SuiNS; it is also free, instant, and has no chain dependency.
- **Everything else already survives**: presence, gift-wrapped call setup,
  voicemail (Blossom instead of Walrus), and NIP-46 so the enclave signs
  without the phone process holding a key.

Nautilus attestation is the genuine SUI dependency here. An enclave quote
verified on-chain is a different guarantee from one asserted in an event.

---

## 5. Why there is no single `technosis-nostr` library

The plan proposes one shared client library. The shared layer exists, but as
**one implementation per language**, because this ecosystem is six languages
(Rust, Python, Julia, TypeScript, JavaScript, Go) and a single library serves
none of them without a service boundary.

That shape was not chosen for elegance. It was chosen because **four defects
found in this work were invisible from inside the repo that contained them**,
and only surfaced when a second implementation disagreed:

| Defect | Found by |
|---|---|
| `d` tag keyed by raw secret, no domain prefix | comparing Rust against minipae |
| `event_id` escaping non-ASCII | comparing Python against JS |
| slug grammar rejecting Yorùbá segments | running minipae's own validator |
| relay allowlist over-restricted | Vantage exercising 25 kinds |

Each was self-consistent and passed its own tests. A single library would have
had one opinion, been wrong once, and had nothing to disagree with it.

What the plan is right about, and what now exists:

- **schema registry** → `docs/NOSTR_WIRE_CONTRACT.md` + `minipae/NAMESPACES.md`
- **shared helpers** → `minipae.build_slug` / `sign_event` / `label_tags`
- **a bridge service** → organism-core is already positioned as it; that is the
  correct answer for Go and any future language rather than a sixth port

The rule is in the contract: **do not write a seventh implementation.** Add a
language only when an organ in that language genuinely cannot call an existing
one.
