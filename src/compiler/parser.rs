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
    #[error("Invalid threshold number: {0}")]
    InvalidNumber(String),
    #[error("Missing ritual name in invocation")]
    MissingRitualName,
}

impl IfaParser {
    /// Parse a full .ifa program string — returns all invocations
    pub fn parse_program(input: &str) -> Result<Vec<ParsedInvocation>, ParseError> {
        let program = Self::parse(Rule::program, input)?
            .next()
            // SOI guarantees at least one pair (the program rule itself)
            .expect("program rule always present");

        let mut invocations = Vec::new();
        for pair in program.into_inner() {
            if pair.as_rule() == Rule::invocation {
                invocations.push(Self::parse_invocation(pair)?);
            }
        }
        Ok(invocations)
    }

    fn parse_invocation(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<ParsedInvocation, ParseError> {
        let mut ritual_name = None;
        let mut gate_principle = None;
        let mut gate_threshold = None;
        let mut witness_quorum = None;
        let mut sabbath = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::ident => ritual_name = Some(inner.as_str().to_string()),

                // gate_spec is a nested rule — extract principle and number from it
                Rule::gate_spec => {
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            Rule::principle => {
                                gate_principle = Some(part.as_str().to_string());
                            }
                            Rule::number => {
                                gate_threshold = Some(
                                    part.as_str()
                                        .parse::<f64>()
                                        .map_err(|_| ParseError::InvalidNumber(
                                            part.as_str().to_string(),
                                        ))?,
                                );
                            }
                            _ => {}
                        }
                    }
                }

                // Any number at the invocation level is the witness quorum
                Rule::number => {
                    witness_quorum = inner.as_str().parse::<u8>().ok();
                }

                Rule::sabbath_spec => {
                    let raw = inner.as_str();
                    // Strip surrounding quotes from string literals
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_invoke() {
        let input = "invoke thunder_justice;";
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ritual_name, "thunder_justice");
        assert_eq!(result[0].gate_principle, None);
        assert_eq!(result[0].gate_threshold, None);
        assert_eq!(result[0].witness_quorum, None);
        assert_eq!(result[0].sabbath, None);
    }

    #[test]
    fn test_parse_invoke_with_gate() {
        let input = "invoke thunder_justice with cause_effect:0.95;";
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result[0].ritual_name, "thunder_justice");
        assert_eq!(result[0].gate_principle, Some("cause_effect".into()));
        assert_eq!(result[0].gate_threshold, Some(0.95));
        assert_eq!(result[0].witness_quorum, None);
    }

    #[test]
    fn test_parse_invoke_with_witness() {
        let input = "invoke council_vote witness 5;";
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result[0].ritual_name, "council_vote");
        assert_eq!(result[0].gate_principle, None);
        assert_eq!(result[0].witness_quorum, Some(5));
    }

    #[test]
    fn test_parse_invoke_full() {
        let input = "invoke thunder_justice with cause_effect:0.95 witness 3 settle Saturday;";
        let result = IfaParser::parse_program(input).unwrap();
        let inv = &result[0];
        assert_eq!(inv.ritual_name, "thunder_justice");
        assert_eq!(inv.gate_principle, Some("cause_effect".into()));
        assert_eq!(inv.gate_threshold, Some(0.95));
        assert_eq!(inv.witness_quorum, Some(3));
        assert_eq!(inv.sabbath, Some("Saturday".into()));
    }

    #[test]
    fn test_parse_settle_any() {
        let input = "invoke ritual settle any;";
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result[0].sabbath, Some("any".into()));
    }

    #[test]
    fn test_parse_settle_string() {
        let input = r#"invoke ritual settle "custom day";"#;
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result[0].sabbath, Some("custom day".into()));
    }

    #[test]
    fn test_parse_multiple_invocations() {
        let input = "invoke ritual_a; invoke ritual_b with mentalism:0.8;";
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].ritual_name, "ritual_a");
        assert_eq!(result[1].gate_principle, Some("mentalism".into()));
    }

    #[test]
    fn test_parse_all_principles() {
        for principle in &[
            "mentalism", "correspondence", "vibration",
            "polarity", "rhythm", "cause_effect", "gender",
        ] {
            let input = format!("invoke ritual with {}:0.5;", principle);
            let result = IfaParser::parse_program(&input).unwrap();
            assert_eq!(result[0].gate_principle, Some(principle.to_string()));
        }
    }

    #[test]
    fn test_parse_empty_program() {
        let result = IfaParser::parse_program("").unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_with_comments() {
        let input = "// Cast Ṣàngó's justice\ninvoke thunder_justice;";
        let result = IfaParser::parse_program(input).unwrap();
        assert_eq!(result[0].ritual_name, "thunder_justice");
    }

    #[test]
    fn test_parse_error_missing_semicolon() {
        let input = "invoke ritual";
        assert!(IfaParser::parse_program(input).is_err());
    }
}
