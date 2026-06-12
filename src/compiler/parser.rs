// ifascript/src/compiler/parser.rs
// Ògún's Forge: Minimal Parser — parses `invoke` statements

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

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
}
