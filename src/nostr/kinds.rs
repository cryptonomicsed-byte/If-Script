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

/// minipae's slug grammar, which every engram address must satisfy.
///
/// Verified against `minipae.py::validate_slug`: each `/`-separated segment
/// after `mem/` must be non-empty, at most 64 bytes, start with a lowercase
/// letter, digit or `_`, and contain only lowercase letters, digits, `_` and
/// `-`. The whole slug must be at most 255 bytes.
pub fn validate_slug(slug: &str) -> bool {
    if slug.len() > 255 || !slug.starts_with("mem/") {
        return false;
    }
    let rest = &slug[4..];
    if rest.is_empty() {
        return false;
    }
    rest.split('/').all(|part| {
        !part.is_empty()
            && part.len() <= 64
            && part
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    })
}

/// Fold one path segment into minipae's grammar.
///
/// IfáScript's ritual names are Yorùbá, and that vocabulary does not satisfy
/// the grammar above — capitals and diacritics are both rejected, so an
/// unnormalised name produces a slug `minipae.py::validate_slug` refuses, and
/// an engram no minipae client can address.
///
/// Normalising costs nothing that matters: the slug is HMAC'd into the `d` tag
/// before it reaches the wire, so it is an addressing key and never display
/// text. The Yorùbá name travels intact in the event content.
///
/// Returns `None` when nothing survives normalisation, so a caller fails here
/// rather than building an address that only breaks later.
pub fn normalize_slug_segment(segment: &str) -> Option<String> {
    let mut out = String::with_capacity(segment.len());
    let mut last_dash = false;

    for c in segment.chars() {
        // Combining marks are dropped rather than mapped, so `ọ́` folds toward
        // `o` instead of becoming a separator.
        if is_combining_mark(c) {
            continue;
        }
        let folded = fold_char(c);
        match folded {
            Some(f) => {
                out.push(f);
                last_dash = false;
            }
            None => {
                if !last_dash && !out.is_empty() {
                    out.push('-');
                    last_dash = true;
                }
            }
        }
    }

    let trimmed = out.trim_matches('-');
    let capped: String = trimmed.chars().take(64).collect();
    let capped = capped.trim_matches('-').to_string();
    if capped.is_empty() {
        None
    } else {
        Some(capped)
    }
}

fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x20D0..=0x20FF)
}

/// Map one character into the slug alphabet, or `None` if it is a separator.
///
/// Handles the Latin-1/Latin-Extended letters Yorùbá orthography uses in
/// precomposed form (`ọ` U+1ECD, `ẹ` U+1EB9, `ṣ` U+1E63, plus the accented
/// vowels), so `Ọ̀rúnmìlà` folds to `orunmila` rather than to dashes.
fn fold_char(c: char) -> Option<char> {
    if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' {
        return Some(c);
    }
    if c.is_ascii_uppercase() {
        return Some(c.to_ascii_lowercase());
    }
    let base = match c {
        'à'..='å' | 'À'..='Å' | 'ā' | 'Ā' => 'a',
        'è'..='ë' | 'È'..='Ë' | 'ē' | 'Ē' | 'ẹ' | 'Ẹ' => 'e',
        'ì'..='ï' | 'Ì'..='Ï' | 'ī' | 'Ī' => 'i',
        'ò'..='ö' | 'Ò'..='Ö' | 'ō' | 'Ō' | 'ọ' | 'Ọ' => 'o',
        'ù'..='ü' | 'Ù'..='Ü' | 'ū' | 'Ū' => 'u',
        'ṣ' | 'Ṣ' => 's',
        'ń' | 'Ń' => 'n',
        'ý' | 'ÿ' | 'Ý' => 'y',
        _ => return None,
    };
    Some(base)
}

/// Slug for a single ritual cast receipt, keyed by receipt hash.
///
/// Returns `None` if the hash cannot form a valid address.
pub fn slug_cast(receipt_hash: &str) -> Option<String> {
    let slug = format!("{SLUG_PREFIX}/cast/{}", normalize_slug_segment(receipt_hash)?);
    validate_slug(&slug).then_some(slug)
}

/// Slug for a ritual invocation record, keyed by ritual name.
///
/// Returns `None` if the name cannot form a valid address.
pub fn slug_ritual(ritual_name: &str) -> Option<String> {
    let slug = format!("{SLUG_PREFIX}/ritual/{}", normalize_slug_segment(ritual_name)?);
    validate_slug(&slug).then_some(slug)
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
        assert!(slug_cast("abc").unwrap().starts_with("mem/ifa/"));
        assert!(slug_ritual("dawn").unwrap().starts_with("mem/ifa/"));
        assert!(slug_state().starts_with("mem/ifa/"));
    }

    #[test]
    fn a_yoruba_ritual_name_still_produces_a_slug_minipae_accepts() {
        // minipae.py::validate_slug allows only [a-z0-9_-] per segment, so an
        // unnormalised Yorùbá name yields an engram no minipae client can
        // address. Normalising is free: the slug is HMAC'd before the wire and
        // the real name travels in the content.
        assert_eq!(
            slug_ritual("Ọ̀rúnmìlà-Ìwúre").unwrap(),
            "mem/ifa/ritual/orunmila-iwure"
        );
        assert!(validate_slug(&slug_ritual("Ọ̀rúnmìlà-Ìwúre").unwrap()));
    }

    #[test]
    fn slug_validation_matches_minipae_grammar() {
        assert!(validate_slug("mem/ifa/ritual/dawn"));
        assert!(!validate_slug("mem/ifa/ritual/Dawn"), "capitals rejected");
        assert!(!validate_slug("mem/ifa/ritual/ọjọ"), "diacritics rejected");
        assert!(!validate_slug("mem/ifa//dawn"), "empty segments rejected");
        assert!(!validate_slug("ifa/ritual"), "must start with mem/");
        assert!(!validate_slug(&format!("mem/ifa/{}", "x".repeat(65))));
    }

    #[test]
    fn normalisation_folds_diacritics_and_case() {
        assert_eq!(normalize_slug_segment("Ọ̀rúnmìlà").unwrap(), "orunmila");
        assert_eq!(normalize_slug_segment("Ògún").unwrap(), "ogun");
        assert_eq!(normalize_slug_segment("Ẹ̀ṣù").unwrap(), "esu");
        assert_eq!(normalize_slug_segment("Earth + Metal").unwrap(), "earth-metal");
    }

    #[test]
    fn a_segment_that_normalises_to_nothing_is_refused() {
        // Silently emitting an empty segment builds an invalid slug that only
        // fails later, in another client or at the relay.
        assert!(normalize_slug_segment("!!!").is_none());
        assert!(normalize_slug_segment("").is_none());
        assert!(slug_ritual("!!!").is_none());
    }
}
