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

### The relay allowlist constrains this

The production Buzz relay enforces a kind allowlist in
`required_scope_for_kind()` (`buzz-relay/src/handlers/ingest.rs`). A kind with
no match arm is rejected **after** authentication succeeds, with
`restricted: unknown event kind` — which reads like an auth failure and is not
one.

That allowlist is **broad**, not minimal. It covers most of Buzz's own
vocabulary — kind `1` (text note), `5` (deletion), `7` (reaction), `1059`
(gift wrap), `30023` (long-form), `30315` (user status), the NIP-51 lists,
NIP-65 relay lists, agent profiles, stream messages — plus `30174`, plus the
patched `47000..47999`. Vantage exercises roughly 25 of these in production.

What it does **not** cover is an arbitrary new kind. So a fresh `kind:31xxx`
for "ritual cast" would be unpublishable on the one relay this ecosystem
actually runs, and domain events borrow existing vocabularies instead:

- something that happened, and is remembered → **engram, `30174`**
- something asserted, that others should check → **Crucible claim, `47001`**

> **Do not write a local mirror of the relay's allowlist.** A copy drifts out
> of sync silently while asserting an authority the copying component cannot
> verify. An earlier revision of this document said the relay admitted "only
> `30174` and `47xxx`", and four implementations encoded that — which would
> have reported kind `7` as inadmissible, the very NIP-25 witness vote §4
> prescribes.
>
> Guard on the narrower, checkable claim instead: *the kinds this component
> emits*. IfáScript, Zàngbétò, Kóòdù and Ọ̀ṢỌ́VM each expose `is_publishable`,
> where `false` means "not ours", never "the relay would refuse it".

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

| Repo | Signs? | Status | Remaining |
|---|---|---|---|
| Buzz / Crucible | yes | authority for `47xxx` + relay allowlist | — |
| minipae | yes | authority for `30174`; **id serialization fixed** | — |
| BIPON39 | n/a | authority for NIP-06 derivation | — |
| ỌMỌ KỌ́DÀ | yes | authority for agent identity | conform to §7 |
| Vantage | yes | extensive; `_event_id` already §7-correct | conform to §2 |
| **IfáScript** | yes | **implemented + transport**, 150 tests | — |
| **Zàngbétò** | yes | **implemented + transport**, 52 tests | — |
| **Kóòdù** | no, by design | **implemented**, 18 tests | signer wiring |
| **Ọ̀ṢỌ́VM** | no, by design | **implemented**, tests unrun (no Julia) | run tests; signer wiring |
| **organism-core** | no, by design | **implemented**, 19 tests — shared TS module | signer wiring |
| **Mycelium** | via minipae | **implemented**, 17 tests | falsifier modules |
| **Loom** | via minipae | **implemented**, 15 tests | falsifier modules |
| **Waggle** (Agentic) | via minipae | **implemented**, 17 tests | falsifier modules |
| **Triune-Memory** | no, by design | **implemented**, 8 tests | signer wiring |

### One implementation per language

The contract's failure mode is silent divergence, so every added copy is
added risk. Do not write a sixth:

| Language | Module | Signs? |
|---|---|---|
| Python | `minipae.py` | yes |
| Rust | `ifascript::nostr`, `zangbeto_enforcement::nostr_bridge` | yes |
| TypeScript | `organism-core/bridge/nostr-wire.ts` | no |
| Julia | `OSOVM/src/nostr_bridge.jl` | no |
| JavaScript | `Koodu/nostr-adapter.js` | no |

The TypeScript module is namespace-parameterised, so a TS organ passes its own
`mem/<name>/` prefix rather than forking it. The remaining extension organs are
Python (Mycelium, Loom, Waggle) or TypeScript (Triune-Memory, Mycelium's TS
surface) and adopt the corresponding row.

Both governance layers now emit, and so does the VM. Kóòdù and Ọ̀ṢỌ́VM build
unsigned canonical events — neither holds agent keys, and neither should;
IfáScript and Zàngbétò sign with identities derived from seeds they own.

**Transport** is implemented in both signing components — NIP-42 auth, publish,
and read-back verification — and exercised against a loopback mock relay.

### The d-tag: where cross-implementation checking earned its keep

The engram `d` tag is an address, and IfáScript and Zàngbétò computed it as
`HMAC(raw_secret, slug)`. The real construction, read from `minipae.py::d_tag`,
is:

```
HMAC-SHA256(key = conversation_key(seckey, owner_pubkey),
            msg = "agent-memory/v1/d-tag" || 0x00 || slug)
```

Both the key and the message were wrong, so every engram those two wrote landed
where no minipae client would look. The code was entirely self-consistent and
its own tests all passed; only comparing against a *different* implementation
exposed it. Both now match minipae byte-for-byte on a pinned vector.

This is the argument for one implementation per language, stated as evidence
rather than principle: four of the defects found across this work were invisible
from inside the repo that contained them.

### Still not true

1. **Nothing has touched the production relay.** The environment's network
   policy answers 403 to `CONNECT` for every relay host
   (`relay.damus.io`, `nos.lol`, `relay.nostr.band` all refused at the
   gateway), so this could not be attempted here. The mock encodes what the
   ecosystem's own records say the relay does; that is not the same as the
   relay doing it.
2. **Ọ̀ṢỌ́VM's tests have never run.** Julia is absent, `julialang.org` is
   blocked by the same policy, and no distro package exists. The serialization
   algorithm was verified by porting its rules to Python and reproducing the
   pinned vector — field order, separators and escaping all check out — but
   Julia syntax and semantics were not.
3. ~~No falsifier modules exist.~~ **Done** — see `falsifiers/`. Four modules
   covering the claims this ecosystem emits, 24 tests, addresses in §8.

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

## 8. Falsifiers

Crucible rejects a claim with no falsifier at parse time, so a claim path
without one can be built and never resolved. `falsifiers/` supplies them.

| Module | Claim | Pure? | Address |
|---|---|---|---|
| `gates_passed` | governance gates passed | yes | `sha256:18d904870ba845d962ac72cb9cc3198b877c28e0257fac839bc51c1d4e558994` |
| `deterministic_execution` | job ran deterministically | yes | `sha256:975d498fa6991d920b02e26be00fa771f0d1cb4109ae44c2d3a7d12b77c0a9ed` |
| `enforcement_proportionate` | enforcement matched the anomaly | yes | `sha256:2f628270fbc78111b3f6e2172f4f78840ec39f4c8d84f8db84a4382992e41547` |
| `signal_resolved` | directional call was correct | no (`market:close`) | `sha256:bc150381144666386b67fa9d95ed96e8f3e5590ab926112c4a1b7a3e66394a41` |
| `market_resolved` (wasm) | market settled as claimed | no (`market:resolution`) | `sha256:d4c928dc5e988b9f4d9a46272c6093d808a22963093b8e9f736feae77aa04af1` |

The `market_resolved` **binary** in `src/bin/` is a gatherer, not an entry
in this table: it fetches a settlement for a probe to pass in. Only the wasm
module above is content-addressed and executed by Crucible.

Which one each emitter passes as its `falsifier`:

| Emitter | Module |
|---|---|
| IfáScript `ritual_claim` | `gates_passed` |
| Kóòdù `gateClaim` | `gates_passed` |
| Ọ̀ṢỌ́VM `execution_claim` | `deterministic_execution` |
| Zàngbétò `enforcement_claim` | `enforcement_proportionate` |
| Loom `build_signal_claim` | `signal_resolved` |
| Mycelium, Waggle | none yet — findings need a predicate per finding type |

Two rules a new falsifier must satisfy, both enforced rather than advised:

- **Cannot see → `indeterminate`, never `fails`.** The kernel weighs a verdict
  as evidence either way, so treating blindness as refutation lets an outage
  manufacture disagreement.
- **A pure module's outcome must move under mutation of its inputs.**
  `claim.build` runs `audit_vacuity` and refuses one that does not — a module
  returning `holds` unconditionally is worse than none, because it looks like
  one.

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
- [ ] Kind guard states what the component emits, not what the relay accepts
- [ ] Admissibility checked locally before publish
- [ ] Engram slugs HMAC'd; raw slug never on the wire
- [ ] Claims carry a falsifier
- [ ] Reactions not counted as attestations
- [ ] Secrets excluded from `Debug`/log rendering
- [ ] Writes verified by independent read-back
- [ ] Canonical JSON emits raw UTF-8 (Python: `ensure_ascii=False`)
