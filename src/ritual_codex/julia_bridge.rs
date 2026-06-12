/// Julia bridge stubs for future interop with Julia-based spiritual computation
/// These will be activated when Julia FFI is enabled.
use crate::ritual_codex::ResonancePacket;

#[derive(Debug)]
pub enum JuliaBridgeError {
    NotAvailable,
    SerializationError(String),
}

impl std::fmt::Display for JuliaBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JuliaBridgeError::NotAvailable => write!(f, "Julia bridge not available"),
            JuliaBridgeError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

/// Serialize a ResonancePacket to JSON for Julia ingestion
pub fn packet_to_julia_json(packet: &ResonancePacket) -> Result<String, JuliaBridgeError> {
    serde_json::to_string(packet).map_err(|e| JuliaBridgeError::SerializationError(e.to_string()))
}

/// Stub: call Julia resonance computation. Returns None until Julia FFI is wired up.
pub fn call_julia_resonance(_packet_json: &str) -> Option<f64> {
    None
}
