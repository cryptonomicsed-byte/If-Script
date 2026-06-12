use thiserror::Error;

#[derive(Debug, Error)]
pub enum IfaError {
    #[error("Ebo unpaid: Token burn required — provide a non-empty transaction ID")]
    TokenBurnRequired,

    #[error("Ebo rejected: Vow insufficient — '{vow}'")]
    VowRejected { vow: String },

    #[error("Stack overflow: maximum stack depth ({max}) exceeded")]
    StackOverflow { max: usize },

    #[error("PoW nonce not found within attempt limit for difficulty {difficulty}")]
    PowExhausted { difficulty: u32 },

    #[error("Entropy source error: {0}")]
    EntropyError(String),
}
