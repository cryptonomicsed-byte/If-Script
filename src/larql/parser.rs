use pest::Parser;
use pest_derive::Parser;

use crate::larql::ast::{
    parse_vessel, Condition, DescribeQuery, LarqlQuery, OduRef, Operator, PrepareQuery, Scale,
    VerifyQuery,
};
use crate::larql::error::LarqlError;

#[derive(Parser)]
#[grammar = "src/larql/grammar.pest"]
pub struct LarqlParser;

/// Parse a LARQL query string into a typed [`LarqlQuery`].
pub fn parse_query(input: &str) -> Result<LarqlQuery, LarqlError> {
    let pair = LarqlParser::parse(Rule::larql_query, input)
        .map_err(|e| LarqlError::Parse(e.to_string()))?
        .next()
        .ok_or_else(|| LarqlError::Parse("empty query".into()))?;

    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| LarqlError::Parse("empty query".into()))?;

    match inner.as_rule() {
        Rule::describe_query => parse_describe(inner),
        Rule::verify_query => parse_verify(inner),
        Rule::prepare_query => parse_prepare(inner),
        other => Err(LarqlError::Parse(format!(
            "unexpected top-level rule: {other:?}"
        ))),
    }
}

fn unquote(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].replace("\\\"", "\"")
    } else {
        s.to_string()
    }
}

fn parse_target(pair: pest::iterators::Pair<Rule>) -> Result<OduRef, LarqlError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| LarqlError::Parse("missing DESCRIBE target".into()))?;
    match inner.as_rule() {
        Rule::number => inner
            .as_str()
            .parse::<u8>()
            .map(OduRef::Index)
            .map_err(|_| LarqlError::Parse(format!("index out of range: {}", inner.as_str()))),
        Rule::string => Ok(OduRef::Name(unquote(inner.as_str()))),
        other => Err(LarqlError::Parse(format!(
            "unexpected target rule: {other:?}"
        ))),
    }
}

fn parse_scale_list(pair: pest::iterators::Pair<Rule>) -> Vec<Scale> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::scale)
        .map(|p| match p.as_str() {
            "meso" => Scale::Meso,
            "macro" => Scale::Macro,
            _ => Scale::Micro,
        })
        .collect()
}

fn parse_describe(pair: pest::iterators::Pair<Rule>) -> Result<LarqlQuery, LarqlError> {
    let mut target = None;
    let mut scales = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::target => target = Some(parse_target(inner)?),
            Rule::scale_list => scales = parse_scale_list(inner),
            _ => {}
        }
    }
    if scales.is_empty() {
        scales.push(Scale::Micro);
    }
    Ok(LarqlQuery::Describe(DescribeQuery {
        target: target.ok_or_else(|| LarqlError::Parse("missing DESCRIBE target".into()))?,
        scales,
    }))
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition, LarqlError> {
    let mut field = None;
    let mut operator = None;
    let mut value = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::field => field = Some(inner.as_str().to_string()),
            Rule::operator => {
                operator = Some(match inner.as_str() {
                    "CONTAINS" => Operator::Contains,
                    _ => Operator::Eq,
                })
            }
            Rule::value => {
                let raw = inner
                    .into_inner()
                    .next()
                    .ok_or_else(|| LarqlError::Parse("empty value".into()))?;
                value = Some(match raw.as_rule() {
                    Rule::string => unquote(raw.as_str()),
                    _ => raw.as_str().to_string(),
                });
            }
            _ => {}
        }
    }
    Ok(Condition {
        field: field.ok_or_else(|| LarqlError::Parse("missing condition field".into()))?,
        operator: operator.ok_or_else(|| LarqlError::Parse("missing condition operator".into()))?,
        value: value.ok_or_else(|| LarqlError::Parse("missing condition value".into()))?,
    })
}

fn parse_verify(pair: pest::iterators::Pair<Rule>) -> Result<LarqlQuery, LarqlError> {
    let mut vessel = None;
    let mut condition = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::vessel_name => {
                vessel = Some(
                    parse_vessel(inner.as_str())
                        .ok_or_else(|| LarqlError::UnknownVessel(inner.as_str().to_string()))?,
                )
            }
            Rule::condition => condition = Some(parse_condition(inner)?),
            _ => {}
        }
    }
    Ok(LarqlQuery::Verify(VerifyQuery {
        vessel: vessel.ok_or_else(|| LarqlError::Parse("missing VERIFY vessel".into()))?,
        condition,
    }))
}

fn parse_prepare(pair: pest::iterators::Pair<Rule>) -> Result<LarqlQuery, LarqlError> {
    let mut action = None;
    let mut vessel = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => action = Some(inner.as_str().to_string()),
            Rule::vessel_name => {
                vessel = Some(
                    parse_vessel(inner.as_str())
                        .ok_or_else(|| LarqlError::UnknownVessel(inner.as_str().to_string()))?,
                )
            }
            _ => {}
        }
    }
    Ok(LarqlQuery::Prepare(PrepareQuery {
        action: action.ok_or_else(|| LarqlError::Parse("missing PREPARE action".into()))?,
        vessel: vessel.ok_or_else(|| LarqlError::Parse("missing PREPARE CHECK vessel".into()))?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odu::ActionVessel;

    #[test]
    fn parses_describe_by_index_default_scale() {
        let q = parse_query("DESCRIBE 0").unwrap();
        match q {
            LarqlQuery::Describe(d) => {
                assert_eq!(d.target, OduRef::Index(0));
                assert_eq!(d.scales, vec![Scale::Micro]);
            }
            other => panic!("expected Describe, got {other:?}"),
        }
    }

    #[test]
    fn parses_describe_by_name_with_scales() {
        let q = parse_query(r#"DESCRIBE "Genesis × Genesis" AT SCALE micro,macro"#).unwrap();
        match q {
            LarqlQuery::Describe(d) => {
                assert_eq!(d.scales, vec![Scale::Micro, Scale::Macro]);
                assert!(matches!(d.target, OduRef::Name(_)));
            }
            other => panic!("expected Describe, got {other:?}"),
        }
    }

    #[test]
    fn parses_verify_with_condition() {
        let q = parse_query(r#"VERIFY Consent WHERE archetype CONTAINS "Steward""#).unwrap();
        match q {
            LarqlQuery::Verify(v) => {
                assert_eq!(v.vessel, ActionVessel::Consent);
                let c = v.condition.unwrap();
                assert_eq!(c.field, "archetype");
                assert_eq!(c.operator, Operator::Contains);
                assert_eq!(c.value, "Steward");
            }
            other => panic!("expected Verify, got {other:?}"),
        }
    }

    #[test]
    fn parses_verify_without_condition() {
        let q = parse_query("VERIFY Genesis").unwrap();
        match q {
            LarqlQuery::Verify(v) => {
                assert_eq!(v.vessel, ActionVessel::Genesis);
                assert!(v.condition.is_none());
            }
            other => panic!("expected Verify, got {other:?}"),
        }
    }

    #[test]
    fn parses_prepare() {
        let q = parse_query("PREPARE deploy CHECK: Consent").unwrap();
        match q {
            LarqlQuery::Prepare(p) => {
                assert_eq!(p.action, "deploy");
                assert_eq!(p.vessel, ActionVessel::Consent);
            }
            other => panic!("expected Prepare, got {other:?}"),
        }
    }

    #[test]
    fn unknown_vessel_is_an_error() {
        // "Alignment" parses structurally as a vessel_name (one capital +
        // lowercase run) but isn't one of the 16 real Action Vessels.
        assert!(matches!(
            parse_query("VERIFY Alignment"),
            Err(LarqlError::UnknownVessel(_))
        ));
    }

    #[test]
    fn garbage_input_is_a_parse_error() {
        assert!(parse_query("SYNTHESIZE not_supported_yet").is_err());
    }
}
