//! `deterministic-execution` — "the job executed deterministically"
//!
//! Falsifies Ọ̀ṢỌ́VM's execution claim. Pure.
//!
//! ```json
//! manifest: {"observations": []}
//! inputs:   {"state_root": "ab12…", "replay_root": "ab12…", "deterministic": true}
//! ```
//!
//! Determinism is not something a VM can assert about itself credibly — the
//! claim "this ran deterministically" is exactly the claim an agent has every
//! incentive to make and no ability to prove alone. What is checkable is
//! narrower and useful: the declared post-state root and the root of an
//! independent replay must be the same string.
//!
//! So the module refutes on divergence and abstains when there is no replay to
//! compare against. Abstaining matters: a claim with no replay root is
//! unchecked, and reporting "holds" for it would let an agent earn belief by
//! omitting the evidence rather than by producing it.

#![no_std]
#![no_main]

use ifa_falsifiers::*;

#[no_mangle]
pub extern "C" fn crucible_falsify() {
    let Some(input) = inputs() else {
        return say(INDETERMINATE, "inputs absent or larger than this module can read");
    };

    let Some(declared) = json_str(input, "state_root") else {
        return say(INDETERMINATE, "inputs declare no state_root");
    };
    if declared.is_empty() {
        return say(FAILS, "state_root is empty");
    }

    let Some(replay) = json_str(input, "replay_root") else {
        // No independent replay was supplied, so nothing here has been
        // checked. Say so rather than crediting the claim.
        return say(INDETERMINATE, "no replay_root supplied; nothing to compare against");
    };

    if !bytes_eq(declared, replay) {
        return say(FAILS, "replay produced a different state root than the one declared");
    }

    // A record that asserts non-determinism while its roots agree is
    // internally inconsistent; refuse rather than resolve the contradiction.
    if json_bool(input, "deterministic") == Some(false) {
        return say(FAILS, "roots agree but the record declares the run non-deterministic");
    }

    say(HOLDS, "an independent replay reproduced the declared state root")
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
