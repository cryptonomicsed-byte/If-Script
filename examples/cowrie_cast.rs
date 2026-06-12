use ifascript::IfaVM;

fn main() {
    let mut vm = IfaVM::with_intent("Ẹbọ: I seek clarity and truth from the sky");

    let _ = vm.execute(vec!["Èjì Ogbè", "CastCowries", "Ìwòrì Méjì", "Ọ̀túúrúpọ̀n"]);

    if vm.stack.len() >= 2 {
        let cowries = vm.stack[1] as u16;
        println!("Live NIST Beacon cowries cast: {:016b}", cowries);
        println!(
            "{} heads, {} tails → Branch: {:?}",
            cowries.count_ones(),
            16 - cowries.count_ones(),
            if cowries.count_ones() > 8 {
                "Ọ̀wọ́nrín Méjì"
            } else {
                "Èjì Ogbè"
            }
        );
    }

    println!("Àṣẹ sealed from atmospheric thunder.");
}
