// ifascript/src/compiler/mod.rs

pub mod ast;
pub mod parser;

pub use parser::{IfaParser, ParseError, ParsedInvocation};

use ast::{GateSpec, HermeticPrinciple, Invocation, SabbathSpec};

/// Lower a raw `ParsedInvocation` into a typed `ast::Invocation`
pub fn lower_invocation(parsed: ParsedInvocation) -> Invocation {
    let gate = parsed.gate_principle.and_then(|p| {
        let principle = HermeticPrinciple::parse(&p)?;
        Some(GateSpec {
            principle,
            threshold: parsed.gate_threshold.unwrap_or(0.5),
        })
    });
    Invocation {
        ritual_name: parsed.ritual_name,
        gate,
        witness_quorum: parsed.witness_quorum,
        sabbath: parsed.sabbath.map(|s| SabbathSpec::parse(&s)),
    }
}

/// Parse a .ifa source string and return typed AST invocations
pub fn compile_invocations(source: &str) -> Result<Vec<Invocation>, ParseError> {
    let parsed = IfaParser::parse_program(source)?;
    Ok(parsed.into_iter().map(lower_invocation).collect())
}

/// Parse a full .ifa source string into a typed [`ast::Program`] — `odù` and
/// `ritual` definitions plus invocations. This is how a manifesto can be
/// *written* in IfáScript rather than only backed by a raw Odù id.
pub fn compile_program(source: &str) -> Result<ast::Program, ParseError> {
    let (definitions, parsed_invocations) = IfaParser::parse_definitions(source)?;
    let invocations = parsed_invocations
        .into_iter()
        .map(lower_invocation)
        .collect();
    Ok(ast::Program {
        imports: Vec::new(),
        definitions,
        invocations,
    })
}
