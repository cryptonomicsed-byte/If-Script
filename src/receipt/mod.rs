// Canonical receipt hashing using SHA3-256 + JSON
use sha3::{Digest, Sha3_256};
use crate::ritual_codex::ResonanceReceipt;

pub fn hash_receipt_canonical(receipt: &ResonanceReceipt) -> Result<String, String> {
    let json = serde_json::to_string(receipt).map_err(|e| e.to_string())?;
    let mut hasher = Sha3_256::new();
    hasher.update(json.as_bytes());
    Ok(format!("0x{}", hex::encode(hasher.finalize())))
}
