//! `market-resolved` — "the market settled the way the claim said it would"
//!
//! **Observational**, like [`signal_resolved`], and for the same reason: whether
//! a prediction came true is a fact about the world, not about the record.
//!
//! ```json
//! manifest: {"observations": ["market:resolution"]}
//! inputs:   {"outcome": "yes", "venue": "limitless", "market": "eth-15m-up"}
//! ```
//!
//! # Why a resolving market is the strongest falsifier we have
//!
//! A backtest is a replayable simulation, and its author controls every input,
//! so a claim resting on one can be tuned until it flatters. A prediction
//! market **resolves**: at a stated time, adjudicated by a party the claimant
//! does not control, with other people's money on the other side. Nothing else
//! in this ecosystem currently produces ground truth with that property.
//!
//! That also makes the claim non-vacuous by construction. `audit_vacuity`
//! exempts impure falsifiers rather than testing them, so the exemption is a
//! concession, not a clean bill of health — a module could hide behind it and
//! assert nothing. This one cannot: its verdict tracks a settlement it had no
//! part in producing.
//!
//! # Reading the observation
//!
//! `market:resolution` arrives as the settled outcome, compared literally
//! against the claimed `outcome`. Both sides are matched byte-for-byte after
//! ASCII-lowercasing, so `"Yes"` and `"yes"` agree while `"yes"` and `"yes "`
//! do not — a venue that pads its strings is a venue whose format we have not
//! actually verified, and guessing at it here would manufacture agreement.
//!
//! Two settlement states are *not* refutations and must never be reported as
//! `Fails`:
//!
//! * **Unresolved.** The market has not settled yet. A claim whose deadline has
//!   not arrived has not been proven wrong.
//! * **Void / cancelled.** The venue declined to settle at all. The claimant
//!   was not wrong; the question was withdrawn.
//!
//! Both are `Indeterminate`, as is a resolution that was never gathered. A
//! resolution kernel that treated any of them as refutation would let a venue
//! outage — or a cancelled market — manufacture verdicts against agents.

#![no_std]
#![no_main]

use ifa_falsifiers::*;

const OBS: &str = "market:resolution";

/// Settlement states that mean "no answer", not "wrong answer".
const UNSETTLED: [&[u8]; 6] = [
    b"unresolved",
    b"pending",
    b"open",
    b"void",
    b"cancelled",
    b"canceled",
];

#[no_mangle]
pub extern "C" fn crucible_falsify() {
    let Some(input) = inputs() else {
        return say(INDETERMINATE, "inputs absent or larger than this module can read");
    };

    let Some(claimed) = json_str(input, "outcome") else {
        return say(INDETERMINATE, "inputs declare no outcome");
    };
    if claimed.is_empty() {
        return say(INDETERMINATE, "the claimed outcome is empty");
    }

    let mut buf = [0u8; 64];
    let settled = match observe(OBS, &mut buf) {
        Err(()) => return say(INDETERMINATE, "market:resolution is longer than this module can read"),
        Ok(None) => return say(INDETERMINATE, "market:resolution was never gathered"),
        Ok(Some(v)) => v,
    };
    if settled.is_empty() {
        return say(INDETERMINATE, "market:resolution is empty");
    }

    // Fold both sides once, into fixed buffers. No allocator here, so anything
    // longer than the buffer is refused rather than silently truncated — a
    // truncated comparison could report agreement between two different
    // outcomes that happen to share a prefix.
    let mut lhs = [0u8; 64];
    let mut rhs = [0u8; 64];
    let Some(claimed) = fold_ascii(claimed, &mut lhs) else {
        return say(INDETERMINATE, "the claimed outcome is longer than this module can compare");
    };
    let Some(settled) = fold_ascii(settled, &mut rhs) else {
        return say(INDETERMINATE, "market:resolution is longer than this module can compare");
    };

    // Check the venue's non-answers before comparing, so "void" never reads as
    // a mismatch with a claimed "yes".
    let mut i = 0;
    while i < UNSETTLED.len() {
        if bytes_eq(settled, UNSETTLED[i]) {
            return say(INDETERMINATE, "the market has not settled on an outcome");
        }
        i += 1;
    }

    if bytes_eq(claimed, settled) {
        say(HOLDS, "the market settled on the outcome the claim named")
    } else {
        say(FAILS, "the market settled on a different outcome than the claim named")
    }
}

/// ASCII-lowercase `src` into `out`, returning the written prefix.
///
/// `None` when `src` does not fit: refusing is the only safe answer without an
/// allocator, since a truncated comparison invents agreement.
fn fold_ascii<'a>(src: &[u8], out: &'a mut [u8]) -> Option<&'a [u8]> {
    if src.len() > out.len() {
        return None;
    }
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        out[i] = if c.is_ascii_uppercase() { c + 32 } else { c };
        i += 1;
    }
    Some(&out[..src.len()])
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
