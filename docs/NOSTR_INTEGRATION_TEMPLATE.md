# Nostr Integration Checklist — [REPO_NAME]

Copy to `NOSTR_INTEGRATION.md` in the repo and fill in. Delete rows that do not
apply rather than leaving them unchecked forever — an unticked box that was
never relevant is indistinguishable from one that was forgotten.

## 0. Before anything

- [ ] Namespace registered in `minipae/NAMESPACES.md` **before the first
      write**. That file's own rules require it, and an unregistered namespace
      collides silently.
- [ ] Read `docs/NOSTR_WIRE_CONTRACT.md` §7 (serialization) and §3 (slug
      grammar). Those two are where implementations diverge without any error
      naming the cause.

## 1. Which implementation does this repo use?

**Do not write a new one.** One per language:

| Language | Module | Signs? |
|---|---|---|
| Python | `minipae.py` | yes |
| Rust | `ifascript::nostr`, `zangbeto_enforcement::nostr_bridge` | yes |
| TypeScript | `organism-core/bridge/nostr-wire` | no |
| Julia | `OSOVM/src/nostr_bridge.jl` | no |
| JavaScript | `Koodu/nostr-adapter.js` | no |
| anything else | call organism-core as a bridge | — |

- [ ] Dependency added
- [ ] Confirmed this repo adds **no** wire-format logic of its own

## 2. Identity

- [ ] Does this component need its own key?
  - **Usually no.** Inherit the agent's. A second key for one agent is
    indistinguishable on the wire from a second agent.
  - Infrastructure daemons are the exception (see `Zàngbétò::Guardian`).
- [ ] If it signs: does the key come from the birth layer, or is it derived
      here? Deriving twice is the failure this rule exists to prevent.
- [ ] **NIP-46 bunker instead of a local key?** Recommended for anything
      long-running or network-exposed — `minipae.Nip46Client`. The process then
      holds a transport key, not the identity.

## 3. Events emitted

| Event | Fires when | Kind | Labels (NIP-32) | Required tags |
|---|---|---|---|---|
| | | | | |

Kinds are **not** invented. Engram `30174` for something remembered; Crucible
`47001` for something asserted. Anything else needs an owner named in the
contract.

- [ ] Every emitted kind is in `is_publishable` for this repo's module
- [ ] Claims carry a falsifier (Crucible rejects them otherwise, at parse time)
- [ ] Nothing unfalsifiable is published as a claim — a self-report is an
      engram, not an assertion

## 4. Memory

- [ ] Slugs listed here: `mem/<ns>/...`
- [ ] Segments normalised (`build_slug` / `normalizeSlugSegment`) — free text,
      Yorùbá names and URIs all violate the grammar unnormalised
- [ ] Addressable slugs keyed so a later write does not silently replace an
      earlier one that still matters

## 5. Privacy

- [ ] Engram content is **NIP-44 ciphertext**. A plaintext body in an engram is
      a well-formed, correctly signed, fully public leak.
- [ ] `d` tag is the HMAC'd slug, never the raw slug
- [ ] Checked what a relay operator can infer from tags alone

## 6. Publishing

- [ ] Direct (`RelayConnection` / `minipae.publish_authenticated`) or via
      organism-core as bridge
- [ ] NIP-42 auth — the production relay refuses unauthenticated writes and
      reports it as a rejection that reads like something else
- [ ] **A publish response is not proof of storage.** Read back before
      reporting success (`publish_verified`)
- [ ] Rejections surface as rejections

## 7. Tests

- [ ] Event id pinned against the cross-language vector
      (`e24b148552d35adf425c92e2e701ee3be6b4c86dbfd5fa2cc84a4c922250ac3b` for
      `"Òrìṣà Ògún"` at `created_at=1700000000`, `kind=30174`, no tags,
      pubkey `"a"*64`)
- [ ] Slug output checked against `minipae.validate_slug`, not just its own idea
- [ ] Signature verifies
- [ ] Encryption round-trips
- [ ] **At least one assertion compared against a different implementation.**
      Every cross-repo defect found so far was invisible to the repo's own tests.

## 8. SUI

- [ ] Does this repo move value? If not, it should link no chain SDK at all.
- [ ] If it does: settlement behind an interface, Nostr still the record.
      See `docs/NOSTR_CORE_SUI_OPTIONAL.md`.

## 9. Docs

- [ ] README section listing events emitted and how to listen for them
