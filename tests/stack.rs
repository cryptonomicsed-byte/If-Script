use ifascript::IfaVM;

#[test]
fn test_push_and_dup() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Genesis", "Attention"]).unwrap();
    assert_eq!(vm.stack, vec![1, 1]);
}

#[test]
fn test_add() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Genesis", "Genesis", "Receipt"]).unwrap();
    assert_eq!(vm.stack, vec![2]);
}

#[test]
fn test_swap() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Genesis", "Residue", "Loop"]).unwrap();
    assert_eq!(vm.stack, vec![0, 1]);
}
