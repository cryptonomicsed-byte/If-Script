//! `signal-resolved` — "the directional call was correct"
//!
//! Falsifies Loom's trading-signal claim. **Observational**: unlike the other
//! three, it cannot be settled from the claim alone, because whether a
//! prediction came true is a fact about the world and not about the record.
//!
//! ```json
//! manifest: {"observations": ["market:close"]}
//! inputs:   {"direction": "long", "entry": 142.5}
//! ```
//!
//! Its manifest grants exactly one observation, so a reviewer reads the entire
//! blast radius in one line — which is the whole argument for declaring
//! capabilities rather than trusting a module's description of itself.
//!
//! Being observational, it is exempt from the vacuity audit: `audit_vacuity`
//! skips impure falsifiers precisely because their output may legitimately
//! depend on what they observed rather than on their inputs.
//!
//! Note what it refuses to do. If the observation was never gathered it emits
//! `Indeterminate`, not `Fails`. A probe that could not see has not disagreed,
//! and a resolution kernel that treated blindness as refutation would let a
//! relay outage manufacture verdicts.

#![no_std]
#![no_main]

use ifa_falsifiers::*;

const OBS: &str = "market:close";

#[no_mangle]
pub extern "C" fn crucible_falsify() {
    let Some(input) = inputs() else {
        return say(INDETERMINATE, "inputs absent or larger than this module can read");
    };

    let Some(direction) = json_str(input, "direction") else {
        return say(INDETERMINATE, "inputs declare no direction");
    };
    let Some(entry) = json_milli(input, "entry") else {
        return say(INDETERMINATE, "inputs declare no entry price");
    };

    let mut buf = [0u8; 64];
    let close_bytes = match observe(OBS, &mut buf) {
        Err(()) => return say(INDETERMINATE, "market:close is longer than this module can read"),
        Ok(None) => return say(INDETERMINATE, "market:close was never gathered"),
        Ok(Some(v)) => v,
    };

    // The observation arrives as a bare number; reuse the numeric scanner by
    // pointing it at a synthetic key would need an allocator, so parse in place.
    let Some(close) = scan_milli(close_bytes) else {
        return say(INDETERMINATE, "market:close is not a number this module can read");
    };

    let correct = if bytes_eq(direction, b"long") {
        close > entry
    } else if bytes_eq(direction, b"short") {
        close < entry
    } else {
        return say(INDETERMINATE, "direction is neither long nor short");
    };

    if correct {
        say(HOLDS, "the close moved in the direction the signal called")
    } else {
        say(FAILS, "the close did not move in the direction the signal called")
    }
}

/// Parse a bare decimal number, scaled by 1000. Same fixed-point convention as
/// `json_milli`, so both sides of the comparison truncate identically.
fn scan_milli(b: &[u8]) -> Option<i64> {
    let mut j = 0usize;
    let neg = if b.first() == Some(&b'-') {
        j = 1;
        true
    } else {
        false
    };
    let mut whole: i64 = 0;
    let mut saw = false;
    while j < b.len() && b[j].is_ascii_digit() {
        whole = whole.checked_mul(10)?.checked_add((b[j] - b'0') as i64)?;
        saw = true;
        j += 1;
    }
    if !saw {
        return None;
    }
    let mut frac: i64 = 0;
    let mut scale = 100;
    if b.get(j) == Some(&b'.') {
        j += 1;
        while j < b.len() && b[j].is_ascii_digit() {
            if scale > 0 {
                frac += ((b[j] - b'0') as i64) * scale;
                scale /= 10;
            }
            j += 1;
        }
    }
    let v = whole.checked_mul(1000)?.checked_add(frac)?;
    Some(if neg { -v } else { v })
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
