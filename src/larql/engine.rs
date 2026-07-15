use crate::larql::ast::{
    Condition, DescribeQuery, LarqlQuery, OduRef, Operator, PrepareQuery, Scale, VerifyQuery,
};
use crate::larql::error::LarqlError;
use crate::odu::{get_odu, lookup_by_name, Odu, ODU_SET};

/// Result of executing a [`LarqlQuery`] — human-readable summary lines, plus
/// a pass/fail flag that's only meaningful for `VERIFY`.
#[derive(Debug, Clone, PartialEq)]
pub struct LarqlAnswer {
    pub summary: Vec<String>,
    /// `Some(passed)` for `VERIFY`; `None` for `DESCRIBE`/`PREPARE`.
    pub passed: Option<bool>,
}

/// Execute a query against the Digital Calabash corpus (`crate::odu`).
pub fn execute(query: &LarqlQuery) -> Result<LarqlAnswer, LarqlError> {
    match query {
        LarqlQuery::Describe(q) => execute_describe(q),
        LarqlQuery::Verify(q) => execute_verify(q),
        LarqlQuery::Prepare(q) => execute_prepare(q),
    }
}

fn resolve(target: &OduRef) -> Result<&'static Odu, LarqlError> {
    match target {
        OduRef::Index(i) => Ok(get_odu(*i)),
        OduRef::Name(n) => lookup_by_name(n).ok_or_else(|| LarqlError::NotFound(n.clone())),
    }
}

fn describe_at_scale(odu: &Odu, scale: Scale) -> Vec<String> {
    let mut lines = vec![format!("{} — {}", odu.universal_name, odu.archetype)];
    if matches!(scale, Scale::Meso | Scale::Macro) {
        for p in odu.prescriptions {
            lines.push(format!("  prescription: {p}"));
        }
    }
    if matches!(scale, Scale::Macro) {
        for t in odu.taboos {
            lines.push(format!("  taboo: {t}"));
        }
        lines.push(format!("  archetypes: {:?}", odu.archetypes));
        lines.push(format!(
            "  interpretation_type: {}",
            odu.interpretation_type
        ));
    }
    lines
}

fn execute_describe(query: &DescribeQuery) -> Result<LarqlAnswer, LarqlError> {
    let odu = resolve(&query.target)?;
    let mut summary = Vec::new();
    let multi = query.scales.len() > 1;
    for &scale in &query.scales {
        if multi {
            let label = match scale {
                Scale::Micro => "micro",
                Scale::Meso => "meso",
                Scale::Macro => "macro",
            };
            summary.push(format!("[{label}]"));
        }
        summary.extend(describe_at_scale(odu, scale));
    }
    Ok(LarqlAnswer {
        summary,
        passed: None,
    })
}

fn field_matches(odu: &Odu, condition: &Condition) -> bool {
    let target_field: &[&str] = match condition.field.as_str() {
        "universal_name" => &[odu.universal_name],
        "archetype" => &[odu.archetype],
        "archetypes" => odu.archetypes,
        "prescription" | "prescriptions" => odu.prescriptions,
        "name" => &[odu.name],
        _ => &[],
    };
    match condition.operator {
        Operator::Eq => target_field.iter().any(|f| *f == condition.value),
        Operator::Contains => target_field
            .iter()
            .any(|f| f.to_lowercase().contains(&condition.value.to_lowercase())),
    }
}

fn execute_verify(query: &VerifyQuery) -> Result<LarqlAnswer, LarqlError> {
    let matches: Vec<&'static Odu> = ODU_SET
        .iter()
        .filter(|o| o.vessel == query.vessel)
        .filter(|o| {
            query
                .condition
                .as_ref()
                .map(|c| field_matches(o, c))
                .unwrap_or(true)
        })
        .collect();

    let passed = !matches.is_empty();
    let mut summary = vec![if passed {
        format!(
            "✓ {:?} verification passed — {} matching entr{}",
            query.vessel,
            matches.len(),
            if matches.len() == 1 { "y" } else { "ies" }
        )
    } else {
        format!(
            "✗ {:?} verification failed — no matching entries",
            query.vessel
        )
    }];
    summary.extend(matches.iter().take(3).map(|o| format!("  e.g. {}", o.name)));

    Ok(LarqlAnswer {
        summary,
        passed: Some(passed),
    })
}

fn execute_prepare(query: &PrepareQuery) -> Result<LarqlAnswer, LarqlError> {
    let file_domain = query.vessel.file_domain();
    let summary = vec![
        format!("Write `{}` intent to {}", query.action, file_domain),
        format!("Vessel: {:?}", query.vessel),
        format!(
            "Log `{}` outcome to {} once complete",
            query.action, file_domain
        ),
    ];
    Ok(LarqlAnswer {
        summary,
        passed: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::larql::parser::parse_query;

    #[test]
    fn describe_by_index_micro() {
        let q = parse_query("DESCRIBE 0").unwrap();
        let answer = execute(&q).unwrap();
        assert_eq!(answer.summary.len(), 1);
        assert!(answer.summary[0].contains("The Light of First Becoming"));
        assert!(answer.passed.is_none());
    }

    #[test]
    fn describe_by_name_macro_includes_taboos() {
        let q = parse_query(r#"DESCRIBE "Genesis × Genesis" AT SCALE macro"#).unwrap();
        let answer = execute(&q).unwrap();
        assert!(answer.summary.iter().any(|l| l.contains("taboo:")));
        assert!(answer
            .summary
            .iter()
            .any(|l| l.contains("interpretation_type: synthetic")));
    }

    #[test]
    fn describe_unknown_name_is_not_found() {
        let q = parse_query(r#"DESCRIBE "not a real odu""#).unwrap();
        assert!(matches!(execute(&q), Err(LarqlError::NotFound(_))));
    }

    #[test]
    fn verify_passes_when_archetype_present_in_vessel() {
        // Genesis wave includes an entry with archetype "Steward" (index 5, 7).
        let q = parse_query(r#"VERIFY Genesis WHERE archetypes CONTAINS "Steward""#).unwrap();
        let answer = execute(&q).unwrap();
        assert_eq!(answer.passed, Some(true));
    }

    #[test]
    fn verify_fails_on_nonexistent_archetype() {
        let q = parse_query(r#"VERIFY Genesis WHERE archetypes CONTAINS "Not A Real Archetype""#)
            .unwrap();
        let answer = execute(&q).unwrap();
        assert_eq!(answer.passed, Some(false));
    }

    #[test]
    fn verify_without_condition_passes_for_any_populated_vessel() {
        let q = parse_query("VERIFY Rhythm").unwrap();
        let answer = execute(&q).unwrap();
        assert_eq!(answer.passed, Some(true));
    }

    #[test]
    fn prepare_references_vessel_file_domain() {
        let q = parse_query("PREPARE deploy CHECK: Consent").unwrap();
        let answer = execute(&q).unwrap();
        assert!(answer.summary.iter().any(|l| l.contains("consent_log.md")));
    }
}
