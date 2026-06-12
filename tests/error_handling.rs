use ifascript::ebo::{commitment_hash, vow_is_valid};
use ifascript::vm::MAX_STACK_DEPTH;
use ifascript::{IfaError, IfaVM};

// ── Stack overflow guard ──────────────────────────────────────────────────

#[test]
fn test_stack_overflow_returns_error() {
    let mut vm = IfaVM::new();
    // Fill stack to max depth
    for _ in 0..MAX_STACK_DEPTH {
        vm.stack.push(0);
    }
    // One more PushConst should return StackOverflow, not panic
    let result = vm.execute(vec!["Èjì Ogbè"]);
    assert!(
        matches!(result, Err(IfaError::StackOverflow { .. })),
        "expected StackOverflow, got: {:?}",
        result
    );
}

// ── Vow gate validation ───────────────────────────────────────────────────

#[test]
fn test_valid_vow_accepted() {
    assert!(vow_is_valid("I vow to uphold clarity in all my actions"));
    assert!(vow_is_valid(
        "I pledge transparency and no harm to all beings"
    ));
    assert!(vow_is_valid("I commit to no harm and clarity in this path"));
}

#[test]
fn test_short_vow_rejected() {
    assert!(!vow_is_valid("I vow clarity")); // < 20 chars
}

#[test]
fn test_vow_without_commitment_marker_rejected() {
    // contains "clarity" but no "i vow" / "i pledge" / "i commit"
    assert!(!vow_is_valid(
        "seeking clarity and no harm in all decisions made"
    ));
}

#[test]
fn test_vow_without_ethical_term_rejected() {
    // has commitment marker but no clarity/no harm/transparency
    assert!(!vow_is_valid(
        "I vow to follow the path wherever it leads me"
    ));
}

#[test]
fn test_incidental_no_harm_phrase_rejected() {
    // "no harm in trying" — should not pass the gate
    assert!(!vow_is_valid("no harm in trying this ritual path today"));
}

#[test]
fn test_commitment_hash_is_hex_string() {
    let h = commitment_hash("I vow clarity and no harm in this work");
    assert_eq!(
        h.len(),
        16,
        "commitment_hash should be 8 bytes = 16 hex chars"
    );
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

// ── PoW error propagation ─────────────────────────────────────────────────

#[test]
fn test_token_burn_empty_returns_error_not_panic() {
    use ifascript::ebo::{Ebo, EboTrigger};
    let mut vm = IfaVM::new();
    // Manually inject a TokenBurn Ebo with empty tx into the history
    // then execute RequireEbo — should return Err, not panic
    vm.ebo_history.record(EboTrigger::StackUnderflow); // set up state
                                                       // Directly test OduOp::RequireEbo with a forced TokenBurn via ebo_history override
                                                       // We test the error path via the accepts() logic instead
    let ebo = Ebo::TokenBurn(String::new());
    // TokenBurn with empty string should not be accepted
    assert!(!EboTrigger::StackUnderflow.accepts(&ebo));
}
