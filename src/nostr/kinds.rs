// ifascript/src/nostr/kinds.rs
//! The Nostr wire contract shared across the Ọ̀ṢỌ́VM / ỌMỌ KỌ́DÀ / VANTAGE
//! ecosystem.
//!
//! Independent repositories interoperate only insofar as they agree on event
//! kinds and tag names. This module is that agreement, written down once and
//! verified against the implementations that already ship it:
//!
//! | Kind    | Owner            | Verified against                                    |
//! |---------|------------------|-----------------------------------------------------|
//! | `22242` | NIP-42           | `minipae.py::KIND_AUTH`                             |
//! | `10002` | NIP-65           | `minipae.py::KIND_RELAY_LIST`                       |
//! | `30174` | NIP-AE / minipae | `minipae.py::KIND_AGENT_ENGRAM`                     |
//! | `47001` | Crucible CLAIM   | `crucible-core/src/kinds.rs::CLAIM`                 |
//! | `47002` | Crucible ATTEST  | `crucible-core/src/kinds.rs::ATTESTATION`           |
//! | `47004` | Crucible VERDICT | `crucible-core/src/kinds.rs::VERDICT`               |
//!
//! # Why IfáScript defines no kinds of its own
//!
//! The production Buzz relay enforces a kind allowlist in
//! `required_scope_for_kind()` (`buzz-relay/src/handlers/ingest.rs`) — a kind
//! with no match arm is rejected at ingest, **after** authentication, with
//! `restricted: unknown event kind`, which reads like an auth failure and is
//! not one.
//!
//! That allowlist is broad: it covers Buzz's own vocabulary (profiles, text
//! notes, reactions, deletions, gift wraps, stream messages, NIP-51 lists,
//! NIP-65 relay lists…), `30174`, and — added by the operational patch
//! recorded in minipae's `docs/D_2_2_RELAY_KIND_COMPATIBILITY.md` — the
//! `47000..48000` block. It does **not** cover an arbitrary new kind.
//!
//! So a fresh `kind:31xxx` for "ritual cast" would be silently unpublishable
//! on the one relay the ecosystem actually runs. A ritual cast travels as a
//! **minipae engram** (`30174`), and a ritual assertion that wants witness
//! consensus travels as a **Crucible claim** (`47001`). IfáScript borrows both
//! vocabularies rather than minting a third.
//!
//! `47000..48000` belongs to Crucible; IfáScript must never squat inside it.

/// NIP-42 relay authentication (ephemeral challenge/response).
pub const KIND_AUTH: u64 = 22242;

/// NIP-65 relay list metadata.
pub const KIND_RELAY_LIST: u64 = 10002;

/// NIP-25 reaction — used for witness voting on a ritual.
pub const KIND_REACTION: u64 = 7;

/// NIP-AE agent engram (addressable). The transport for ritual receipts.
pub const KIND_AGENT_ENGRAM: u64 = 30174;

/// Crucible: a falsifiable proposition entering the shared belief space.
pub const KIND_CLAIM: u64 = 47001;

/// Crucible: one agent independently executing a claim's falsifier.
pub const KIND_ATTESTATION: u64 = 47002;

/// Crucible: the resolution kernel's derived epistemic status for a claim.
pub const KIND_VERDICT: u64 = 47004;

/// The half-open kind block Crucible reserves. IfáScript reads these but never
/// mints a kind inside the range.
pub const CRUCIBLE_RESERVED: core::ops::Range<u64> = 47000..48000;

/// True when `kind` belongs to Crucible's reserved block.
pub fn is_crucible(kind: u64) -> bool {
    CRUCIBLE_RESERVED.contains(&kind)
}

/// True when `kind` is one IfáScript is allowed to publish under.
///
/// This is deliberately **not** a mirror of the relay's allowlist. That
/// allowlist is a large match arm covering most of Buzz's own vocabulary, and
/// a copy of it here would drift out of sync silently — asserting a
/// permissiveness this crate cannot actually verify. What this function states
/// is narrower and checkable: the four kinds IfáScript itself emits.
///
/// Anything outside this set is a bug in the caller, not a question about the
/// relay. A caller that needs a kind the relay accepts but IfáScript does not
/// emit (a text note, a long-form post) should use a client for that
/// vocabulary rather than widening this.
pub fn is_publishable(kind: u64) -> bool {
    matches!(
        kind,
        KIND_AGENT_ENGRAM | KIND_CLAIM | KIND_REACTION | KIND_AUTH
    )
}

/// Kinds the relay is known to reject outright — anything with no match arm in
/// `required_scope_for_kind`. Used to distinguish "IfáScript does not emit
/// this" from "the relay would refuse this from anyone".
pub fn relay_rejects(kind: u64) -> bool {
    // Conservative: only claims rejection for kinds outside every documented
    // Buzz range. Silence here means "unknown", never "accepted".
    !(kind <= 41
        || (1059..=1063).contains(&kind)
        || (10000..=10999).contains(&kind)
        || (20000..=29999).contains(&kind)
        || (30000..=39999).contains(&kind)
        || (40000..=46999).contains(&kind)
        || is_crucible(kind))
}

// === Engram slug namespace ===
//
// minipae addresses memory by slug, hashed into the `d` tag. IfáScript claims
// the `mem/ifa/` prefix; sibling systems use their own (`mem/ga/`,
// `mem/genteam/`) so a merged read view stays separable by origin.

/// Slug prefix for every engram IfáScript writes.
pub const SLUG_PREFIX: &str = "mem/ifa";

/// Slug for a single ritual cast receipt, keyed by receipt hash.
pub fn slug_cast(receipt_hash: &str) -> String {
    format!("{SLUG_PREFIX}/cast/{receipt_hash}")
}

/// Slug for a ritual invocation record, keyed by ritual name.
pub fn slug_ritual(ritual_name: &str) -> String {
    format!("{SLUG_PREFIX}/ritual/{ritual_name}")
}

/// Slug for an agent's current governance state (tier, vessel standing).
pub fn slug_state() -> String {
    format!("{SLUG_PREFIX}/state")
}

// === Tag names ===

/// Addressable-event identifier tag (NIP-33). Value is HMAC'd, never the raw slug.
pub const TAG_D: &str = "d";
/// Owner pubkey tag — who this engram is *for*, per NIP-AE.
pub const TAG_P: &str = "p";
/// Referenced event tag.
pub const TAG_E: &str = "e";
/// Odù index (0–255) the cast landed on.
pub const TAG_ODU: &str = "odu";
/// Action Vessel governing the cast.
pub const TAG_VESSEL: &str = "vessel";
/// Whether Kóòdù/Zàngbétò governance gates passed.
pub const TAG_GATES: &str = "gates";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crucible_block_is_respected() {
        assert!(is_crucible(KIND_CLAIM));
        assert!(is_crucible(KIND_ATTESTATION));
        assert!(is_crucible(47999));
        assert!(!is_crucible(48000));
        assert!(!is_crucible(KIND_AGENT_ENGRAM));
    }

    #[test]
    fn every_kind_ifascript_emits_is_publishable() {
        for k in [KIND_AGENT_ENGRAM, KIND_CLAIM, KIND_REACTION, KIND_AUTH] {
            assert!(is_publishable(k), "kind {k} is emitted but not publishable");
        }
    }

    #[test]
    fn a_kind_ifascript_does_not_emit_is_not_publishable() {
        // The guard exists so an invented kind fails at the call site rather
        // than at ingest, where the relay reports it as a post-auth rejection
        // that reads like an auth problem.
        assert!(!is_publishable(31337));
        assert!(!is_publishable(1), "text notes are the message layer's job");
    }

    #[test]
    fn publishable_does_not_claim_to_mirror_the_relay() {
        // The relay accepts far more than IfáScript emits -- kind 1, 30023,
        // 30315 and much of Buzz's vocabulary. is_publishable says nothing
        // about those; relay_rejects is the function that speaks to the relay,
        // and it must not call them rejected.
        for k in [1u64, 7, 30023, 30315, 30174] {
            assert!(!relay_rejects(k), "kind {k} is accepted by the relay");
        }
        assert!(relay_rejects(99999));
    }

    #[test]
    fn slugs_are_namespaced_to_ifa() {
        assert!(slug_cast("abc").starts_with("mem/ifa/"));
        assert!(slug_ritual("dawn").starts_with("mem/ifa/"));
        assert!(slug_state().starts_with("mem/ifa/"));
    }
}
