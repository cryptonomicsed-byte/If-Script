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
}

pub struct GateRule {
    pub principle: HermeticPrinciple,
    pub enforcement: EnforcementLevel,
    pub alert_zangbeto: bool,
    pub check: Box<dyn Fn(&GateContext<'_>) -> Option<GateViolation> + Send + Sync>,
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
}

impl HermeticGate {
    pub fn new() -> Self {
        HermeticGate {
            rules: HashMap::new(),
        }
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

        let warnings = violations
            .iter()
            .filter(|v| matches!(v.enforcement, EnforcementLevel::Soft | EnforcementLevel::AuditOnly))
            .count();

        GateValidationResult {
            allowed: !has_hard_block,
            violations,
            warnings,
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
                _ => return Some(GateViolation::TierAccessDenied { required: 1, actual: ctx.tier }),
            };
            if ctx.odu_id > max_odu {
                Some(GateViolation::OduOutOfRange { odu_id: ctx.odu_id, max_odu })
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
            if ctx.odu_id == 0 && matches!(ctx.access_class, crate::cosmogram::AccessClass::MachineOnly) {
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
            if ctx.tier >= 6 && matches!(ctx.memory_tier, crate::soul::MemoryTier::Tier3Contributable) {
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
