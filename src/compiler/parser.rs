// ifascript/src/compiler/parser.rs
// Ògún's Forge: Minimal Parser — parses `invoke` statements

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::compiler::ast::{
    Definition, Literal, OduDef, Param, PrescriptionStmt, PrimitiveType, RitualDef, Statement,
    TypeExpr,
};

#[derive(Parser)]
#[grammar = "src/compiler/grammar.pest"]
pub struct IfaParser;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInvocation {
    pub ritual_name: String,
    pub gate_principle: Option<String>,
    pub gate_threshold: Option<f64>,
    pub witness_quorum: Option<u8>,
    pub sabbath: Option<String>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse failed: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Missing ritual name")]
    MissingRitualName,
}

impl IfaParser {
    /// Parse a full .ifa program string — returns all invocations
    pub fn parse_program(input: &str) -> Result<Vec<ParsedInvocation>, ParseError> {
        let program = Self::parse(Rule::program, input)?
            .next()
            .expect("program rule always present");

        program
            .into_inner()
            .filter(|p| p.as_rule() == Rule::invocation)
            .map(parse_invocation)
            .collect()
    }

    /// Parse a full program into typed definitions and raw invocations.
    /// Definitions (`odù …`, `ritual …`) populate the AST directly; invocations
    /// are returned raw for the caller to lower.
    #[allow(clippy::type_complexity)]
    pub fn parse_definitions(
        input: &str,
    ) -> Result<(Vec<Definition>, Vec<ParsedInvocation>), ParseError> {
        let program = Self::parse(Rule::program, input)?
            .next()
            .expect("program rule always present");

        let mut definitions = Vec::new();
        let mut invocations = Vec::new();
        for pair in program.into_inner() {
            match pair.as_rule() {
                Rule::odu_def => definitions.push(Definition::Odu(parse_odu_def(pair))),
                Rule::ritual_def => definitions.push(Definition::Ritual(parse_ritual_def(pair))),
                Rule::invocation => invocations.push(parse_invocation(pair)?),
                _ => {}
            }
        }
        Ok((definitions, invocations))
    }
}

fn unquote(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].replace("\\\"", "\"")
    } else {
        s.to_string()
    }
}

fn parse_literal(pair: pest::iterators::Pair<Rule>) -> Literal {
    let raw = pair.as_str().to_string();
    match pair.into_inner().next() {
        Some(p) => match p.as_rule() {
            Rule::string => Literal::Str(unquote(p.as_str())),
            Rule::number => Literal::Number(p.as_str().parse().unwrap_or(0.0)),
            Rule::ident => Literal::OduName(p.as_str().to_string()),
            _ => Literal::Str(p.as_str().to_string()),
        },
        None => Literal::Str(raw),
    }
}

fn parse_prescription(pair: pest::iterators::Pair<Rule>) -> PrescriptionStmt {
    let mut action = String::new();
    let mut args = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => action = inner.as_str().to_string(),
            Rule::literal_list => {
                for lit in inner.into_inner() {
                    if lit.as_rule() == Rule::literal {
                        args.push(parse_literal(lit));
                    }
                }
            }
            _ => {}
        }
    }
    PrescriptionStmt { action, args }
}

fn parse_odu_def(pair: pest::iterators::Pair<Rule>) -> OduDef {
    let mut name = String::new();
    let mut type_param = String::new();
    let mut prescriptions = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::odu_name => name = inner.as_str().to_string(),
            Rule::ident => type_param = inner.as_str().to_string(),
            Rule::prescription => prescriptions.push(parse_prescription(inner)),
            _ => {}
        }
    }
    OduDef {
        name,
        type_param,
        prescriptions,
    }
}

fn parse_type_expr(s: &str) -> TypeExpr {
    match s {
        "u8" => TypeExpr::Primitive(PrimitiveType::U8),
        "u16" => TypeExpr::Primitive(PrimitiveType::U16),
        "u32" => TypeExpr::Primitive(PrimitiveType::U32),
        "u64" => TypeExpr::Primitive(PrimitiveType::U64),
        "bool" => TypeExpr::Primitive(PrimitiveType::Bool),
        "string" => TypeExpr::Primitive(PrimitiveType::StringT),
        other => TypeExpr::Generic {
            name: other.to_string(),
            param: String::new(),
        },
    }
}

fn parse_param_list(pair: pest::iterators::Pair<Rule>) -> Vec<Param> {
    let mut params = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() != Rule::param {
            continue;
        }
        let mut name = String::new();
        let mut typ = TypeExpr::Primitive(PrimitiveType::StringT);
        for inner in p.into_inner() {
            match inner.as_rule() {
                Rule::ident => name = inner.as_str().to_string(),
                Rule::type_expr => typ = parse_type_expr(inner.as_str()),
                _ => {}
            }
        }
        params.push(Param { name, typ });
    }
    params
}

fn parse_ritual_def(pair: pest::iterators::Pair<Rule>) -> RitualDef {
    let mut name = String::new();
    let mut params = Vec::new();
    let mut body = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => name = inner.as_str().to_string(),
            Rule::param_list => params = parse_param_list(inner),
            Rule::prescription => body.push(Statement::Prescription(parse_prescription(inner))),
            _ => {}
        }
    }
    RitualDef {
        name,
        params,
        attributes: Vec::new(),
        body,
    }
}

fn parse_invocation(pair: pest::iterators::Pair<Rule>) -> Result<ParsedInvocation, ParseError> {
    let mut ritual_name = None;
    let mut gate_principle = None;
    let mut gate_threshold = None;
    let mut witness_quorum = None;
    let mut sabbath = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => ritual_name = Some(inner.as_str().to_string()),

            // hermetic_principle is a silent rule — it folds into gate_spec's span.
            // Split gate_spec's raw text on ':' to recover both parts.
            Rule::gate_spec => {
                let raw = inner.as_str();
                if let Some(colon) = raw.rfind(':') {
                    gate_principle = Some(raw[..colon].trim().to_string());
                    gate_threshold = raw[colon + 1..].trim().parse().ok();
                }
            }

            // witness_spec = { "witness" ~ number } — number is the only child
            Rule::witness_spec => {
                witness_quorum = inner
                    .into_inner()
                    .next()
                    .and_then(|p| p.as_str().parse().ok());
            }

            Rule::sabbath_spec => {
                let raw = inner.as_str();
                sabbath = Some(if raw.starts_with('"') && raw.ends_with('"') {
                    raw[1..raw.len() - 1].replace("\\\"", "\"")
                } else {
                    raw.to_string()
                });
            }

            _ => {}
        }
    }

    Ok(ParsedInvocation {
        ritual_name: ritual_name.ok_or(ParseError::MissingRitualName)?,
        gate_principle,
        gate_threshold,
        witness_quorum,
        sabbath,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<ParsedInvocation> {
        IfaParser::parse_program(s).expect("parse should succeed")
    }

    #[test]
    fn simple_invoke() {
        let r = parse("invoke thunder_justice;");
        assert_eq!(r[0].ritual_name, "thunder_justice");
        assert_eq!(r[0].gate_principle, None);
        assert_eq!(r[0].witness_quorum, None);
    }

    #[test]
    fn with_gate() {
        let r = parse("invoke t with cause_effect:0.95;");
        assert_eq!(r[0].gate_principle, Some("cause_effect".into()));
        assert_eq!(r[0].gate_threshold, Some(0.95));
    }

    #[test]
    fn with_witness_only() {
        // Witness without gate must parse correctly
        let r = parse("invoke t witness 3;");
        assert_eq!(r[0].witness_quorum, Some(3));
        assert_eq!(r[0].gate_principle, None);
    }

    #[test]
    fn full_invoke() {
        let r = parse("invoke t with cause_effect:0.95 witness 3 settle Saturday;");
        let i = &r[0];
        assert_eq!(i.ritual_name, "t");
        assert_eq!(i.gate_principle, Some("cause_effect".into()));
        assert_eq!(i.gate_threshold, Some(0.95));
        assert_eq!(i.witness_quorum, Some(3));
        assert_eq!(i.sabbath, Some("Saturday".into()));
    }

    #[test]
    fn keyword_as_ident_fails() {
        // Reserved keywords cannot be used as ritual names
        assert!(IfaParser::parse_program("invoke invoke;").is_err());
        assert!(IfaParser::parse_program("invoke witness;").is_err());
        assert!(IfaParser::parse_program("invoke settle;").is_err());
        assert!(IfaParser::parse_program("invoke ritual;").is_err());
    }

    #[test]
    fn multiple_invocations() {
        let r = parse("invoke alpha; invoke beta witness 2;");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].ritual_name, "alpha");
        assert_eq!(r[1].ritual_name, "beta");
        assert_eq!(r[1].witness_quorum, Some(2));
    }

    #[test]
    fn empty_program() {
        assert_eq!(parse("").len(), 0);
    }

    #[test]
    fn line_comment_skipped() {
        let r = parse("// Ṣàngó's justice\ninvoke thunder_justice;");
        assert_eq!(r[0].ritual_name, "thunder_justice");
    }

    #[test]
    fn block_comment_skipped() {
        let r = parse("/* opening */ invoke test;");
        assert_eq!(r[0].ritual_name, "test");
    }

    #[test]
    fn settle_any() {
        let r = parse("invoke r settle any;");
        assert_eq!(r[0].sabbath, Some("any".into()));
    }

    #[test]
    fn settle_quoted_string() {
        let r = parse(r#"invoke r settle "custom day";"#);
        assert_eq!(r[0].sabbath, Some("custom day".into()));
    }

    #[test]
    fn all_principles() {
        for p in &[
            "mentalism",
            "correspondence",
            "vibration",
            "polarity",
            "rhythm",
            "cause_effect",
            "gender",
        ] {
            let src = format!("invoke r with {}:0.5;", p);
            let result = parse(&src);
            assert_eq!(
                result[0].gate_principle.as_deref(),
                Some(*p),
                "failed for {}",
                p
            );
        }
    }

    #[test]
    fn missing_semicolon_fails() {
        assert!(IfaParser::parse_program("invoke ritual").is_err());
    }

    // ── definitions (v0.3) ──────────────────────────────────────────────

    #[test]
    fn parse_odu_definition() {
        let src = r#"
            odù Ogbe<dawn> {
                offer("coconut", "water");
                meditate;
            }
        "#;
        let (defs, invs) = IfaParser::parse_definitions(src).unwrap();
        assert!(invs.is_empty());
        assert_eq!(defs.len(), 1);
        match &defs[0] {
            Definition::Odu(o) => {
                assert_eq!(o.name, "Ogbe");
                assert_eq!(o.type_param, "dawn");
                assert_eq!(o.prescriptions.len(), 2);
                assert_eq!(o.prescriptions[0].action, "offer");
                assert_eq!(o.prescriptions[0].args.len(), 2);
                assert_eq!(o.prescriptions[1].action, "meditate");
            }
            other => panic!("expected Odu def, got {other:?}"),
        }
    }

    #[test]
    fn parse_ritual_definition_with_params() {
        let src = r#"
            ritual dawn_rite(agent: string, depth: u8) {
                light("candle");
                seal;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        assert_eq!(defs.len(), 1);
        match &defs[0] {
            Definition::Ritual(r) => {
                assert_eq!(r.name, "dawn_rite");
                assert_eq!(r.params.len(), 2);
                assert_eq!(r.params[0].name, "agent");
                assert_eq!(r.params[1].name, "depth");
                assert_eq!(r.body.len(), 2);
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }

    #[test]
    fn definitions_and_invocations_mix() {
        let src = r#"
            odù Oyeku<dusk> { release; }
            invoke dawn_rite with rhythm:0.8;
        "#;
        let (defs, invs) = IfaParser::parse_definitions(src).unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(invs.len(), 1);
        assert_eq!(invs[0].ritual_name, "dawn_rite");
        // The legacy invocation-only API still sees the invocation.
        assert_eq!(IfaParser::parse_program(src).unwrap().len(), 1);
    }
}
