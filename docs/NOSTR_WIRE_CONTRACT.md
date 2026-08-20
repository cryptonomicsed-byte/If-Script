# The Ecosystem Nostr Wire Contract

**Status:** implemented in IfáScript (`src/nostr/`), pinned by tests.
**Audience:** every repo in the Ọ̀ṢỌ́VM / ỌMỌ KỌ́DÀ / VANTAGE ecosystem.

Independent repositories, in four languages, do not interoperate because they
share code. They interoperate because they agree on a wire format. This
document is that agreement.

Every constant below was read out of an implementation that already ships it.
Nothing here is inferred from a design document, because several of this
ecosystem's design documents describe behaviour the code does not have — see
[Known contradictions](#known-contradictions).

---

## 1. Identity — one agent, one key

ỌMỌ KỌ́DÀ births agents. A BIPON39 mnemonic seeds two identities off one seed:

| Branch | Curve | Used for |
|---|---|---|
| `DerivationMode::Native` | Ed25519 | kernel identity, agent_id signing |
| `DerivationMode::Bip32` | secp256k1 | **Nostr identity** |

The secp256k1 branch walks the standard NIP-06 path:

```
m/44'/1237'/<account>'/0/0
```

Verified in `BIPON39/src/derivation.rs` (coin type `1237`, hardened purpose
`44'`) and in `Omo-Koda2/omokoda-core/src/identity/nip06.rs`.

**Rule: derive once, adopt everywhere.** Only the birth layer derives. Every
other component *accepts* the resulting 32-byte secret. A component that
re-derives risks the same agent signing under two different pubkeys, which is
indistinguishable on the wire from two different agents.

IfáScript implements this as `NostrIdentity::from_secret_bytes`.
`NostrIdentity::from_mnemonic` exists only for standalone BIP-39 agents with no
kernel, and is pinned against the test vector published in NIP-06 itself — so
it agrees with every other Nostr client, not merely with itself.

> BIPON39 seed derivation is **not** BIP-39. A BIPON39 mnemonic passed to a
> BIP-39 `from_mnemonic` yields a valid-looking but *wrong* key, silently.

---

## 2. Kinds — nobody mints a new one

| Kind | Meaning | Authority |
|---|---|---|
| `7` | reaction / witness vote | NIP-25 |
| `10002` | relay list | NIP-65, `minipae.py::KIND_RELAY_LIST` |
| `22242` | relay auth | NIP-42, `minipae.py::KIND_AUTH` |
| `30174` | agent engram (portable memory) | NIP-AE, `minipae.py::KIND_AGENT_ENGRAM` |
| `47001` | Crucible CLAIM | `crucible-core/src/kinds.rs` |
| `47002` | Crucible ATTESTATION | `crucible-core/src/kinds.rs` |
| `47003` | Crucible CHALLENGE | `crucible-core/src/kinds.rs` |
| `47004` | Crucible VERDICT | `crucible-core/src/kinds.rs` |
| `47007` | falsifier manifest | `crucible-core/src/kinds.rs` |
| `47009` | provenance attestation | `crucible-core/src/kinds.rs` |
| `47010` | oracle verdict | `crucible-core/src/kinds.rs` |

`47000..48000` is **Crucible's reserved block**. No other component may mint a
kind inside it.

### The relay allowlist constrains this absolutely

The production Buzz relay enforces a strict kind allowlist in
`required_scope_for_kind()`. An unadmitted kind is rejected **after**
authentication succeeds, with `restricted: unknown event kind` — which reads
like an auth failure and is not one. Admitted today: `30174` and
`47000..47999`.

Consequence: **a new `kind:31xxx` for "ritual cast" would be unpublishable** on
the one relay this ecosystem actually runs. So domain events borrow existing
vocabularies instead of minting new ones:

- something that happened, and is remembered → **engram, `30174`**
- something asserted, that others should check → **Crucible claim, `47001`**

Implementations should check admissibility locally and fail there, rather than
discovering it at ingest. IfáScript: `kinds::buzz_relay_admits`.

---

## 3. Engrams (`kind:30174`)

Tags, per NIP-AE as implemented in `minipae.py`:

```
["d", HMAC-SHA256(conversation_key, slug) as hex]
["p", owner_pubkey_hex]
```

The slug is **HMAC'd, never published**. A relay operator learns that an agent
wrote something without learning what it named it. Content is NIP-44 encrypted.

### Slug namespaces

Each component owns a prefix so a merged read view stays separable by origin:

| Prefix | Owner |
|---|---|
| `mem/ifa/` | IfáScript |
| `mem/ga/` | OpenAgents bridge |
| `mem/genteam/` | GenTeam / Hermes adapters |
| `mem/buzz/` | Buzz mirrors |

IfáScript writes `mem/ifa/cast/<receipt_hash>`, `mem/ifa/ritual/<name>`,
`mem/ifa/state`.

---

## 4. Claims (`kind:47001`) — assertion requires falsifiability

Crucible's one rule: *you may not assert into the shared belief space without
saying how you could be proven wrong.* A claim carries a content-addressed WASM
falsifier. No falsifier, no claim — rejected at parse time, not by convention.

This matters for governance events specifically. A ritual that reports "gates
passed" is an assertion, and the swarm should be able to check it rather than
take the asserting agent's word.

**A NIP-25 reaction is not an attestation.** A reaction carries no evidence and
no falsifier run. The resolution kernel must not count votes as witnesses:
Crucible discounts agreement for redundancy precisely so that volume cannot
substitute for independent verification.

---

## 5. Domain tags

Shared across components so a reader can filter without parsing content:

| Tag | Value |
|---|---|
| `odu` | Odù index, 0–255 |
| `vessel` | Action Vessel governing the cast |
| `gates` | `true` / `false` — did Kóòdù/Zàngbétò gates pass |

---

## 6. Per-repo status

| Repo | Nostr today | Gap |
|---|---|---|
| Vantage | extensive | conform to this doc |
| Buzz / Crucible | full NIP-01 + BIP-340 | authority for `47xxx` |
| ỌMỌ KỌ́DÀ | NIP-06 identity | authority for identity |
| minipae | NIP-01/06/42/44/65 | authority for `30174` |
| BIPON39 | NIP-06 path | authority for derivation |
| **IfáScript** | **implemented** | relay transport |
| Ọ̀ṢỌ́VM | comments only | **no implementation** |
| **Kóòdù** | **implemented** (unsigned events) | signer wiring |
| **Zàngbétò** | **implemented** | relay transport |
| Loom, Mycelium, Waggle, organism-core, Triune-Memory | none | extensions |

Both governance layers now emit. Kóòdù publishes gate decisions as unsigned
canonical events (it holds no keys by design); Zàngbétò publishes enforcement
receipts signed by a guardian identity derived from its own seed. The remaining
gap is Ọ̀ṢỌ́VM, and relay transport everywhere — no component yet opens a socket.

---

## 7. Canonical serialization — the subtlest way to break interop

A NIP-01 event id is `sha256` over a canonical JSON array:

```
[0, pubkey, created_at, kind, tags, content]
```

serialized with no whitespace. **The signature is over that id**, so two
implementations that serialize differently compute different ids and each
rejects the other's signatures — with no error message that points at
serialization.

NIP-01 requires raw UTF-8. JavaScript's `JSON.stringify` and Rust's serde do
this. **Python's `json.dumps` escapes non-ASCII to `\uXXXX` by default**
(`ensure_ascii=True`), producing a different id. Measured, for content
`"Òrìṣà Ògún"`:

| Serialization | event id |
|---|---|
| `ensure_ascii=True` (Python default) | `f5ceda251451b3571736436644e34ca50eca23ad68ea3e067934e5f8668c2337` |
| raw UTF-8 (NIP-01 correct; JS, Rust) | `e24b148552d35adf425c92e2e701ee3be6b4c86dbfd5fa2cc84a4c922250ac3b` |

**`minipae.py::event_id` uses the default.** This is *latent* today because
engram content is NIP-44 encrypted into ASCII base64 and its tags are hex — so
nothing non-ASCII currently reaches the hash. It activates the moment any
non-ASCII appears in a tag value or in an unencrypted event such as a Crucible
claim.

This ecosystem's vocabulary is Yorùbá. Ritual names, Òrìṣà names and vessel
names all carry diacritics, so any component publishing them in the clear hits
this immediately. Python implementations must pass `ensure_ascii=False`.

Pinned by `koodu/nostr-adapter.test.js` and reproducible with the vector above.

---

## Known contradictions

Recorded because acting on the docs alone would produce broken integrations.

1. **Crucible's `docs/BUZZ.md` claims** a stock Buzz relay "stores and serves
   these events untouched — it just has no opinion about them." The relay's
   `required_scope_for_kind()` did not admit the `47xxx` block; this was
   resolved by an operational patch, not by the claim having been true.
   Verified live in minipae's `docs/D_2_2_RELAY_KIND_COMPATIBILITY.md`.

2. **A publish response is not proof of storage.** minipae shipped a real bug
   where a relay's `auth-required` rejection was misread as `ok: true`. Read
   back what you wrote, over a separate query, before reporting success.

---

## Conformance checklist

- [ ] Identity adopted from the birth layer, never re-derived
- [ ] No kind minted outside an owning authority's block
- [ ] Admissibility checked locally before publish
- [ ] Engram slugs HMAC'd; raw slug never on the wire
- [ ] Claims carry a falsifier
- [ ] Reactions not counted as attestations
- [ ] Secrets excluded from `Debug`/log rendering
- [ ] Writes verified by independent read-back
- [ ] Canonical JSON emits raw UTF-8 (Python: `ensure_ascii=False`)
