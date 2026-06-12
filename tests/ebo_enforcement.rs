use ifascript::{IfaVM, ebo::EboTrigger};

#[test]
fn test_stack_underflow_triggers_ebo() {
    let mut vm = IfaVM::new();
    let _ = vm.execute(vec!["Ọ̀yẹ̀kú Méjì"]);  // POP on empty stack

    assert!(vm.ebo_history.has_trigger(&EboTrigger::StackUnderflow));
}

#[test]
fn test_ebo_escalation() {
    let mut vm = IfaVM::new();

    // First 3 underflows trigger TimeDelay(1s)
    let _ = vm.execute(vec!["Ọ̀yẹ̀kú Méjì"]);
    let _ = vm.execute(vec!["Ọ̀yẹ̀kú Méjì"]);
    let _ = vm.execute(vec!["Ọ̀yẹ̀kú Méjì"]);

    // Fourth underflow escalates to PoW(20)
    let _ = vm.execute(vec!["Ọ̀yẹ̀kú Méjì"]);

    assert!(vm.ebo_history.has_trigger(&EboTrigger::StackUnderflow));
}
