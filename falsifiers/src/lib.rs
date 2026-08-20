//! Shared plumbing for Crucible falsifier modules.
//!
//! A falsifier answers one question about one claim: **is this false?** It runs
//! in `crucible-probe`'s sandbox with no allocator, no filesystem, no network,
//! and no imports beyond the five below. That austerity is the point — a
//! reviewer decides whether to run somebody else's falsifier by reading its
//! capability manifest, and a manifest is only meaningful if the module cannot
//! reach past it.
//!
//! # The host ABI
//!
//! Read out of `crucible-probe/src/sandbox.rs`, not from documentation:
//!
//! | Import | Meaning |
//! |---|---|
//! | `input_len() -> i32` | bytes of claim input available |
//! | `input_read(ptr)` | copy them to `ptr` |
//! | `observe_len(k, klen) -> i32` | length of observation `k`, or `-1` |
//! | `observe_read(k, klen, out)` | copy observation `k` to `out` |
//! | `emit(verdict, ptr, len)` | 1 holds, 2 fails, 3 indeterminate |
//!
//! Two host behaviours matter and are easy to get wrong:
//!
//! * **Denial is a trap, not a return value.** Asking for an observation the
//!   manifest does not grant kills the module. `-1` means *granted but never
//!   gathered* — a different thing entirely, and the reason a probe that
//!   cannot see must emit `Indeterminate` rather than guessing.
//! * **Returning without emitting is inconclusive**, not "holds". Every path
//!   must reach an `emit`.
//!
//! # Why inputs are scanned rather than parsed
//!
//! `audit_vacuity` hands the module `serde_json::to_vec(inputs)`, so inputs are
//! JSON text. A full parser would need an allocator and would dwarf the logic
//! it serves. These modules instead scan for a specific key at the top level
//! and read the literal after it — enough for flat claim inputs, and small
//! enough that the whole module stays auditable.
//!
//! The scanner is deliberately strict: anything it does not understand becomes
//! `None`, and every caller turns `None` into `Indeterminate`. A falsifier that
//! guesses when it cannot read is worse than one that abstains, because the
//! kernel weighs its verdict as evidence either way.

#![no_std]

pub mod abi {
    //! Raw host imports. Everything else in this crate is a safe wrapper.
    #[link(wasm_import_module = "crucible")]
    extern "C" {
        pub fn input_len() -> i32;
        pub fn input_read(ptr: *mut u8);
        pub fn observe_len(key: *const u8, key_len: i32) -> i32;
        pub fn observe_read(key: *const u8, key_len: i32, out: *mut u8);
        pub fn emit(verdict: i32, ptr: *const u8, len: i32);
    }
}

/// Verdict codes the host accepts. Anything else traps.
pub const HOLDS: i32 = 1;
pub const FAILS: i32 = 2;
pub const INDETERMINATE: i32 = 3;

/// Scratch space for inputs and observations. Fixed and static because there is
/// no allocator; a claim whose inputs exceed this is refused rather than
/// silently truncated, since a truncated JSON scan can find a key whose value
/// was cut off and read it wrong.
pub const BUF_LEN: usize = 8192;
static mut BUF: [u8; BUF_LEN] = [0; BUF_LEN];

/// Emit a verdict with an explanation and stop.
pub fn say(verdict: i32, msg: &str) {
    unsafe { abi::emit(verdict, msg.as_ptr(), msg.len() as i32) }
}

/// Read the claim's inputs into the static buffer.
///
/// Returns `None` when the inputs do not fit, which callers report as
/// indeterminate: a partial read is indistinguishable from a short document,
/// and scanning one would produce a confident answer about text nobody sent.
pub fn inputs() -> Option<&'static [u8]> {
    unsafe {
        let n = abi::input_len();
        if n < 0 || n as usize > BUF_LEN {
            return None;
        }
        let n = n as usize;
        abi::input_read(BUF.as_mut_ptr());
        Some(core::slice::from_raw_parts(BUF.as_ptr(), n))
    }
}

/// Read one observation into a caller-supplied buffer.
///
/// `Ok(None)` means the runner never gathered it — granted, but absent. Only
/// ever call this for a key the manifest grants: asking for anything else
/// traps and the module dies without a verdict.
pub fn observe<'a>(key: &str, out: &'a mut [u8]) -> Result<Option<&'a [u8]>, ()> {
    unsafe {
        let n = abi::observe_len(key.as_ptr(), key.len() as i32);
        if n < 0 {
            return Ok(None);
        }
        let n = n as usize;
        if n > out.len() {
            return Err(());
        }
        abi::observe_read(key.as_ptr(), key.len() as i32, out.as_mut_ptr());
        Ok(Some(&out[..n]))
    }
}

// === Minimal JSON scanning ===
//
// Not a parser. Finds `"key"` at any nesting depth outside a string literal and
// returns the literal that follows the colon. Flat claim inputs are the only
// shape these falsifiers declare, so depth tracking is not needed; string-aware
// skipping is, or a key appearing inside a value would be matched.

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }
    i
}

/// Byte index just past the colon following `"key"`, if present.
fn find_key(b: &[u8], key: &str) -> Option<usize> {
    let k = key.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            b'"' => {
                let start = i + 1;
                let mut j = start;
                // Walk the string, honouring escapes so a `\"` does not end it.
                while j < b.len() && b[j] != b'"' {
                    j += if b[j] == b'\\' { 2 } else { 1 };
                }
                if j <= b.len() && &b[start..j.min(b.len())] == k {
                    let mut c = skip_ws(b, j + 1);
                    if c < b.len() && b[c] == b':' {
                        c = skip_ws(b, c + 1);
                        return Some(c);
                    }
                }
                i = j + 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// `true`/`false` for `key`, or `None` if absent or not a boolean.
pub fn json_bool(b: &[u8], key: &str) -> Option<bool> {
    let i = find_key(b, key)?;
    if b[i..].starts_with(b"true") {
        Some(true)
    } else if b[i..].starts_with(b"false") {
        Some(false)
    } else {
        None
    }
}

/// String value for `key`, or `None` if absent or not a string.
///
/// Returned as raw bytes between the quotes: no unescaping, because these
/// falsifiers compare identifiers and hashes, which carry no escapes. A value
/// containing a backslash is therefore compared literally, which is correct for
/// equality and would be wrong for display.
pub fn json_str<'a>(b: &'a [u8], key: &str) -> Option<&'a [u8]> {
    let i = find_key(b, key)?;
    if b.get(i)? != &b'"' {
        return None;
    }
    let start = i + 1;
    let mut j = start;
    while j < b.len() && b[j] != b'"' {
        if b[j] == b'\\' {
            return None; // escapes unsupported; abstain rather than mis-compare
        }
        j += 1;
    }
    if j >= b.len() {
        return None;
    }
    Some(&b[start..j])
}

/// Numeric value for `key`, scaled by 1000 and truncated, or `None`.
///
/// Fixed-point because there is no float formatting here and comparisons on
/// confidence and severity only need three decimals. Truncation is toward zero
/// and is applied consistently on both sides of every comparison.
pub fn json_milli(b: &[u8], key: &str) -> Option<i64> {
    let i = find_key(b, key)?;
    let mut j = i;
    let neg = if b.get(j) == Some(&b'-') {
        j += 1;
        true
    } else {
        false
    };

    let mut whole: i64 = 0;
    let mut saw_digit = false;
    while j < b.len() && b[j].is_ascii_digit() {
        whole = whole.checked_mul(10)?.checked_add((b[j] - b'0') as i64)?;
        saw_digit = true;
        j += 1;
    }
    if !saw_digit {
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

/// Constant-time-ish byte equality. Not secret-dependent here, but comparing
/// hashes with early exit invites habit formation in the wrong direction.
pub fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}
