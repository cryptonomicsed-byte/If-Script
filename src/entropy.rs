//! entropy — CowrieOracle: NIST Randomness Beacon + ChaCha20, mixed via HKDF.
//!
//! Entropy pipeline (RFC 5869):
//!   - Beacon path: each 512-bit NIST pulse is used as the HKDF salt with the
//!     ritual seed (SHA-256 of the intent string) as input keying material.
//!     The expanded output seeds a per-pulse ChaCha20 keystream, refreshed
//!     when the beacon publishes a new pulse (every 60 s).
//!   - Fallback path: when the beacon is unreachable, HKDF over the ritual
//!     seed salted with wall-clock nanoseconds + process id seeds a ChaCha20
//!     keystream, so two agents with the same intent string still diverge.
//!
//! WASM note: the beacon fetch is compiled out on wasm32 (no blocking HTTP),
//! and the fallback salt degrades to the ritual seed alone — deterministic
//! per intent. Host-provided entropy should be injected via the intent string
//! until an async fetch path exists.

use hkdf::Hkdf;
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
use {reqwest::blocking::Client, serde_json::Value};

pub struct CowrieOracle {
    ritual_seed: [u8; 32],
    /// ChaCha20 keystream seeded from HKDF(salt = beacon pulse, ikm = ritual seed).
    /// `None` until the first successful beacon fetch, or after a fetch failure.
    pulse_rng: Option<ChaCha20Rng>,
    /// Lazily-initialised fallback keystream, seeded from
    /// HKDF(salt = time ‖ pid, ikm = ritual seed) on first use.
    fallback_rng: Option<ChaCha20Rng>,
    last_fetch: Instant,
    fetch_interval: Duration,
    #[cfg(not(target_arch = "wasm32"))]
    client: Client,
}

impl CowrieOracle {
    pub fn new(ritual_intent: &str) -> Self {
        let seed: [u8; 32] = Sha256::digest(ritual_intent.as_bytes()).into();

        Self {
            ritual_seed: seed,
            pulse_rng: None,
            fallback_rng: None,
            last_fetch: Instant::now() - Duration::from_secs(61),
            fetch_interval: Duration::from_secs(60),
            #[cfg(not(target_arch = "wasm32"))]
            client: Client::new(),
        }
    }

    pub fn cast_cowries(&mut self) -> u16 {
        (self.next_u32() & 0xFFFF) as u16
    }

    fn next_u32(&mut self) -> u32 {
        if self.pulse_rng.is_none() || self.last_fetch.elapsed() > self.fetch_interval {
            self.refill_from_beacon();
        }

        match &mut self.pulse_rng {
            Some(rng) => rng.next_u32(),
            None => self.fallback_u32(),
        }
    }

    fn fallback_u32(&mut self) -> u32 {
        if self.fallback_rng.is_none() {
            let seed = hkdf_seed(
                &fallback_salt(),
                &self.ritual_seed,
                b"ifascript-fallback-v1",
            );
            self.fallback_rng = Some(ChaCha20Rng::from_seed(seed));
        }
        self.fallback_rng
            .as_mut()
            .expect("fallback_rng initialised above")
            .next_u32()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refill_from_beacon(&mut self) {
        let result = self
            .client
            .get("https://beacon.nist.gov/beacon/2.0/chain/1/pulse/last")
            .send();

        self.pulse_rng = None;
        if let Ok(resp) = result {
            if let Ok(json) = resp.json::<Value>() {
                if let Some(output) = json["pulse"]["outputValue"].as_str() {
                    if let Ok(pulse_bytes) = hex::decode(output) {
                        let seed =
                            hkdf_seed(&pulse_bytes, &self.ritual_seed, b"ifascript-pulse-v1");
                        self.pulse_rng = Some(ChaCha20Rng::from_seed(seed));
                    }
                }
            }
        }
        if self.pulse_rng.is_none() {
            log::warn!("NIST Beacon unavailable — using ritual fallback");
        }

        self.last_fetch = Instant::now();
    }

    #[cfg(target_arch = "wasm32")]
    fn refill_from_beacon(&mut self) {
        // Network access in WASM requires async JS FFI; beacon fetch is a no-op
        // here so the fallback keystream is used automatically.
        log::warn!("NIST Beacon fetch not supported in WASM — using ritual fallback");
        self.last_fetch = Instant::now();
    }
}

/// HKDF-SHA256 (RFC 5869): Extract with the given salt and input keying
/// material, then Expand with a domain-separation info string to a 32-byte
/// ChaCha20 seed.
fn hkdf_seed(salt: &[u8], ikm: &[u8; 32], info: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    okm
}

/// Wall-clock nanoseconds + process id, so identical intents diverge when the
/// beacon is down. On wasm32 neither source exists; the salt is empty and the
/// fallback is deterministic per intent (documented limitation).
#[cfg(not(target_arch = "wasm32"))]
fn fallback_salt() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut salt = Vec::with_capacity(20);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    salt.extend_from_slice(&now.as_nanos().to_be_bytes());
    salt.extend_from_slice(&std::process::id().to_be_bytes());
    salt
}

#[cfg(target_arch = "wasm32")]
fn fallback_salt() -> Vec<u8> {
    Vec::new()
}
