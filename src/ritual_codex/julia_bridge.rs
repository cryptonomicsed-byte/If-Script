//! Julia bridge — interop with the Block Mesh Julia resonance/scoring layer.
//!
//! When `JULIA_URL` (or `OSUN_URL`) points at a running Julia service, a
//! `ResonancePacket` is POSTed to `{base}/mesh/resonance` and the computed score
//! is returned. Fail-open: returns `None` when no service is configured, on any
//! transport error, or on `wasm32` (no blocking HTTP there).

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

/// Serialize a ResonancePacket to JSON for Julia ingestion.
pub fn packet_to_julia_json(packet: &ResonancePacket) -> Result<String, JuliaBridgeError> {
    serde_json::to_string(packet).map_err(|e| JuliaBridgeError::SerializationError(e.to_string()))
}

/// Base URL of the Julia resonance service, or `None` when unconfigured.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn julia_base() -> Option<String> {
    let raw = std::env::var("JULIA_URL")
        .ok()
        .or_else(|| std::env::var("OSUN_URL").ok())?;
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Extract a resonance score from a Julia response: `{"resonance": x}`,
/// `{"score": x}`, or a bare number.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn parse_resonance(val: &serde_json::Value) -> Option<f64> {
    val.get("resonance")
        .and_then(serde_json::Value::as_f64)
        .or_else(|| val.get("score").and_then(serde_json::Value::as_f64))
        .or_else(|| val.as_f64())
}

/// Call the Julia resonance computation with a serialized packet. Returns the
/// computed score, or `None` when no Julia service is configured / reachable.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_julia_resonance(packet_json: &str) -> Option<f64> {
    let base = julia_base()?;
    let url = format!("{base}/mesh/resonance");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(packet_json.to_string())
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let val: serde_json::Value = resp.json().ok()?;
    parse_resonance(&val)
}

/// WASM build: no blocking HTTP — always `None` (fail-open).
#[cfg(target_arch = "wasm32")]
pub fn call_julia_resonance(_packet_json: &str) -> Option<f64> {
    None
}

/// Convenience: serialize a packet and request its Julia resonance score.
/// Fail-open — `None` if the packet can't be serialized or no service responds.
pub fn resonance_for_packet(packet: &ResonancePacket) -> Option<f64> {
    let json = packet_to_julia_json(packet).ok()?;
    call_julia_resonance(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_resonance_accepts_several_shapes() {
        assert_eq!(parse_resonance(&json!({"resonance": 0.7})), Some(0.7));
        assert_eq!(parse_resonance(&json!({"score": 0.5})), Some(0.5));
        assert_eq!(parse_resonance(&json!(0.9)), Some(0.9));
        assert_eq!(parse_resonance(&json!({"other": 1})), None);
    }

    #[test]
    fn packet_serializes_to_json() {
        use crate::cosmogram::Day;
        let packet = ResonancePacket::new(42, 2, Day::Wednesday, 0, "seek wisdom");
        let s = packet_to_julia_json(&packet).unwrap();
        assert!(s.contains("\"odu_id\":42"));
        assert!(s.contains("seek wisdom"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn call_is_noop_without_service() {
        // Neither JULIA_URL nor OSUN_URL is set in the test environment.
        std::env::remove_var("JULIA_URL");
        std::env::remove_var("OSUN_URL");
        assert!(call_julia_resonance("{}").is_none());
        assert!(julia_base().is_none());
    }
}
