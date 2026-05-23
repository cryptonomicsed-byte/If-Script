// ifascript/src/compiler/mod.rs
// Compiler module — lexer → parser → AST → IR (Week 1: parser + AST)

pub mod ast;
pub mod parser;

pub use parser::{IfaParser, ParsedInvocation, ParseError};

use ast::{Invocation, GateSpec, HermeticPrinciple, SabbathSpec};

/// Lower a `ParsedInvocation` (from the raw parser) into a typed `ast::Invocation`
pub fn lower_invocation(parsed: ParsedInvocation) -> Invocation {
    let gate = parsed.gate_principle.and_then(|p| {
        let principle = HermeticPrinciple::from_str(&p)?;
        Some(GateSpec {
            principle,
            threshold: parsed.gate_threshold.unwrap_or(0.5),
        })
    });

    let sabbath = parsed.sabbath.map(|s| SabbathSpec::from_str(&s));

    Invocation {
        ritual_name: parsed.ritual_name,
        gate,
        witness_quorum: parsed.witness_quorum,
        sabbath,
    }
}

/// Parse a .ifa source string and return typed AST invocations
pub fn compile_invocations(source: &str) -> Result<Vec<Invocation>, ParseError> {
    let parsed = IfaParser::parse_program(source)?;
    Ok(parsed.into_iter().map(lower_invocation).collect())
}
