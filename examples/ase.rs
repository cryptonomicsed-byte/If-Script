use ifascript::IfaVM;

fn main() {
    let mut vm = IfaVM::new();
    // → prints "Àṣẹ" and exits
    let _ = vm.execute(vec!["Genesis", "Attention", "Consent"]);
}
