//! `gates-passed` — "the ritual's governance gates passed"
//!
//! Falsifies the claim IfáScript and Kóòdù attach to a governed ritual. Pure:
//! it is a closed computation over the claim's own declared inputs, so two
//! agents anywhere must get byte-identical output, and a disagreement is a
//! defect rather than a difference of opinion about the world.
//!
//! ```json
//! manifest: {"observations": []}
//! inputs:   {"gates_passed": true, "vessel": "Ogun", "odu_index": 7}
//! ```
//!
//! What it actually checks, and why that is not trivial: a claim asserting the
//! gates passed is refuted when the accompanying record says they did not.
//! That catches the failure worth catching — an agent asserting sanction it
//! did not receive — without pretending to re-run Kóòdù's window arithmetic,
//! which this module cannot see and must not guess at.
//!
//! An `odu_index` outside 0–255 refutes too: the record is then internally
//! inconsistent, and a claim resting on an impossible cast should not stand
//! merely because its boolean happened to say true.

#![no_std]
#![no_main]

use ifa_falsifiers::*;

#[no_mangle]
pub extern "C" fn crucible_falsify() {
    let Some(input) = inputs() else {
        return say(INDETERMINATE, "inputs absent or larger than this module can read");
    };

    let Some(passed) = json_bool(input, "gates_passed") else {
        return say(INDETERMINATE, "inputs declare no gates_passed boolean");
    };

    if !passed {
        return say(FAILS, "the record says the governance gates did not pass");
    }

    // A cast index outside the corpus makes the record incoherent, whatever
    // its boolean claims.
    if let Some(odu) = json_milli(input, "odu_index") {
        if !(0..=255_000).contains(&odu) {
            return say(FAILS, "odu_index is outside the 0-255 corpus");
        }
    }

    say(HOLDS, "the record reports the governance gates passed")
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // A panicking falsifier emits nothing, which the kernel reads as
    // inconclusive. That is the correct reading: a module that crashed did not
    // form a view.
    core::arch::wasm32::unreachable()
}
