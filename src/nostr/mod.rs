// ifascript/src/nostr/mod.rs
//! Nostr integration — how an IfáScript agent's ritual life reaches the swarm.
//!
//! # The shape of the ecosystem
//!
//! Ọ̀ṢỌ́VM, ỌMỌ KỌ́DÀ and VANTAGE are the pillars; everything else connects to
//! them or extends them. What lets independent repositories connect at all is
//! not shared code — it is a shared **wire contract**: the same NIP set, the
//! same event kinds, the same tag names, the same identity derivation. That
//! contract lives in [`kinds`], written down and pinned by tests.
//!
//! IfáScript's place in it:
//!
//! - **ỌMỌ KỌ́DÀ births the agent.** BIPON39 seeds an Ed25519 kernel identity
//!   and, off the same seed, a secp256k1 Nostr identity at the NIP-06 path.
//!   IfáScript adopts that key ([`NostrIdentity::from_secret_bytes`]); it does
//!   not mint a second one.
//! - **IfáScript governs the agent's daily life.** Each ritual cast becomes a
//!   signed event: a receipt others can read, or a claim others can falsify.
//! - **Kóòdù and Zàngbétò gate it.** Whether governance passed rides on the
//!   event, so a reader can tell a sanctioned cast from an unsanctioned one.
//! - **Crucible decides what the swarm believes.** Agreement is discounted for
//!   redundancy, so twenty copies of one agent do not read as twenty witnesses.
//!
//! # NIP coverage
//!
//! | NIP | Role | Where |
//! |-----|------|-------|
//! | NIP-01 | event format, ids, signatures | [`events`] |
//! | NIP-06 | secp256k1 identity from seed | [`identity`] |
//! | NIP-25 | witness voting | [`events::witness_vote`] |
//! | NIP-42 | relay authentication | [`events::relay_auth`] |
//! | NIP-AE | portable memory (`kind:30174`) | [`events::cast_engram`] |
//!
//! # Scope of this module
//!
//! Event construction, signing, the interop contract and relay transport are
//! all implemented here and covered by tests.
//!
//! [`NostrGateway`] prepares and signs; [`relay::RelayConnection`] opens the
//! socket, completes NIP-42 auth, publishes, and reads back. The two are
//! separate because signing is pure and testable while transport is not — and
//! because a component that holds no keys still needs the transport half.

pub mod events;
pub mod identity;
pub mod kinds;
// Relay transport needs a real socket — native targets only, same gate as the
// blocking HTTP used elsewhere in this crate.
#[cfg(not(target_arch = "wasm32"))]
pub mod relay;

pub use events::{
    cast_engram, relay_auth, ritual_claim, witness_vote, CastReceipt, EventError, RitualClaim,
};
pub use identity::NostrIdentity;
#[cfg(not(target_arch = "wasm32"))]
pub use relay::{parse_frame, RelayConnection, RelayError, RelayMessage};

use nostr::Event;

/// A relay this agent speaks to, and whether NIP-42 auth has completed.
#[derive(Debug, Clone)]
pub struct Relay {
    pub url: String,
    /// Id of the accepted `kind:22242` auth event, once authenticated.
    pub auth_event_id: Option<String>,
}

impl Relay {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            auth_event_id: None,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_event_id.is_some()
    }
}

/// Holds an agent's identity and its relay set, and prepares the events a
/// ritual produces.
///
/// Transport is deliberately absent — see the module note. `prepare_*` returns
/// signed events for a caller (or a later transport layer) to publish.
#[derive(Debug, Clone)]
pub struct NostrGateway {
    identity: NostrIdentity,
    relays: Vec<Relay>,
}

impl NostrGateway {
    /// Build a gateway around an agent identity.
    pub fn new(identity: NostrIdentity) -> Self {
        Self {
            identity,
            relays: Vec::new(),
        }
    }

    /// Register a relay. Ignores a URL already present, so repeated
    /// configuration is idempotent.
    pub fn add_relay(&mut self, url: impl Into<String>) {
        let url = url.into();
        if !self.relays.iter().any(|r| r.url == url) {
            self.relays.push(Relay::new(url));
        }
    }

    pub fn relays(&self) -> &[Relay] {
        &self.relays
    }

    /// Record that a relay accepted this agent's NIP-42 auth.
    pub fn mark_authenticated(&mut self, url: &str, auth_event_id: impl Into<String>) {
        if let Some(relay) = self.relays.iter_mut().find(|r| r.url == url) {
            relay.auth_event_id = Some(auth_event_id.into());
        }
    }

    /// Sign the NIP-42 responses for every registered relay's challenge.
    pub fn prepare_auth(&self, challenge: &str) -> Result<Vec<(String, Event)>, EventError> {
        self.relays
            .iter()
            .map(|r| relay_auth(&self.identity, challenge, &r.url).map(|e| (r.url.clone(), e)))
            .collect()
    }

    /// Sign a cast receipt as an engram addressed to `owner_pubkey_hex`.
    pub fn prepare_cast_engram(
        &self,
        receipt: &CastReceipt,
        owner_pubkey_hex: &str,
    ) -> Result<Event, EventError> {
        cast_engram(&self.identity, receipt, owner_pubkey_hex)
    }

    /// Sign a falsifiable ritual claim for Crucible.
    pub fn prepare_claim(&self, claim: &RitualClaim) -> Result<Event, EventError> {
        ritual_claim(&self.identity, claim)
    }

    /// Sign a witness vote on another agent's event.
    pub fn prepare_vote(&self, target: &Event, endorse: bool) -> Result<Event, EventError> {
        witness_vote(&self.identity, target, endorse)
    }

    /// This agent's public key in bech32 `npub1…` form.
    pub fn npub(&self) -> &str {
        self.identity.npub()
    }

    /// This agent's x-only public key as hex.
    pub fn pubkey(&self) -> &str {
        self.identity.public_key_hex()
    }

    pub fn identity(&self) -> &NostrIdentity {
        &self.identity
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NostrGateway {
    /// Connect to every registered relay, authenticate, and publish `event`,
    /// reading it back from each before counting that relay a success.
    ///
    /// Returns `(succeeded, failures)`. Publishing is **not** all-or-nothing:
    /// one relay refusing an event does not mean another did, and reporting a
    /// partial success as total failure would be as wrong as the reverse. A
    /// caller that needs every relay to have the event must check that
    /// `failures` is empty.
    ///
    /// Each relay gets its own connection, opened and closed here. That is
    /// slower than a pooled connection and is the right default for an
    /// operation that happens once per ritual rather than in a loop.
    pub fn publish_everywhere(
        &mut self,
        event: &Event,
    ) -> (Vec<String>, Vec<(String, relay::RelayError)>) {
        let urls: Vec<String> = self.relays.iter().map(|r| r.url.clone()).collect();
        let mut succeeded = Vec::new();
        let mut failures = Vec::new();

        for url in urls {
            match self.publish_to(&url, event) {
                Ok(auth_id) => {
                    self.mark_authenticated(&url, auth_id);
                    succeeded.push(url);
                }
                Err(e) => failures.push((url, e)),
            }
        }
        (succeeded, failures)
    }

    /// Publish to one relay, verified by read-back. Returns the event id, which
    /// the caller records as proof this relay accepted an authenticated session.
    fn publish_to(&self, url: &str, event: &Event) -> Result<String, relay::RelayError> {
        let mut conn = relay::RelayConnection::connect_authenticated(url, &self.identity)?;
        conn.publish_verified(event)?;
        conn.close();
        Ok(event.id().to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::IfaVM;

    fn gateway() -> NostrGateway {
        NostrGateway::new(NostrIdentity::generate())
    }

    #[test]
    fn adding_the_same_relay_twice_is_idempotent() {
        let mut g = gateway();
        g.add_relay("wss://relay.example.com");
        g.add_relay("wss://relay.example.com");
        assert_eq!(g.relays().len(), 1);
    }

    #[test]
    fn a_relay_starts_unauthenticated_and_can_be_marked() {
        let mut g = gateway();
        g.add_relay("wss://relay.example.com");
        assert!(!g.relays()[0].is_authenticated());

        g.mark_authenticated("wss://relay.example.com", "evt1");
        assert!(g.relays()[0].is_authenticated());
    }

    #[test]
    fn marking_an_unknown_relay_is_a_no_op() {
        let mut g = gateway();
        g.add_relay("wss://a.example.com");
        g.mark_authenticated("wss://b.example.com", "evt1");
        assert!(!g.relays()[0].is_authenticated());
    }

    #[test]
    fn auth_is_prepared_for_every_registered_relay() {
        let mut g = gateway();
        g.add_relay("wss://a.example.com");
        g.add_relay("wss://b.example.com");

        let prepared = g.prepare_auth("challenge-1").unwrap();
        assert_eq!(prepared.len(), 2);
        for (_, event) in prepared {
            assert!(event.verify().is_ok());
        }
    }

    #[test]
    fn the_full_ritual_path_produces_verifiable_events() {
        // A cast happens, becomes an engram, and another agent witnesses it —
        // the end-to-end shape of a governed ritual reaching the swarm.
        let mut agent = gateway();
        agent.add_relay("wss://relay.example.com");

        let cast = IfaVM::new().cast_odu();
        let receipt = CastReceipt::from_cast(&cast, true).with_receipt_hash("r-1");

        let engram = agent.prepare_cast_engram(&receipt, agent.pubkey()).unwrap();
        assert!(engram.verify().is_ok());

        let witness = gateway();
        let vote = witness.prepare_vote(&engram, true).unwrap();
        assert!(vote.verify().is_ok());
        assert_ne!(vote.pubkey.to_hex(), engram.pubkey.to_hex());
    }

    #[test]
    fn gateway_exposes_the_identity_it_signs_with() {
        let id = NostrIdentity::generate();
        let expected = id.public_key_hex().to_string();
        let g = NostrGateway::new(id);
        assert_eq!(g.pubkey(), expected);
        assert!(g.npub().starts_with("npub1"));
    }
}
