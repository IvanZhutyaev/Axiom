//! Type inference for AQL expressions.

use aql_syntax::ast::{AggFunc, Expr, Literal, Stage};
use aql_syntax::Program;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Unknown,
    Event,
}

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("unknown field {0}")]
    UnknownField(String),
    #[error("type mismatch in {0}")]
    Mismatch(String),
}

pub struct TypeChecker;

impl TypeChecker {
    pub fn check_program(program: &Program) -> Result<(), TypeError> {
        for stage in &program.stages {
            Self::check_stage(stage)?;
        }
        Ok(())
    }

    fn check_stage(stage: &Stage) -> Result<(), TypeError> {
        match stage {
            Stage::Filter { predicate } => {
                Self::infer_expr(predicate)?;
            }
            Stage::Map { projection } => {
                for b in projection {
                    Self::infer_expr(&b.expr)?;
                }
            }
            Stage::Window { aggregates, .. } => {
                for a in aggregates {
                    if let Some(arg) = &a.arg {
                        Self::infer_expr(arg)?;
                    }
                    let _ = a.func.clone();
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn infer_expr(expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(Literal::Int(_)) => Ok(Type::Int),
            Expr::Literal(Literal::Float(_)) => Ok(Type::Float),
            Expr::Literal(Literal::Bool(_)) => Ok(Type::Bool),
            Expr::Literal(Literal::String(_)) => Ok(Type::String),
            Expr::Literal(Literal::Null) => Ok(Type::Unknown),
            Expr::Ident(_) | Expr::Field(_, _) => Ok(Type::Unknown),
            Expr::Binary { op, left, right } => {
                let _l = Self::infer_expr(left)?;
                let _r = Self::infer_expr(right)?;
                use aql_syntax::ast::BinOp;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => Ok(Type::Float),
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        Ok(Type::Bool)
                    }
                    BinOp::And | BinOp::Or => Ok(Type::Bool),
                }
            }
            Expr::Unary { expr, .. } => Self::infer_expr(expr),
            Expr::Call { .. } => Ok(Type::Unknown),
        }
    }

    pub fn agg_result_type(func: &AggFunc) -> Type {
        match func {
            AggFunc::Count => Type::Int,
            AggFunc::Avg | AggFunc::Sum | AggFunc::Stddev | AggFunc::Percentile(_) => Type::Float,
            AggFunc::Min | AggFunc::Max | AggFunc::First | AggFunc::Last => Type::Unknown,
        }
    }
}
