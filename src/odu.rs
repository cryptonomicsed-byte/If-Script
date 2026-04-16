use crate::vm::OduOp;

lazy_static::lazy_static! {
    pub static ref ODU_TABLE: std::collections::HashMap<&'static str, OduOp> = {
        let mut m = std::collections::HashMap::new();
        
        // Èjì Ogbè — Creation
        m.insert("Èjì Ogbè", OduOp::PushConst(1));
        
        // Ọ̀yẹ̀kú Méjì — Void
        m.insert("Ọ̀yẹ̀kú Méjì", OduOp::PopVoid);
        
        // Ìwòrì Méjì — Mirror
        m.insert("Ìwòrì Méjì", OduOp::Dup);
        
        // Ọ̀dí Méjì — Reversal
        m.insert("Ọ̀dí Méjì", OduOp::Swap);
        
        // Ìrosùn — Union
        m.insert("Ìrosùn", OduOp::Add);
        
        // Ọ̀wọ́nrín — Separation
        m.insert("Ọ̀wọ́nrín", OduOp::Sub);
        
        // Ọ̀bàrà — Provision
        m.insert("Ọ̀bàrà", OduOp::PushConst(0));
        
        // Ọ̀túúrúpọ̀n — Seal
        m.insert("Ọ̀túúrúpọ̀n", OduOp::HaltIfOne);

        // CastCowries — Entropy Oracle
        m.insert("CastCowries", OduOp::CastCowries);
        
        // Special Instructions
        m.insert("CastCowries", OduOp::CastCowries);
        
        // ... expand to 256
        
        m
    };
}
