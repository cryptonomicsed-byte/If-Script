use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use reqwest::blocking::Client;
use serde_json::Value;

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
    client: Client,
}

impl CowrieOracle {
    pub fn new(ritual_intent: &str) -> Self {
        let seed: [u8; 32] = Sha256::digest(ritual_intent.as_bytes()).into();
        let fallback_rng = ChaCha20Rng::from_seed(seed);
        let client = Client::new();

        Self {
            source: EntropySource::Atmospheric(NISTBeacon),
            buffer: VecDeque::new(),
            ritual_seed: seed,
            fallback_rng,
            last_fetch: Instant::now() - Duration::from_secs(61),
            fetch_interval: Duration::from_secs(60),
            client,
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
                    val ^ self.hash_seed()
                } else {
                    self.fallback_u32()
                }
            }
            EntropySource::Fallback(_) => self.fallback_u32() ^ self.hash_seed(),
        }
    }

    fn fallback_u32(&mut self) -> u32 {
        self.fallback_rng.next_u32()
    }


    fn refill_from_beacon(&mut self) {
        if let Ok(resp) = self.client.get("https://beacon.nist.gov/beacon/2.0/chain/1/pulse/last").send() {
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
        } else {
            // Silent fallback — buffer remains empty, next_u32 uses fallback_u32
            println!("NIST Beacon unavailable — using ritual fallback");
        }

        self.last_fetch = Instant::now();
    }

    fn hash_seed(&self) -> u32 {
        let hash = Sha256::digest(&self.ritual_seed);
        u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
    }
}
