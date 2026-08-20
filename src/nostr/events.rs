// ifascript/src/nostr/events.rs
//! Turning ritual execution into signed Nostr events.
//!
//! Three things an IfáScript agent publishes, each on the vocabulary its owner
//! already defined (see [`super::kinds`] for why IfáScript mints no kind of its
//! own):
//!
//! - **A cast receipt** — what the Odù said and whether governance passed —
//!   travels as a minipae engram (`kind:30174`), addressable and private.
//! - **A ritual claim** — an assertion the agent wants the swarm to check —
//!   travels as a Crucible claim (`kind:47001`), which Crucible resolves into
//!   a verdict weighted by *independent* witnesses rather than by volume.
//! - **A witness vote** — a NIP-25 reaction (`kind:7`).

use hmac::{Hmac, Mac};
use nostr::{Event, EventBuilder, EventId, Kind, Tag, Url};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::identity::NostrIdentity;
use super::kinds;
use crate::vm::CastResult;

type HmacSha256 = Hmac<Sha256>;

/// Hash an engram slug into its `d` tag value.
///
/// minipae HMACs the slug rather than publishing it, so a relay operator
/// learns that an agent wrote *something* without learning what it named it.
/// Same construction here (`HMAC-SHA256(key, slug)`, hex) so an IfáScript
/// engram is addressable by any minipae client holding the same key.
pub fn d_tag(slug: &str, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC-SHA256 accepts a key of any length");
    mac.update(slug.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// The body of a cast-receipt engram.
///
/// Serialised as the event content so any reader — a sibling agent, VANTAGE,
/// a Crucible prober — can reconstruct what the cast decided without needing
/// IfáScript's own types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CastReceipt {
    /// Raw Odù index (0–255).
    pub odu_index: u8,
    /// Action Vessel governing this cast.
    pub vessel: String,
    /// Canonical file domain for the vessel.
    pub file_domain: String,
    /// Universal English name of the cast Odù.
    pub universal_name: String,
    /// Ordered prescription steps.
    pub prescriptions: Vec<String>,
    /// Whether Kóòdù/Zàngbétò governance gates passed.
    pub gates_passed: bool,
    /// Hash of the underlying resonance receipt, when one was issued.
    pub receipt_hash: Option<String>,
}

impl CastReceipt {
    /// Build a receipt body from a VM cast.
    pub fn from_cast(cast: &CastResult, gates_passed: bool) -> Self {
        Self {
            odu_index: cast.index,
            vessel: format!("{:?}", cast.vessel),
            file_domain: cast.file_domain.to_string(),
            universal_name: cast.universal_name.to_string(),
            prescriptions: cast.prescriptions.iter().map(|s| s.to_string()).collect(),
            gates_passed,
            receipt_hash: None,
        }
    }

    /// Attach the Zàngbétò receipt hash that audited this cast.
    pub fn with_receipt_hash(mut self, hash: impl Into<String>) -> Self {
        self.receipt_hash = Some(hash.into());
        self
    }
}

/// A falsifiable assertion an agent puts into the shared belief space.
///
/// Crucible's one rule: you may not assert without saying how you could be
/// proven wrong. `falsifier` is the content address of the WASM predicate that
/// returns false if `statement` is false — Crucible rejects a claim without one
/// at parse time, so this field is not optional.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RitualClaim {
    /// The assertion, in prose.
    pub statement: String,
    /// Content address of the falsifier module.
    pub falsifier: String,
    /// Odù index that produced the assertion.
    pub odu_index: u8,
    /// Half-life in seconds, after which belief decays.
    pub half_life_secs: u64,
}

/// Errors from building or signing an event.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("event signing failed: {0}")]
    Signing(String),
    #[error("serialising event content failed: {0}")]
    Serialisation(String),
    #[error("invalid relay url: {0}")]
    RelayUrl(String),
    #[error("kind {0} is not one IfáScript publishes under")]
    KindNotAdmitted(u64),
}

/// Publish a cast receipt as a minipae engram (`kind:30174`).
///
/// `owner` is the pubkey the engram is *for* — usually the agent itself, but a
/// supervising ỌMỌ KỌ́DÀ steward when memory is held on the agent's behalf.
pub fn cast_engram(
    identity: &NostrIdentity,
    receipt: &CastReceipt,
    owner_pubkey_hex: &str,
) -> Result<Event, EventError> {
    let content =
        serde_json::to_string(receipt).map_err(|e| EventError::Serialisation(e.to_string()))?;

    let slug = kinds::slug_cast(
        receipt
            .receipt_hash
            .as_deref()
            .unwrap_or(&receipt.odu_index.to_string()),
    );
    let key = identity
        .secret_bytes()
        .map_err(EventError::Signing)?;

    let tags = vec![
        tag(kinds::TAG_D, &d_tag(&slug, &key))?,
        tag(kinds::TAG_P, owner_pubkey_hex)?,
        tag(kinds::TAG_ODU, &receipt.odu_index.to_string())?,
        tag(kinds::TAG_VESSEL, &receipt.vessel)?,
        tag(kinds::TAG_GATES, &receipt.gates_passed.to_string())?,
    ];

    build(identity, kinds::KIND_AGENT_ENGRAM, content, tags)
}

/// Publish a falsifiable ritual assertion as a Crucible claim (`kind:47001`).
pub fn ritual_claim(
    identity: &NostrIdentity,
    claim: &RitualClaim,
) -> Result<Event, EventError> {
    let content =
        serde_json::to_string(claim).map_err(|e| EventError::Serialisation(e.to_string()))?;

    let tags = vec![
        tag(kinds::TAG_ODU, &claim.odu_index.to_string())?,
        tag("falsifier", &claim.falsifier)?,
        tag("half_life", &claim.half_life_secs.to_string())?,
    ];

    build(identity, kinds::KIND_CLAIM, content, tags)
}

/// Vote on another agent's ritual event as a NIP-25 reaction (`kind:7`).
///
/// `"+"` endorses, `"-"` objects. This is a witness signal, *not* a Crucible
/// attestation: a reaction carries no evidence and no falsifier run, so the
/// resolution kernel must not count it as one. Use [`ritual_claim`] plus a real
/// attestation when the vote needs to bear epistemic weight.
pub fn witness_vote(
    identity: &NostrIdentity,
    target: &Event,
    endorse: bool,
) -> Result<Event, EventError> {
    let reaction = if endorse { "+" } else { "-" };
    EventBuilder::reaction(target, reaction)
        .to_event(identity.keys())
        .map_err(|e| EventError::Signing(e.to_string()))
}

/// Build a NIP-42 authentication response (`kind:22242`) for a relay challenge.
///
/// The Buzz relay refuses writes from unauthenticated connections, so this runs
/// before any publish.
pub fn relay_auth(
    identity: &NostrIdentity,
    challenge: &str,
    relay_url: &str,
) -> Result<Event, EventError> {
    let url = Url::parse(relay_url).map_err(|e| EventError::RelayUrl(e.to_string()))?;
    EventBuilder::auth(challenge, url)
        .to_event(identity.keys())
        .map_err(|e| EventError::Signing(e.to_string()))
}

/// Reference an event by id, for building a reply or attestation chain.
pub fn event_ref(id: EventId) -> Result<Tag, EventError> {
    tag(kinds::TAG_E, &id.to_hex())
}

fn tag(name: &str, value: &str) -> Result<Tag, EventError> {
    Tag::parse(vec![name, value]).map_err(|e| EventError::Serialisation(e.to_string()))
}

fn build(
    identity: &NostrIdentity,
    kind: u64,
    content: String,
    tags: Vec<Tag>,
) -> Result<Event, EventError> {
    // Fail here rather than at relay ingest. The relay rejects unknown kinds
    // *after* a successful auth, which reads as a confusing auth problem.
    if !kinds::is_publishable(kind) {
        return Err(EventError::KindNotAdmitted(kind));
    }
    EventBuilder::new(Kind::Custom(kind), content, tags)
        .to_event(identity.keys())
        .map_err(|e| EventError::Signing(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::IfaVM;

    fn a_cast() -> CastResult {
        IfaVM::new().cast_odu()
    }

    #[test]
    fn a_cast_engram_is_signed_and_verifies() {
        let id = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true);
        let event = cast_engram(&id, &receipt, id.public_key_hex()).unwrap();

        // Signature must verify against the agent's own key, or no downstream
        // reader can attribute the cast.
        assert!(event.verify().is_ok());
        assert_eq!(event.kind(), Kind::Custom(kinds::KIND_AGENT_ENGRAM));
        assert_eq!(event.pubkey.to_hex(), id.public_key_hex());
    }

    #[test]
    fn engram_content_round_trips() {
        let id = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true).with_receipt_hash("deadbeef");
        let event = cast_engram(&id, &receipt, id.public_key_hex()).unwrap();

        let decoded: CastReceipt = serde_json::from_str(event.content()).unwrap();
        assert_eq!(decoded, receipt);
    }

    #[test]
    fn engram_carries_the_d_and_p_tags_nip_ae_requires() {
        let id = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true);
        let event = cast_engram(&id, &receipt, id.public_key_hex()).unwrap();

        let names: Vec<String> = event
            .tags
            .iter()
            .filter_map(|t| t.as_vec().first().cloned())
            .collect();
        assert!(names.iter().any(|n| n == kinds::TAG_D), "missing d tag");
        assert!(names.iter().any(|n| n == kinds::TAG_P), "missing p tag");
    }

    #[test]
    fn the_raw_slug_never_appears_on_the_wire() {
        // The whole point of HMACing the slug: a relay operator sees the
        // ciphertext of the address, not "mem/ifa/cast/...".
        let id = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true).with_receipt_hash("abc123");
        let event = cast_engram(&id, &receipt, id.public_key_hex()).unwrap();

        let wire = serde_json::to_string(&event).unwrap();
        assert!(!wire.contains("mem/ifa/cast/abc123"));
    }

    #[test]
    fn d_tag_is_deterministic_and_key_dependent() {
        let slug = "mem/ifa/cast/x";
        assert_eq!(d_tag(slug, b"key-one"), d_tag(slug, b"key-one"));
        assert_ne!(d_tag(slug, b"key-one"), d_tag(slug, b"key-two"));
        assert_ne!(d_tag(slug, b"key-one"), d_tag("mem/ifa/cast/y", b"key-one"));
    }

    #[test]
    fn a_ritual_claim_is_a_crucible_claim() {
        let id = NostrIdentity::generate();
        let claim = RitualClaim {
            statement: "the dawn ritual completed within its vessel".into(),
            falsifier: "sha256:abc".into(),
            odu_index: 7,
            half_life_secs: 900,
        };
        let event = ritual_claim(&id, &claim).unwrap();
        assert_eq!(event.kind(), Kind::Custom(kinds::KIND_CLAIM));
        assert!(event.verify().is_ok());
    }

    #[test]
    fn a_witness_vote_is_a_nip25_reaction_pointing_at_its_target() {
        let author = NostrIdentity::generate();
        let witness = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true);
        let target = cast_engram(&author, &receipt, author.public_key_hex()).unwrap();

        let vote = witness_vote(&witness, &target, true).unwrap();
        assert_eq!(vote.kind(), Kind::Custom(kinds::KIND_REACTION));
        assert_eq!(vote.content(), "+");
        assert!(vote.verify().is_ok());

        // It must reference the event it is voting on, or it is unattributable.
        let wire = serde_json::to_string(&vote).unwrap();
        assert!(wire.contains(&target.id().to_hex()));
    }

    #[test]
    fn a_dissent_vote_is_distinguishable_from_an_endorsement() {
        let author = NostrIdentity::generate();
        let witness = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true);
        let target = cast_engram(&author, &receipt, author.public_key_hex()).unwrap();

        assert_eq!(witness_vote(&witness, &target, false).unwrap().content(), "-");
    }

    #[test]
    fn relay_auth_is_kind_22242() {
        let id = NostrIdentity::generate();
        let event = relay_auth(&id, "challenge-abc", "wss://relay.example.com").unwrap();
        assert_eq!(event.kind(), Kind::Custom(kinds::KIND_AUTH));
        assert!(event.verify().is_ok());
    }

    #[test]
    fn a_malformed_relay_url_is_rejected_before_signing() {
        let id = NostrIdentity::generate();
        assert!(matches!(
            relay_auth(&id, "c", "not a url"),
            Err(EventError::RelayUrl(_))
        ));
    }

    #[test]
    fn publishing_under_an_unadmitted_kind_fails_locally() {
        // Guards the failure the relay would otherwise report as a confusing
        // post-auth rejection.
        let id = NostrIdentity::generate();
        let err = build(&id, 31337, "{}".into(), vec![]).unwrap_err();
        assert!(matches!(err, EventError::KindNotAdmitted(31337)));
    }

    #[test]
    fn two_agents_produce_distinguishable_engrams_for_the_same_cast() {
        let a = NostrIdentity::generate();
        let b = NostrIdentity::generate();
        let receipt = CastReceipt::from_cast(&a_cast(), true).with_receipt_hash("same");

        let ea = cast_engram(&a, &receipt, a.public_key_hex()).unwrap();
        let eb = cast_engram(&b, &receipt, b.public_key_hex()).unwrap();

        assert_ne!(ea.id(), eb.id());
        // Same slug, different keys, so the addresses must differ too.
        assert_ne!(ea.tags, eb.tags);
    }
}
