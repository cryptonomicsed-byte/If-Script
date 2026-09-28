//! Hermetic gate validation — the 7 principles of the IfáScript gate.
//!
//! ⚠️ **COVERAGE IS NOT 7 OF 7.** `default_gate()` installs rules for only
//! THREE principles — Correspondence, Polarity, Rhythm. Mentalism, Vibration,
//! CauseEffect and Gender have no rule. Because `validate_all()` iterates only
//! the rules that exist, an empty principle is indistinguishable from a
//! satisfied one: `allowed == true` has historically meant "nothing violated
//! in the rules I happen to have", NOT "all seven principles were evaluated".
//!
//! `GateValidationResult` now reports `evaluated_principles` /
//! `unevaluated_principles` so a caller can never read a partial pass as a
//! complete one. Use [`HermeticGate::new_strict`] / [`default_gate_strict`] to
//! make missing coverage a HARD block rather than silence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HermeticPrinciple {
    Mentalism,
    Correspondence,
    Vibration,
    Polarity,
    Rhythm,
    CauseEffect,
    Gender,
}

impl std::fmt::Display for HermeticPrinciple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HermeticPrinciple::Mentalism => write!(f, "Mentalism"),
            HermeticPrinciple::Correspondence => write!(f, "Correspondence"),
            HermeticPrinciple::Vibration => write!(f, "Vibration"),
            HermeticPrinciple::Polarity => write!(f, "Polarity"),
            HermeticPrinciple::Rhythm => write!(f, "Rhythm"),
            HermeticPrinciple::CauseEffect => write!(f, "CauseEffect"),
            HermeticPrinciple::Gender => write!(f, "Gender"),
        }
    }
}

impl std::str::FromStr for HermeticPrinciple {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "mentalism" => Ok(HermeticPrinciple::Mentalism),
            "correspondence" => Ok(HermeticPrinciple::Correspondence),
            "vibration" => Ok(HermeticPrinciple::Vibration),
            "polarity" => Ok(HermeticPrinciple::Polarity),
            "rhythm" => Ok(HermeticPrinciple::Rhythm),
            "cause_effect" | "causeeffect" | "cause-effect" => Ok(HermeticPrinciple::CauseEffect),
            "gender" => Ok(HermeticPrinciple::Gender),
            _ => Err(format!("Unknown hermetic principle: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementLevel {
    Hard,
    Soft,
    AuditOnly,
}

impl HermeticPrinciple {
    /// All seven, in the canonical Ascension-Domains tier order
    /// (docs/256---65536.md). Used to compute gate coverage: a principle in
    /// this list with no installed rule is UNEVALUATED, not satisfied.
    pub const ALL: [HermeticPrinciple; 7] = [
        HermeticPrinciple::Mentalism,
        HermeticPrinciple::Correspondence,
        HermeticPrinciple::Vibration,
        HermeticPrinciple::Polarity,
        HermeticPrinciple::Rhythm,
        HermeticPrinciple::CauseEffect,
        HermeticPrinciple::Gender,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            HermeticPrinciple::Mentalism => "Mentalism",
            HermeticPrinciple::Correspondence => "Correspondence",
            HermeticPrinciple::Vibration => "Vibration",
            HermeticPrinciple::Polarity => "Polarity",
            HermeticPrinciple::Rhythm => "Rhythm",
            HermeticPrinciple::CauseEffect => "CauseEffect",
            HermeticPrinciple::Gender => "Gender",
        }
    }
}

#[derive(Debug, Error)]
pub enum GateViolation {
    #[error("Tier access denied: required tier {required}, got {actual}")]
    TierAccessDenied { required: u8, actual: u8 },
    #[error("Odu out of range for tier: odu_id {odu_id} exceeds max {max_odu}")]
    OduOutOfRange { odu_id: u16, max_odu: u16 },
    #[error("Sovereignty violation: access class mismatch")]
    SovereigntyViolation,
    #[error("Memory tier too low: {message}")]
    MemoryTierTooLow { message: String },
    #[error("Gate {gate_name} violated: {detail}")]
    CustomViolation { gate_name: String, detail: String },
    #[error("Hermetic principle {principle} was never evaluated (no rule installed)")]
    PrincipleNotEvaluated { principle: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateViolationRecord {
    pub principle: HermeticPrinciple,
    pub message: String,
    pub enforcement: EnforcementLevel,
    pub alert_zangbeto: bool,
}

impl GateViolationRecord {
    pub fn should_block_message(&self, _msg: &str) -> bool {
        match self.enforcement {
            EnforcementLevel::Hard => true,
            EnforcementLevel::Soft => false,
            EnforcementLevel::AuditOnly => false,
        }
    }

    pub fn should_block(&self) -> bool {
        matches!(self.enforcement, EnforcementLevel::Hard)
    }
}

#[derive(Debug)]
pub struct GateValidationResult {
    pub allowed: bool,
    pub violations: Vec<GateViolationRecord>,
    pub warnings: usize,
    /// Principles that actually had at least one rule installed and ran.
    pub evaluated_principles: Vec<HermeticPrinciple>,
    /// Principles with NO installed rule — silently absent before this field
    /// existed. Non-empty means `allowed == true` is a PARTIAL verdict.
    pub unevaluated_principles: Vec<HermeticPrinciple>,
}

impl GateValidationResult {
    /// True only when all seven principles were actually evaluated.
    ///
    /// Read this instead of `allowed` whenever the question is "did the
    /// hermetic gate pass", because `allowed` alone answers the much weaker
    /// "did none of the installed rules fire".
    pub fn is_complete(&self) -> bool {
        self.unevaluated_principles.is_empty()
    }

    /// A one-line summary suitable for a receipt or a log line, so a partial
    /// verdict is never recorded as a bare `passed`.
    pub fn coverage_summary(&self) -> String {
        format!(
            "{}/7 principles evaluated{}",
            self.evaluated_principles.len(),
            if self.is_complete() {
                String::new()
            } else {
                let names: Vec<&str> = self
                    .unevaluated_principles
                    .iter()
                    .map(|p| p.as_str())
                    .collect();
                format!("; NOT EVALUATED: {}", names.join(", "))
            }
        )
    }
}

pub type GateCheckFn = Box<dyn Fn(&GateContext<'_>) -> Option<GateViolation> + Send + Sync>;

pub struct GateRule {
    pub principle: HermeticPrinciple,
    pub enforcement: EnforcementLevel,
    pub alert_zangbeto: bool,
    pub check: GateCheckFn,
}

impl std::fmt::Debug for GateRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GateRule")
            .field("principle", &self.principle)
            .field("enforcement", &self.enforcement)
            .field("alert_zangbeto", &self.alert_zangbeto)
            .finish()
    }
}

pub struct GateContext<'a> {
    pub tier: u8,
    pub odu_id: u16,
    pub access_class: &'a crate::cosmogram::AccessClass,
    pub memory_tier: &'a crate::soul::MemoryTier,
}

pub struct HermeticGate {
    rules: HashMap<HermeticPrinciple, Vec<GateRule>>,
    /// When true, a principle with no installed rule counts as a HARD block
    /// instead of silent absence. Off by default so existing callers keep
    /// their behaviour; every new caller that treats this gate as a security
    /// boundary should turn it on (see `new_strict` / `default_gate_strict`).
    strict: bool,
}

impl HermeticGate {
    pub fn new() -> Self {
        HermeticGate {
            rules: HashMap::new(),
            strict: false,
        }
    }

    /// A gate that refuses to report success unless all seven principles ran.
    pub fn new_strict() -> Self {
        HermeticGate {
            rules: HashMap::new(),
            strict: true,
        }
    }

    /// Principles with no installed rule, in canonical tier order.
    pub fn missing_principles(&self) -> Vec<HermeticPrinciple> {
        HermeticPrinciple::ALL
            .iter()
            .filter(|p| !self.rules.contains_key(p))
            .cloned()
            .collect()
    }

    pub fn add_rule(&mut self, rule: GateRule) {
        self.rules
            .entry(rule.principle.clone())
            .or_default()
            .push(rule);
    }

    pub fn validate_all(&self, ctx: &GateContext<'_>) -> GateValidationResult {
        let mut violations = Vec::new();
        let mut has_hard_block = false;

        for rules in self.rules.values() {
            for rule in rules {
                if let Some(violation) = (rule.check)(ctx) {
                    let message = violation.to_string();
                    let record = GateViolationRecord {
                        principle: rule.principle.clone(),
                        message,
                        enforcement: rule.enforcement.clone(),
                        alert_zangbeto: rule.alert_zangbeto,
                    };
                    if record.should_block() {
                        has_hard_block = true;
                    }
                    violations.push(record);
                }
            }
        }

        // Coverage: report what actually ran, in canonical order, so a caller
        // can never mistake a partial pass for a complete one. In strict mode
        // each unevaluated principle is escalated to a HARD violation.
        let mut evaluated = Vec::new();
        let mut unevaluated = Vec::new();
        for principle in HermeticPrinciple::ALL.iter() {
            if self.rules.contains_key(principle) {
                evaluated.push(principle.clone());
            } else {
                unevaluated.push(principle.clone());
                if self.strict {
                    has_hard_block = true;
                    violations.push(GateViolationRecord {
                        principle: principle.clone(),
                        message: GateViolation::PrincipleNotEvaluated {
                            principle: principle.as_str().to_string(),
                        }
                        .to_string(),
                        enforcement: EnforcementLevel::Hard,
                        alert_zangbeto: true,
                    });
                }
            }
        }

        let warnings = violations
            .iter()
            .filter(|v| {
                matches!(
                    v.enforcement,
                    EnforcementLevel::Soft | EnforcementLevel::AuditOnly
                )
            })
            .count();

        GateValidationResult {
            allowed: !has_hard_block,
            violations,
            warnings,
            evaluated_principles: evaluated,
            unevaluated_principles: unevaluated,
        }
    }
}

impl Default for HermeticGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a default HermeticGate with standard rules
pub fn default_gate() -> HermeticGate {
    let mut gate = HermeticGate::new();

    // Correspondence rule: tier must match odu range
    gate.add_rule(GateRule {
        principle: HermeticPrinciple::Correspondence,
        enforcement: EnforcementLevel::Hard,
        alert_zangbeto: true,
        check: Box::new(|ctx| {
            let max_odu = match ctx.tier {
                1 => 255u16,
                2 => 2047,
                3 => 4095,
                4 => 8191,
                5 => 16383,
                6 => 32767,
                7 => 65535,
                _ => {
                    return Some(GateViolation::TierAccessDenied {
                        required: 1,
                        actual: ctx.tier,
                    })
                }
            };
            if ctx.odu_id > max_odu {
                Some(GateViolation::OduOutOfRange {
                    odu_id: ctx.odu_id,
                    max_odu,
                })
            } else {
                None
            }
        }),
    });

    // Polarity rule: MachineOnly cannot have odu_id 0
    gate.add_rule(GateRule {
        principle: HermeticPrinciple::Polarity,
        enforcement: EnforcementLevel::Hard,
        alert_zangbeto: true,
        check: Box::new(|ctx| {
            if ctx.odu_id == 0
                && matches!(ctx.access_class, crate::cosmogram::AccessClass::MachineOnly)
            {
                Some(GateViolation::SovereigntyViolation)
            } else {
                None
            }
        }),
    });

    // Rhythm rule: memory tier audit
    gate.add_rule(GateRule {
        principle: HermeticPrinciple::Rhythm,
        enforcement: EnforcementLevel::AuditOnly,
        alert_zangbeto: false,
        check: Box::new(|ctx| {
            if ctx.tier >= 6
                && matches!(ctx.memory_tier, crate::soul::MemoryTier::Tier3Contributable)
            {
                Some(GateViolation::MemoryTierTooLow {
                    message: format!("tier {} requires deeper memory tier", ctx.tier),
                })
            } else {
                None
            }
        }),
    });

    gate
}

/// `default_gate()` with coverage enforcement ON.
///
/// Identical rules, but a principle with no rule becomes a HARD violation
/// instead of silence. Use this wherever the gate is being treated as a
/// security boundary — with `default_gate()` the result is a 3-of-7 verdict
/// wearing a pass.
pub fn default_gate_strict() -> HermeticGate {
    let mut gate = default_gate();
    gate.strict = true;
    gate
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::cosmogram::AccessClass;
    use crate::soul::MemoryTier;

    /// A context that violates none of the three installed rules.
    fn clean_ctx() -> (AccessClass, MemoryTier) {
        (AccessClass::Public, MemoryTier::Tier1Deep)
    }

    fn ctx<'a>(ac: &'a AccessClass, mt: &'a MemoryTier) -> GateContext<'a> {
        GateContext {
            tier: 1,
            odu_id: 10,
            access_class: ac,
            memory_tier: mt,
        }
    }

    #[test]
    fn default_gate_reports_three_of_seven_and_names_the_four_missing() {
        let gate = default_gate();
        let (ac, mt) = clean_ctx();
        let result = gate.validate_all(&ctx(&ac, &mt));

        // The pass is real but PARTIAL — this is the fact that used to be
        // invisible, because `allowed` alone said `true`.
        assert!(result.allowed);
        assert_eq!(result.evaluated_principles.len(), 3);
        assert_eq!(result.unevaluated_principles.len(), 4);
        assert!(!result.is_complete());

        let missing: Vec<&str> = result
            .unevaluated_principles
            .iter()
            .map(|p| p.as_str())
            .collect();
        assert_eq!(
            missing,
            vec!["Mentalism", "Vibration", "CauseEffect", "Gender"]
        );

        // And the evaluated set is the three the source actually installs.
        let ran: Vec<&str> = result
            .evaluated_principles
            .iter()
            .map(|p| p.as_str())
            .collect();
        assert_eq!(ran, vec!["Correspondence", "Polarity", "Rhythm"]);

        // A receipt/log line can no longer record this as a bare `passed`.
        let summary = result.coverage_summary();
        assert!(
            summary.contains("3/7"),
            "summary lost the coverage count: {summary}"
        );
        assert!(
            summary.contains("NOT EVALUATED") && summary.contains("Mentalism"),
            "summary hides the unevaluated principles: {summary}"
        );
    }

    #[test]
    fn strict_gate_blocks_on_incomplete_coverage_where_the_default_does_not() {
        let (ac, mt) = clean_ctx();
        let c = ctx(&ac, &mt);

        let lax = default_gate().validate_all(&c);
        assert!(lax.allowed, "the 3-of-7 gate still passes a clean context");

        let strict = default_gate_strict().validate_all(&c);
        assert!(
            !strict.allowed,
            "strict gate must refuse to report success at 3/7 coverage"
        );
        let blocks = strict
            .violations
            .iter()
            .filter(|v| v.should_block())
            .count();
        assert_eq!(
            blocks, 4,
            "expected one hard block per unevaluated principle"
        );
        assert!(strict
            .violations
            .iter()
            .any(|v| v.message.contains("Gender")));
    }

    #[test]
    fn a_fully_covered_gate_is_complete_and_not_penalised_by_strict_mode() {
        // Install a rule for every principle, then confirm strict mode is a
        // no-op: coverage enforcement must not manufacture violations when
        // coverage actually exists.
        let mut gate = HermeticGate::new_strict();
        for principle in HermeticPrinciple::ALL.iter() {
            gate.add_rule(GateRule {
                principle: principle.clone(),
                enforcement: EnforcementLevel::Soft,
                alert_zangbeto: false,
                check: Box::new(|_ctx| None),
            });
        }

        let (ac, mt) = clean_ctx();
        let result = gate.validate_all(&ctx(&ac, &mt));
        assert!(gate.missing_principles().is_empty());
        assert!(result.is_complete());
        assert!(result.allowed);
        assert!(result.violations.is_empty());
        assert_eq!(result.coverage_summary(), "7/7 principles evaluated");
    }
}
