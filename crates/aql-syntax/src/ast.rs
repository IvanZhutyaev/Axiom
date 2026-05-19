//! Abstract syntax tree for AQL pipelines.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stage {
    Source { name: String },
    Sink { name: String },
    Filter { predicate: Expr },
    Map { projection: Vec<FieldBinding> },
    FlatMap { expr: Expr },
    KeyBy { key: Expr },
    Window {
        kind: WindowKind,
        size_ms: u64,
        slide_ms: Option<u64>,
        gap_ms: Option<u64>,
        aggregates: Vec<Aggregate>,
    },
    Watermark { field: String, delay_ms: u64 },
    Join {
        other: String,
        join_type: JoinType,
        key: Expr,
        window_ms: u64,
    },
    Union { streams: Vec<String> },
    Split { branches: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WindowKind {
    Tumbling,
    Sliding,
    Session,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldBinding {
    pub name: String,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aggregate {
    pub name: String,
    pub func: AggFunc,
    pub arg: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggFunc {
    Sum,
    Count,
    Avg,
    Min,
    Max,
    Stddev,
    Percentile(f64),
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Field(Box<Expr>, String),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary { op: UnOp, expr: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}
