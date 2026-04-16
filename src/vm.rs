use std::thread;
use sha2::{Digest, Sha256};
use crate::ebo::{EboHistory, EboTrigger, Ebo};
use crate::entropy::CowrieOracle;

pub type Stack = Vec<i32>;

#[derive(Clone)]
pub enum OduOp {
    PushConst(i32),
    PopVoid,
    Dup,
    Swap,
    Add,
    Sub,
    HaltIfOne,
    CastCowries,
    RequireEbo(EboTrigger),
}

impl OduOp {
    pub fn execute(&self, vm: &mut IfaVM) {
        match self {
            OduOp::PushConst(v) => vm.stack.push(*v),
            OduOp::PopVoid => { 
                if vm.stack.is_empty() {
                    // Auto-trigger ebo for underflow
                    OduOp::RequireEbo(EboTrigger::StackUnderflow).execute(vm);
                    return;  // Halt until paid
                }
                let _ = vm.stack.pop(); 
            }
            OduOp::Dup => if let Some(top) = vm.stack.last() { 
                vm.stack.push(*top); 
            }
            OduOp::Swap => if vm.stack.len() >= 2 {
                let b = vm.stack.pop().unwrap();
                let a = vm.stack.pop().unwrap();
                vm.stack.push(b);
                vm.stack.push(a); 
            }
            OduOp::Add => if vm.stack.len() >= 2 {
                let b = vm.stack.pop().unwrap();
                let a = vm.stack.pop().unwrap();
                vm.stack.push(a + b);
            }
            OduOp::Sub => if vm.stack.len() >= 2 {
                let b = vm.stack.pop().unwrap();
                let a = vm.stack.pop().unwrap();
                vm.stack.push(a - b);
            }
            OduOp::HaltIfOne => if vm.stack.last() == Some(&1) {
                println!("Àṣẹ");
                vm.halted = true;
            }
            OduOp::CastCowries => {
                let cast = vm.oracle.cast_cowries();
                vm.stack.push(cast as i32);
            }
            OduOp::RequireEbo(trigger) => {
                let required = vm.ebo_history.required_ebo(trigger);
                
                // Enforce sacrifice payment
                match &required {
                    Ebo::TimeDelay(d) => {
                        println!("Ebo required: {:?} delay", d);
                        thread::sleep(*d);
                    }
                    Ebo::ProofOfWork(diff) => {
                        println!("Ebo required: PoW({})", diff);
                        let nonce = find_pow_nonce(*diff);
                        println!("PoW nonce found: {}", nonce);
                    }
                    Ebo::TokenBurn(tx) => {
                        if tx.is_empty() { 
                            panic!("Ebo unpaid: Token burn required"); 
                        }
                        println!("Token burn verified: {}", tx);
                    }
                    Ebo::IntentionString(vow) => {
                        let required = vm.ebo_history.required_ebo(trigger); 
                        if !trigger.accepts(&required) { 
                            panic!("Ebo rejected: Vow insufficient — '{}'", vow); 
                        }
                        println!("Vow accepted: {}", vow);
                    }
                }
                
                vm.ebo_history.record(trigger.clone());
            }
        }
    }
}

pub struct IfaVM {
    pub stack: Stack,
    pub oracle: CowrieOracle,
    pub ebo_history: EboHistory,
    pub halted: bool,
}

impl IfaVM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            oracle: CowrieOracle::new("Default ritual intent"),
            ebo_history: EboHistory::new(),
            halted: false,
        }
    }

    pub fn with_intent(intent: &str) -> Self {
        Self {
            stack: Vec::new(),
            oracle: CowrieOracle::new(intent),
            ebo_history: EboHistory::new(),
            halted: false,
        }
    }

    pub fn execute(&mut self, program: Vec<&str>) {
        use crate::odu::ODU_TABLE;
        for odu_name in program {
            if self.halted {
                break;
            }
            if let Some(op) = ODU_TABLE.get(odu_name) {
                op.clone().execute(self);
            }
        }
    }
}

impl Default for IfaVM {
    fn default() -> Self {
        Self::new()
    }
}

fn find_pow_nonce(difficulty: u32) -> u64 {
    let mut nonce = 0u64;
    let max_attempts = 1_000_000u64;  // Safety limit

    while nonce < max_attempts {
        let hash_input = format!("ifascript_ebo_{}", nonce);
        let hash = Sha256::digest(hash_input.as_bytes());
        
        let mut leading_zeros = 0;
        for &byte in hash.as_slice() {
            if byte == 0 {
                leading_zeros += 8;
            } else {
                leading_zeros += byte.leading_zeros();
                break;
            }
        }

        if leading_zeros >= difficulty { 
            return nonce; 
        }

        nonce += 1;
    }

    println!("Warning: PoW max attempts reached");
    0
}
