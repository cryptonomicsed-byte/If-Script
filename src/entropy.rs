use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
use {reqwest::blocking::Client, serde_json::Value};

pub struct NISTBeacon;

pub enum EntropySource {
    Atmospheric(NISTBeacon),
    Fallback(ChaCha20Rng),
}

pub struct CowrieOracle {
    source: EntropySource,
    buffer: VecDeque<u32>,
    ritual_seed: [u8; 32],
    fallback_rng: ChaCha20Rng,
    last_fetch: Instant,
    fetch_interval: Duration,
    #[cfg(not(target_arch = "wasm32"))]
    client: Client,
}

impl CowrieOracle {
    pub fn new(ritual_intent: &str) -> Self {
        let seed: [u8; 32] = Sha256::digest(ritual_intent.as_bytes()).into();
        let fallback_rng = ChaCha20Rng::from_seed(seed);

        Self {
            source: EntropySource::Atmospheric(NISTBeacon),
            buffer: VecDeque::new(),
            ritual_seed: seed,
            fallback_rng,
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
        match &mut self.source {
            EntropySource::Atmospheric(_beacon) => {
                if self.buffer.is_empty() || self.last_fetch.elapsed() > self.fetch_interval {
                    self.refill_from_beacon();
                }

                if !self.buffer.is_empty() {
                    let val = self.buffer.pop_front().unwrap_or(0);
                    val ^ self.derive_mixing_word(0)
                } else {
                    self.fallback_u32()
                }
            }
            EntropySource::Fallback(_) => self.fallback_u32(),
        }
    }

    fn fallback_u32(&mut self) -> u32 {
        // Mix ChaCha20 output with all 32 seed bytes (via a derived word) and a
        // time-based nonce so that two agents with the same intent string produce
        // different sequences when the NIST Beacon is unavailable.
        let chacha_val = self.fallback_rng.next_u32();
        let time_nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        chacha_val ^ self.derive_mixing_word(time_nonce)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refill_from_beacon(&mut self) {
        let result = self
            .client
            .get("https://beacon.nist.gov/beacon/2.0/chain/1/pulse/last")
            .send();

        match result {
            Ok(resp) => {
                if let Ok(json) = resp.json::<Value>() {
                    if let Some(output) = json["pulse"]["outputValue"].as_str() {
                        if let Ok(bytes) = hex::decode(output) {
                            for chunk in bytes.chunks(4) {
                                if chunk.len() == 4 {
                                    let arr = [chunk[0], chunk[1], chunk[2], chunk[3]];
                                    self.buffer.push_back(u32::from_be_bytes(arr));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                log::warn!("NIST Beacon unavailable — using ritual fallback");
            }
        }

        self.last_fetch = Instant::now();
    }

    #[cfg(target_arch = "wasm32")]
    fn refill_from_beacon(&mut self) {
        // Network access in WASM requires async JS FFI; beacon fetch is a no-op
        // here so the fallback_rng path is used automatically.
        log::warn!("NIST Beacon fetch not supported in WASM — using ritual fallback");
        self.last_fetch = Instant::now();
    }

    /// Derive a 32-bit mixing word from all 32 bytes of the ritual seed plus an
    /// additional u32 tweak (e.g. a time nonce). Uses the full SHA-256 output
    /// rather than discarding 28 of 32 bytes as the prior XOR scheme did.
    fn derive_mixing_word(&self, tweak: u32) -> u32 {
        let mut hasher = Sha256::new();
        hasher.update(&self.ritual_seed);
        hasher.update(tweak.to_be_bytes());
        let hash = hasher.finalize();
        // Fold all 32 bytes into a single u32 via XOR of four 8-byte words
        let w0 = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let w1 = u32::from_be_bytes([hash[4], hash[5], hash[6], hash[7]]);
        let w2 = u32::from_be_bytes([hash[8], hash[9], hash[10], hash[11]]);
        let w3 = u32::from_be_bytes([hash[12], hash[13], hash[14], hash[15]]);
        let w4 = u32::from_be_bytes([hash[16], hash[17], hash[18], hash[19]]);
        let w5 = u32::from_be_bytes([hash[20], hash[21], hash[22], hash[23]]);
        let w6 = u32::from_be_bytes([hash[24], hash[25], hash[26], hash[27]]);
        let w7 = u32::from_be_bytes([hash[28], hash[29], hash[30], hash[31]]);
        w0 ^ w1 ^ w2 ^ w3 ^ w4 ^ w5 ^ w6 ^ w7
    }
}
