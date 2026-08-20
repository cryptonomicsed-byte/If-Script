//! `enforcement-proportionate` — "the enforcement action matched the anomaly"
//!
//! Falsifies Zàngbétò's enforcement claim. Pure.
//!
//! ```json
//! manifest: {"observations": []}
//! inputs:   {"severity": "warning", "action": "quarantine_state", "signature_count": 2}
//! ```
//!
//! This is the falsifier with real content, because "the quarantine was
//! warranted" is precisely the assertion a guardian cannot be trusted to
//! self-certify. An immune system that can escalate without anyone able to
//! contest the escalation is not an immune system; it is an authority.
//!
//! It checks proportionality: each severity admits a ceiling of action, and an
//! action above that ceiling refutes the claim. The ladder mirrors
//! `action_ladder.rs`:
//!
//! | Severity | Highest action it justifies |
//! |---|---|
//! | info | observe |
//! | warning | flag_for_review |
//! | severe | quarantine_state |
//! | critical | rollback_transition |
//!
//! Under-reacting is *not* refuted. A guardian that observes where it could
//! have quarantined has not made a false claim about proportionality, and
//! punishing restraint would push the ladder in exactly the wrong direction.
//!
//! The most severe action additionally requires corroboration: a
//! `rollback_transition` carrying no Òrìṣà co-signature is refuted regardless
//! of severity, because unilateral reversal of another agent's state is the one
//! action whose damage cannot be undone by disagreeing with it afterwards.

#![no_std]
#![no_main]

use ifa_falsifiers::*;

/// Rank on the escalation ladder. Higher is more coercive.
fn action_rank(a: &[u8]) -> Option<i32> {
    if bytes_eq(a, b"observe") {
        Some(0)
    } else if bytes_eq(a, b"flag_for_review") {
        Some(1)
    } else if bytes_eq(a, b"quarantine_state") {
        Some(2)
    } else if bytes_eq(a, b"rollback_transition") {
        Some(3)
    } else {
        None
    }
}

/// Highest action rank a severity justifies.
fn severity_ceiling(s: &[u8]) -> Option<i32> {
    if bytes_eq(s, b"info") {
        Some(0)
    } else if bytes_eq(s, b"warning") {
        Some(1)
    } else if bytes_eq(s, b"severe") {
        Some(2)
    } else if bytes_eq(s, b"critical") {
        Some(3)
    } else {
        None
    }
}

#[no_mangle]
pub extern "C" fn crucible_falsify() {
    let Some(input) = inputs() else {
        return say(INDETERMINATE, "inputs absent or larger than this module can read");
    };

    let Some(sev) = json_str(input, "severity") else {
        return say(INDETERMINATE, "inputs declare no severity");
    };
    let Some(act) = json_str(input, "action") else {
        return say(INDETERMINATE, "inputs declare no action");
    };

    let (Some(ceiling), Some(rank)) = (severity_ceiling(sev), action_rank(act)) else {
        // An unrecognised rung is not a violation; it is a vocabulary this
        // module cannot judge. Abstain rather than refuse something that may
        // be perfectly proportionate under a ladder it has not been taught.
        return say(INDETERMINATE, "severity or action is outside the known ladder");
    };

    if rank > ceiling {
        return say(FAILS, "the action escalates beyond what the severity justifies");
    }

    // Irreversible action demands corroboration.
    if rank == 3 && json_milli(input, "signature_count").unwrap_or(0) < 1_000 {
        return say(FAILS, "rollback carries no corroborating signature");
    }

    say(HOLDS, "the action is within the ceiling its severity justifies")
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
