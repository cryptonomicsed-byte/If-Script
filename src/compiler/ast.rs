// ifascript/src/compiler/ast.rs
// IfáScript Abstract Syntax Tree — cognitive skeleton for the compiler pipeline

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub imports: Vec<ImportStmt>,
    pub definitions: Vec<Definition>,
    pub invocations: Vec<Invocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStmt {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Definition {
    Odu(OduDef),
    Ritual(RitualDef),
    Witness(WitnessDef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OduDef {
    pub name: String,
    pub type_param: String,
    pub prescriptions: Vec<PrescriptionStmt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualDef {
    pub name: String,
    pub params: Vec<Param>,
    pub attributes: Vec<Attribute>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub typ: TypeExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeExpr {
    Primitive(PrimitiveType),
    OduType { name: String, param: String },
    Generic { name: String, param: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    Bool,
    StringT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub value: Option<Literal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessDef {
    pub name: String,
    pub quorum: u8,
    pub oracle: String,
    pub anchor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation {
    pub ritual_name: String,
    pub gate: Option<GateSpec>,
    pub witness_quorum: Option<u8>,
    pub sabbath: Option<SabbathSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateSpec {
    pub principle: HermeticPrinciple,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl HermeticPrinciple {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mentalism" => Some(Self::Mentalism),
            "correspondence" => Some(Self::Correspondence),
            "vibration" => Some(Self::Vibration),
            "polarity" => Some(Self::Polarity),
            "rhythm" => Some(Self::Rhythm),
            "cause_effect" => Some(Self::CauseEffect),
            "gender" => Some(Self::Gender),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SabbathSpec {
    Saturday,
    Any,
    Custom(String),
}

impl SabbathSpec {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Saturday" => Self::Saturday,
            "any" => Self::Any,
            other => Self::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Prescription(PrescriptionStmt),
    Let(LetStmt),
    If(IfStmt),
    Return(Option<Expression>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescriptionStmt {
    pub action: String,
    pub args: Vec<Literal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetStmt {
    pub name: String,
    pub typ: TypeExpr,
    pub value: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Ident(String),
    OduLiteral {
        name: String,
        param: Option<Box<Literal>>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Str(String),
    Number(f64),
    Bool(bool),
    OduName(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
}
