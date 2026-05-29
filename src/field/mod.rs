use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::VecDeque;

// ── FieldEvent ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEvent {
    pub odu_id: u16,
    pub tier: u8,
    pub entropy_hash: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

impl FieldEvent {
    pub fn new(odu_id: u16, tier: u8, entropy_hash: String, payload: serde_json::Value) -> Self {
        FieldEvent {
            odu_id,
            tier,
            entropy_hash,
            timestamp: chrono::Utc::now(),
            payload,
        }
    }
}

// ── FieldBuffer ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct FieldBuffer {
    events: VecDeque<FieldEvent>,
    max_size: usize,
}

impl FieldBuffer {
    pub fn new(max_size: usize) -> Self {
        FieldBuffer {
            events: VecDeque::new(),
            max_size,
        }
    }

    pub fn push(&mut self, event: FieldEvent) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn drain(&mut self) -> Vec<FieldEvent> {
        self.events.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

// ── FieldHash ─────────────────────────────────────────────────────────────────

/// Compute a SHA3-256 field hash over a sequence of events
pub fn compute_field_hash(events: &[FieldEvent]) -> String {
    let mut hasher = Sha3_256::new();
    for event in events {
        hasher.update(event.odu_id.to_le_bytes());
        hasher.update([event.tier]);
        hasher.update(event.entropy_hash.as_bytes());
        hasher.update(event.timestamp.timestamp().to_le_bytes());
    }
    format!("0x{}", hex::encode(hasher.finalize()))
}

// ── FieldEngine ────────────────────────────────────────────────────────────────

pub struct FieldEngine {
    pub buffer: FieldBuffer,
    pub field_id: String,
    pub sequence: u64,
}

impl FieldEngine {
    pub fn new(field_id: impl Into<String>) -> Self {
        FieldEngine {
            buffer: FieldBuffer::new(1024),
            field_id: field_id.into(),
            sequence: 0,
        }
    }

    pub fn ingest(&mut self, event: FieldEvent) {
        self.sequence += 1;
        self.buffer.push(event);
    }

    pub fn flush(&mut self) -> Vec<FieldEvent> {
        self.buffer.drain()
    }

    pub fn current_hash(&self) -> String {
        let events: Vec<_> = self.buffer.events.iter().cloned().collect();
        compute_field_hash(&events)
    }

    /// Ingest a cosmogram state as a field event
    pub fn ingest_state(&mut self, state: &crate::cosmogram::CosmogramState) {
        let payload = serde_json::json!({
            "odu_id": state.odu_id,
            "tier": state.tier,
            "day": format!("{:?}", state.day),
            "window_open": state.window_open,
            "dominant_orisha": state.orisha_vector.dominant().map(|o| format!("{:?}", o)),
        });
        let event = FieldEvent::new(
            state.odu_id,
            state.tier,
            state.entropy_hash.clone(),
            payload,
        );
        self.ingest(event);
    }
}

impl Default for FieldEngine {
    fn default() -> Self {
        Self::new("default")
    }
}
