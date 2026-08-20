// ifascript/src/compiler/parser.rs
// Ògún's Forge: Minimal Parser — parses `invoke` statements

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::compiler::ast::{
    AseStmt, BindStmt, BinaryOp, ConsultStmt, Definition, Expression, IfStmt, LetStmt, Literal,
    MatchArm, MatchStmt, OduDef, OduPattern, Param, PrescriptionStmt, PrimitiveType, RitualDef,
    Statement, TypeExpr, WitnessDef,
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
                Rule::witness_def => definitions.push(Definition::Witness(parse_witness_def(pair))),
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

fn parse_binary_op(s: &str) -> BinaryOp {
    match s {
        "+" => BinaryOp::Add,
        "-" => BinaryOp::Sub,
        "*" => BinaryOp::Mul,
        "/" => BinaryOp::Div,
        "==" => BinaryOp::Eq,
        "!=" => BinaryOp::Neq,
        "<" => BinaryOp::Lt,
        ">" => BinaryOp::Gt,
        _ => BinaryOp::Add,
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Expression {
    let mut terms = Vec::new();
    let mut ops = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::expr_term => terms.push(parse_expr_term(p)),
            Rule::binary_op => ops.push(parse_binary_op(p.as_str())),
            _ => {}
        }
    }

    if terms.is_empty() {
        return Expression::Literal(Literal::Number(0.0));
    }

    let mut terms_iter = terms.into_iter();
    let mut result = terms_iter.next().unwrap();
    for (op, right) in ops.into_iter().zip(terms_iter) {
        result = Expression::BinaryOp {
            left: Box::new(result),
            op,
            right: Box::new(right),
        };
    }
    result
}

fn parse_expr_term(pair: pest::iterators::Pair<Rule>) -> Expression {
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::literal => return Expression::Literal(parse_literal(p)),
            Rule::ident => return Expression::Ident(p.as_str().to_string()),
            Rule::odu_literal_expr => return parse_odu_literal(p),
            Rule::call_expr => return parse_call_expr(p),
            Rule::expression => return parse_expression(p),
            _ => {}
        }
    }
    Expression::Literal(Literal::Number(0.0))
}

fn parse_odu_literal(pair: pest::iterators::Pair<Rule>) -> Expression {
    let mut name = String::new();
    let mut param = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::odu_name => name = p.as_str().to_string(),
            Rule::literal => param = Some(Box::new(parse_literal(p))),
            _ => {}
        }
    }

    Expression::OduLiteral { name, param }
}

fn parse_call_expr(pair: pest::iterators::Pair<Rule>) -> Expression {
    let mut name = String::new();
    let mut args = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::ident => name = p.as_str().to_string(),
            Rule::expression => args.push(parse_expression(p)),
            _ => {}
        }
    }

    Expression::Call { name, args }
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Statement {
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::consult_stmt => return Statement::Consult(parse_consult_stmt(p)),
            Rule::match_stmt => return Statement::Match(parse_match_stmt(p)),
            Rule::ase_stmt => return Statement::Ase(parse_ase_stmt(p)),
            Rule::dissolve_stmt => return parse_dissolve_stmt(p),
            Rule::bind_stmt => return Statement::Bind(parse_bind_stmt(p)),
            Rule::deliver_stmt => return parse_deliver_stmt(p),
            Rule::let_stmt => return Statement::Let(parse_let_stmt(p)),
            Rule::if_stmt => return Statement::If(parse_if_stmt(p)),
            Rule::return_stmt => return Statement::Return(parse_return_stmt(p)),
            Rule::prescription => return Statement::Prescription(parse_prescription(p)),
            _ => {}
        }
    }
    Statement::Prescription(PrescriptionStmt {
        action: String::new(),
        args: Vec::new(),
    })
}

fn parse_let_stmt(pair: pest::iterators::Pair<Rule>) -> LetStmt {
    let mut name = String::new();
    let mut typ = TypeExpr::Primitive(PrimitiveType::StringT);
    let mut value = Expression::Literal(Literal::Number(0.0));

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::ident => name = p.as_str().to_string(),
            Rule::type_expr => typ = parse_type_expr(p.as_str()),
            Rule::expression => value = parse_expression(p),
            _ => {}
        }
    }

    LetStmt { name, typ, value }
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> IfStmt {
    let mut condition = Expression::Literal(Literal::Bool(true));
    let mut then_block = Vec::new();
    let else_block = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::expression => {
                condition = parse_expression(p);
            }
            Rule::statement => {
                then_block.push(parse_statement(p));
            }
            _ => {}
        }
    }

    IfStmt {
        condition,
        then_block,
        else_block,
    }
}

fn parse_return_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Expression> {
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            return Some(parse_expression(p));
        }
    }
    None
}

fn parse_consult_stmt(pair: pest::iterators::Pair<Rule>) -> ConsultStmt {
    let mut condition = Expression::Literal(Literal::Bool(true));
    let mut then_block = Vec::new();
    let mut or_consult_clauses = Vec::new();
    let mut taboo_block = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::expression => {
                if condition == Expression::Literal(Literal::Bool(true)) {
                    condition = parse_expression(p);
                }
            }
            Rule::statement => {
                then_block.push(parse_statement(p));
            }
            Rule::or_consult_clause => {
                let mut clause_condition = Expression::Literal(Literal::Bool(true));
                let mut clause_body = Vec::new();
                for inner in p.into_inner() {
                    match inner.as_rule() {
                        Rule::expression => clause_condition = parse_expression(inner),
                        Rule::statement => clause_body.push(parse_statement(inner)),
                        _ => {}
                    }
                }
                or_consult_clauses.push((clause_condition, clause_body));
            }
            Rule::taboo_clause => {
                let mut body = Vec::new();
                for inner in p.into_inner() {
                    if inner.as_rule() == Rule::statement {
                        body.push(parse_statement(inner));
                    }
                }
                taboo_block = Some(body);
            }
            _ => {}
        }
    }

    ConsultStmt {
        condition,
        then_block,
        or_consult_clauses,
        taboo_block,
    }
}

fn parse_match_stmt(pair: pest::iterators::Pair<Rule>) -> MatchStmt {
    let mut expr = Expression::Literal(Literal::Bool(true));
    let mut arms = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::expression => expr = parse_expression(p),
            Rule::match_arm => {
                let mut pattern = OduPattern::Wildcard;
                let mut body = Vec::new();
                for inner in p.into_inner() {
                    match inner.as_rule() {
                        Rule::odu_pattern => pattern = parse_odu_pattern(inner),
                        Rule::statement => body.push(parse_statement(inner)),
                        _ => {}
                    }
                }
                arms.push(MatchArm { pattern, body });
            }
            _ => {}
        }
    }

    MatchStmt { expr, arms }
}

fn parse_odu_pattern(pair: pest::iterators::Pair<Rule>) -> OduPattern {
    let inner: Vec<_> = pair.into_inner().collect();
    let mut name = None;
    let mut param = None;

    for p in inner {
        match p.as_rule() {
            Rule::odu_name => {
                name = Some(p.as_str().to_string());
            }
            Rule::ident => {
                param = Some(p.as_str().to_string());
            }
            _ => {}
        }
    }

    if let Some(n) = name {
        OduPattern::Odu {
            name: n,
            param,
        }
    } else {
        OduPattern::Wildcard
    }
}

fn parse_ase_stmt(pair: pest::iterators::Pair<Rule>) -> AseStmt {
    let mut condition = Expression::Literal(Literal::Bool(true));
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            condition = parse_expression(p);
        }
    }
    AseStmt { condition }
}

fn parse_dissolve_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    for p in pair.into_inner() {
        if p.as_rule() == Rule::string {
            return Statement::Dissolve(unquote(p.as_str()));
        }
    }
    Statement::Dissolve("Unknown error".to_string())
}

fn parse_bind_stmt(pair: pest::iterators::Pair<Rule>) -> BindStmt {
    let mut name = String::new();
    let mut value = Expression::Literal(Literal::Number(0.0));

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::ident => name = p.as_str().to_string(),
            Rule::expression => value = parse_expression(p),
            _ => {}
        }
    }

    BindStmt { name, value }
}

fn parse_deliver_stmt(pair: pest::iterators::Pair<Rule>) -> Statement {
    for p in pair.into_inner() {
        if p.as_rule() == Rule::expression {
            return Statement::Deliver(Some(parse_expression(p)));
        }
    }
    Statement::Deliver(None)
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
            Rule::statement => body.push(parse_statement(inner)),
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

fn parse_witness_def(pair: pest::iterators::Pair<Rule>) -> WitnessDef {
    let mut name = String::new();
    let mut quorum = 0u8;
    let mut strings = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => name = inner.as_str().to_string(),
            Rule::number => quorum = inner.as_str().parse().unwrap_or(0),
            Rule::string => strings.push(unquote(inner.as_str())),
            _ => {}
        }
    }
    WitnessDef {
        name,
        quorum,
        oracle: strings.first().cloned().unwrap_or_default(),
        anchor: strings.get(1).cloned().unwrap_or_default(),
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

    // ── witness definitions ──────────────────────────────────────────────

    #[test]
    fn parse_witness_definition() {
        let src = r#"witness council: 3 @"https://oracle.example/v1" @"anchor.example/net";"#;
        let (defs, invs) = IfaParser::parse_definitions(src).unwrap();
        assert!(invs.is_empty());
        assert_eq!(defs.len(), 1);
        match &defs[0] {
            Definition::Witness(w) => {
                assert_eq!(w.name, "council");
                assert_eq!(w.quorum, 3);
                assert_eq!(w.oracle, "https://oracle.example/v1");
                assert_eq!(w.anchor, "anchor.example/net");
            }
            other => panic!("expected Witness def, got {other:?}"),
        }
    }

    #[test]
    fn witness_definition_mixes_with_odu_ritual_and_invocation() {
        let src = r#"
            witness council: 3 @"https://oracle.example/v1" @"anchor.example/net";
            odù Ogbe<dawn> { meditate; }
            ritual dawn_rite() { seal; }
            invoke dawn_rite witness 3;
        "#;
        let (defs, invs) = IfaParser::parse_definitions(src).unwrap();
        assert_eq!(defs.len(), 3);
        assert_eq!(invs.len(), 1);
        assert!(matches!(defs[0], Definition::Witness(_)));
        assert!(matches!(defs[1], Definition::Odu(_)));
        assert!(matches!(defs[2], Definition::Ritual(_)));
    }

    // ── statement bodies ──────────────────────────────────────────────

    #[test]
    fn ritual_with_prescription_statements() {
        let src = r#"
            ritual simple_rite() {
                offer("coconut");
                seal;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        match &defs[0] {
            Definition::Ritual(r) => {
                assert_eq!(r.name, "simple_rite");
                assert_eq!(r.body.len(), 2);
                assert!(matches!(r.body[0], Statement::Prescription(_)));
                assert!(matches!(r.body[1], Statement::Prescription(_)));
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }

    #[test]
    fn ritual_with_let_statement() {
        let src = r#"
            ritual with_binding() {
                let x: u8 = 42;
                seal;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        match &defs[0] {
            Definition::Ritual(r) => {
                assert_eq!(r.body.len(), 2);
                assert!(matches!(r.body[0], Statement::Let(_)));
                assert!(matches!(r.body[1], Statement::Prescription(_)));
                if let Statement::Let(let_stmt) = &r.body[0] {
                    assert_eq!(let_stmt.name, "x");
                }
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }

    #[test]
    fn ritual_with_if_statement() {
        let src = r#"
            ritual conditional() {
                if 1 { seal; }
                meditate;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        match &defs[0] {
            Definition::Ritual(r) => {
                assert_eq!(r.body.len(), 2);
                assert!(matches!(r.body[0], Statement::If(_)));
                assert!(matches!(r.body[1], Statement::Prescription(_)));
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }

    #[test]
    fn ritual_with_return_statement() {
        let src = r#"
            ritual with_return() {
                return 7;
                seal;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        match &defs[0] {
            Definition::Ritual(r) => {
                assert_eq!(r.body.len(), 2);
                assert!(matches!(r.body[0], Statement::Return(_)));
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }

    #[test]
    fn expression_with_binary_ops() {
        let src = r#"
            ritual math() {
                let result: u8 = 3 + 4 * 2;
                seal;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        match &defs[0] {
            Definition::Ritual(r) => {
                assert_eq!(r.body.len(), 2);
                if let Statement::Let(let_stmt) = &r.body[0] {
                    assert!(matches!(let_stmt.value, Expression::BinaryOp { .. }));
                }
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }

    #[test]
    fn if_statement_with_block() {
        let src = r#"
            ritual branching() {
                if x { seal; offer("yam"); }
                meditate;
            }
        "#;
        let (defs, _) = IfaParser::parse_definitions(src).unwrap();
        match &defs[0] {
            Definition::Ritual(r) => {
                if let Statement::If(if_stmt) = &r.body[0] {
                    assert_eq!(if_stmt.then_block.len(), 2);
                }
            }
            other => panic!("expected Ritual def, got {other:?}"),
        }
    }
}
