use ifascript::IfaVM;

#[test]
fn test_push_and_dup() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Èjì Ogbè", "Ìwòrì Méjì"]).unwrap();
    assert_eq!(vm.stack, vec![1, 1]);
}

#[test]
fn test_add() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Èjì Ogbè", "Èjì Ogbè", "Ìrosùn"]).unwrap();
    assert_eq!(vm.stack, vec![2]);
}

#[test]
fn test_swap() {
    let mut vm = IfaVM::new();
    vm.execute(vec!["Èjì Ogbè", "Ọ̀bàrà", "Ọ̀dí Méjì"]).unwrap();
    assert_eq!(vm.stack, vec![0, 1]);
}
