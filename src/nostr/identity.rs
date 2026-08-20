// ifascript/src/nostr/identity.rs
//! NIP-06 secp256k1 identity for IfáScript agents.
//!
//! # Where identity comes from
//!
//! ỌMỌ KỌ́DÀ owns agent birth: a BIPON39 mnemonic seeds both the kernel's
//! Ed25519 identity and — through BIPON39's `DerivationMode::Bip32` branch —
//! the secp256k1 Nostr identity at the standard NIP-06 path
//! `m/44'/1237'/<account>'/0/0`. One seed of truth, one backup covers both.
//!
//! IfáScript governs an agent's daily life; it does not birth agents. So the
//! load-bearing constructor here is [`NostrIdentity::from_secret_bytes`] —
//! **accept** the key ỌMỌ KỌ́DÀ already derived rather than re-deriving it.
//! Re-deriving would mean reimplementing BIPON39 and risking a silent split
//! where the same agent signs under two different pubkeys.
//!
//! [`NostrIdentity::from_mnemonic`] exists for standalone agents with a plain
//! BIP-39 mnemonic and no ỌMỌ KỌ́DÀ kernel. It walks the same NIP-06 path, so a
//! BIP-39 mnemonic yields the identical key any Nostr client would derive —
//! but a BIPON39 mnemonic must come in through `from_secret_bytes`, because
//! BIPON39's seed derivation is its own and is not BIP-39.

use nostr::nips::nip06::FromMnemonic;
use nostr::nips::nip19::ToBech32;
use nostr::{Keys, SecretKey};

/// An agent's Nostr identity: signing keys plus their bech32 presentations.
///
/// `nsec` and the underlying secret are sensitive. They are deliberately
/// excluded from the [`std::fmt::Debug`] rendering so a stray `{:?}` in a log
/// line cannot leak an agent's private key.
#[derive(Clone)]
pub struct NostrIdentity {
    keys: Keys,
    public_key_hex: String,
    npub: String,
}

impl std::fmt::Debug for NostrIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NostrIdentity")
            .field("npub", &self.npub)
            .field("public_key_hex", &self.public_key_hex)
            .field("secret", &"<redacted>")
            .finish()
    }
}

impl NostrIdentity {
    /// Adopt an already-derived secp256k1 secret key — the ỌMỌ KỌ́DÀ path.
    ///
    /// `secret_bytes` is the 32-byte output of that kernel's NIP-06 derivation.
    pub fn from_secret_bytes(secret_bytes: &[u8]) -> Result<Self, String> {
        let secret_key = SecretKey::from_slice(secret_bytes)
            .map_err(|e| format!("invalid secp256k1 secret key: {e}"))?;
        Ok(Self::from_keys(Keys::new(secret_key)))
    }

    /// Adopt an identity from its bech32 `nsec`.
    pub fn from_nsec(nsec: &str) -> Result<Self, String> {
        let keys = Keys::parse(nsec).map_err(|e| format!("invalid nsec: {e}"))?;
        Ok(Self::from_keys(keys))
    }

    /// Derive from a **BIP-39** mnemonic at NIP-06 path `m/44'/1237'/<account>'/0/0`.
    ///
    /// For standalone agents only. A BIPON39 mnemonic will *not* produce the
    /// agent's real key here — route that through [`Self::from_secret_bytes`].
    pub fn from_mnemonic(mnemonic: &str, account: u32) -> Result<Self, String> {
        let keys = Keys::from_mnemonic_with_account(mnemonic, None, Some(account))
            .map_err(|e| format!("NIP-06 derivation failed: {e}"))?;
        Ok(Self::from_keys(keys))
    }

    /// Generate a fresh, random identity. Test and demo use.
    pub fn generate() -> Self {
        Self::from_keys(Keys::generate())
    }

    fn from_keys(keys: Keys) -> Self {
        let public_key_hex = keys.public_key().to_hex();
        // A valid keypair always bech32-encodes; fall back to hex rather than
        // panicking on the theoretically-unreachable branch.
        let npub = keys
            .public_key()
            .to_bech32()
            .unwrap_or_else(|_| public_key_hex.clone());
        Self {
            keys,
            public_key_hex,
            npub,
        }
    }

    /// x-only public key as hex — the `pubkey` field of every event this agent signs.
    pub fn public_key_hex(&self) -> &str {
        &self.public_key_hex
    }

    /// Public key in bech32 `npub1…` form.
    pub fn npub(&self) -> &str {
        &self.npub
    }

    /// Secret key in bech32 `nsec1…` form. Never log this.
    pub fn nsec(&self) -> Result<String, String> {
        self.keys
            .secret_key()
            .map_err(|e| format!("secret key unavailable: {e}"))?
            .to_bech32()
            .map_err(|e| format!("nsec bech32 encoding failed: {e}"))
    }

    /// Raw secret key bytes — for deriving the NIP-44 conversation key that
    /// HMACs engram slugs.
    pub fn secret_bytes(&self) -> Result<[u8; 32], String> {
        let sk = self
            .keys
            .secret_key()
            .map_err(|e| format!("secret key unavailable: {e}"))?;
        Ok(sk.secret_bytes())
    }

    /// The signing keys, for event construction.
    pub fn keys(&self) -> &Keys {
        &self.keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The BIP-39 test mnemonic NIP-06 itself uses as its worked example.
    const NIP06_VECTOR: &str =
        "leader monkey parrot ring guide accident before fence cannon height naive bean";
    /// NIP-06's published expected pubkey for that mnemonic at account 0.
    const NIP06_EXPECTED_PUBKEY: &str =
        "17162c921dc4d2518f9a101db33695df1afb56ab82f5ff3e5da6eec3ca5cd917";

    #[test]
    fn matches_the_published_nip06_test_vector() {
        // Derivation is only useful if it agrees with every other Nostr client.
        // This pins that against the vector in the NIP itself, rather than
        // merely checking we agree with ourselves.
        let id = NostrIdentity::from_mnemonic(NIP06_VECTOR, 0).unwrap();
        assert_eq!(id.public_key_hex(), NIP06_EXPECTED_PUBKEY);
    }

    #[test]
    fn derivation_is_deterministic() {
        let a = NostrIdentity::from_mnemonic(NIP06_VECTOR, 0).unwrap();
        let b = NostrIdentity::from_mnemonic(NIP06_VECTOR, 0).unwrap();
        assert_eq!(a.public_key_hex(), b.public_key_hex());
        assert_eq!(a.npub(), b.npub());
    }

    #[test]
    fn different_accounts_are_different_identities() {
        let a = NostrIdentity::from_mnemonic(NIP06_VECTOR, 0).unwrap();
        let b = NostrIdentity::from_mnemonic(NIP06_VECTOR, 1).unwrap();
        assert_ne!(a.public_key_hex(), b.public_key_hex());
    }

    #[test]
    fn adopting_a_kernel_derived_key_round_trips() {
        // The ỌMỌ KỌ́DÀ path: the kernel hands over 32 raw bytes.
        let original = NostrIdentity::generate();
        let bytes = original.secret_bytes().unwrap();
        let adopted = NostrIdentity::from_secret_bytes(&bytes).unwrap();
        assert_eq!(original.public_key_hex(), adopted.public_key_hex());
    }

    #[test]
    fn nsec_round_trips_to_the_same_identity() {
        let original = NostrIdentity::generate();
        let adopted = NostrIdentity::from_nsec(&original.nsec().unwrap()).unwrap();
        assert_eq!(original.public_key_hex(), adopted.public_key_hex());
    }

    #[test]
    fn bech32_prefixes_are_correct() {
        let id = NostrIdentity::generate();
        assert!(id.npub().starts_with("npub1"), "npub: {}", id.npub());
        assert!(id.nsec().unwrap().starts_with("nsec1"));
    }

    #[test]
    fn pubkey_is_32_byte_x_only() {
        let id = NostrIdentity::generate();
        assert_eq!(hex::decode(id.public_key_hex()).unwrap().len(), 32);
    }

    #[test]
    fn a_wrong_length_secret_is_rejected() {
        assert!(NostrIdentity::from_secret_bytes(&[7u8; 31]).is_err());
        assert!(NostrIdentity::from_secret_bytes(&[]).is_err());
    }

    #[test]
    fn debug_rendering_does_not_leak_the_secret() {
        let id = NostrIdentity::generate();
        let rendered = format!("{id:?}");
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains(&id.nsec().unwrap()));
    }
}
