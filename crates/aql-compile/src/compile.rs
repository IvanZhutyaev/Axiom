//! Compile AQL source to `.axc` module.

use crate::logical::{build_logical_plan, optimize};
use crate::types::TypeChecker;
use aql_syntax::ast::{BinOp, Expr, Literal, Stage, UnOp};
use aql_syntax::{parse, Program};
use avm_bytecode::module::AxcModule;
use avm_bytecode::opcode::{Instruction, Operand, Opcode};
use avm_bytecode::AXC_VERSION_V2;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("parse error: {0}")]
    Parse(#[from] aql_syntax::ParseError),
    #[error("type error: {0}")]
    Type(#[from] crate::types::TypeError),
}

pub struct CompileResult {
    pub program: Program,
    pub module: AxcModule,
}

pub fn compile(source: &str) -> Result<CompileResult, CompileError> {
    let program = parse(source)?;
    TypeChecker::check_program(&program)?;
    let plan = optimize(build_logical_plan(&program));
    let module = emit_bytecode(&program, &plan);
    Ok(CompileResult { program, module })
}

fn emit_bytecode(program: &Program, plan: &crate::logical::LogicalPlan) -> AxcModule {
    let mut module = AxcModule::default();
    module.version = AXC_VERSION_V2;
    module.pipeline_name = "pipeline".into();

    for stage in &program.stages {
        match stage {
            Stage::Source { name } => {
                module.sources.push(name.clone());
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: format!("source:{name}"),
                    kind: "source".into(),
                });
                module.code.push(Instruction::new(Opcode::NextEvent));
            }
            Stage::Sink { name } => {
                module.sinks.push(name.clone());
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: format!("sink:{name}"),
                    kind: "sink".into(),
                });
                module.code.push(Instruction::new(Opcode::Emit));
            }
            Stage::Filter { predicate } => {
                emit_expr(&mut module, predicate);
                let jmp_false = module.code.len();
                module
                    .code
                    .push(Instruction::with_operand(Opcode::JmpIfNot, Operand::U32(0)));
                module.code.push(Instruction::new(Opcode::Emit));
                let end = module.code.len() as u32;
                if let Some(Instruction {
                    operand: Some(Operand::U32(ref mut off)),
                    ..
                }) = module.code.get_mut(jmp_false)
                {
                    *off = end;
                }
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: "filter".into(),
                    kind: "filter".into(),
                });
            }
            Stage::Window {
                size_ms,
                aggregates,
                ..
            } => {
                module.code.push(Instruction::with_operand(
                    Opcode::Push,
                    Operand::I64(*size_ms as i64),
                ));
                for agg in aggregates {
                    if let Some(arg) = &agg.arg {
                        emit_expr(&mut module, arg);
                    }
                    module.code.push(Instruction::new(Opcode::Emit));
                }
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: "window".into(),
                    kind: "window".into(),
                });
            }
            Stage::Map { projection } => {
                module.code.push(Instruction::new(Opcode::NewStruct));
                for binding in projection {
                    emit_expr(&mut module, &binding.expr);
                    module.code.push(Instruction::with_operand(
                        Opcode::SetField,
                        Operand::Str(binding.name.clone()),
                    ));
                }
                module.code.push(Instruction::new(Opcode::Emit));
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: "map".into(),
                    kind: "map".into(),
                });
            }
            Stage::Watermark { field, delay_ms } => {
                let idx = module.constants.len() as u32;
                module.constants.push(field.clone());
                module.code.push(Instruction::with_operand(Opcode::Push, Operand::U32(idx)));
                module.code.push(Instruction::with_operand(
                    Opcode::Push,
                    Operand::I64(*delay_ms as i64),
                ));
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: format!("watermark:{field}"),
                    kind: "watermark".into(),
                });
            }
            _ => {
                module.operators.push(avm_bytecode::module::OperatorMeta {
                    name: format!("{stage:?}"),
                    kind: "op".into(),
                });
            }
        }
    }
    module.code.push(Instruction::new(Opcode::Halt));
    let _ = plan;
    module
}

fn emit_expr(module: &mut AxcModule, expr: &Expr) {
    match expr {
        Expr::Literal(lit) => {
            let op = match lit {
                Literal::Int(v) => Operand::I64(*v),
                Literal::Float(v) => Operand::F64(*v),
                Literal::Bool(v) => Operand::Bool(*v),
                Literal::String(s) => {
                    let idx = module.constants.len() as u32;
                    module.constants.push(s.clone());
                    Operand::U32(idx)
                }
                Literal::Null => Operand::I64(0),
            };
            module
                .code
                .push(Instruction::with_operand(Opcode::Push, op));
        }
        Expr::Ident(name) => {
            module.code.push(Instruction::with_operand(
                Opcode::GetField,
                Operand::Str(name.clone()),
            ));
        }
        Expr::Field(inner, field) => {
            emit_expr(module, inner);
            module.code.push(Instruction::with_operand(
                Opcode::GetField,
                Operand::Str(field.clone()),
            ));
        }
        Expr::Binary { op, left, right } => {
            emit_expr(module, left);
            emit_expr(module, right);
            let opcode = match op {
                BinOp::Add => Opcode::Add,
                BinOp::Sub => Opcode::Sub,
                BinOp::Mul => Opcode::Mul,
                BinOp::Div => Opcode::Div,
                BinOp::Mod => Opcode::Mod,
                BinOp::Eq => Opcode::Eq,
                BinOp::Ne => Opcode::Ne,
                BinOp::Lt => Opcode::Lt,
                BinOp::Gt => Opcode::Gt,
                BinOp::Le => Opcode::Le,
                BinOp::Ge => Opcode::Ge,
                BinOp::And => Opcode::And,
                BinOp::Or => Opcode::Or,
            };
            module.code.push(Instruction::new(opcode));
        }
        Expr::Unary { op, expr } => {
            emit_expr(module, expr);
            let opcode = match op {
                UnOp::Not => Opcode::Not,
                UnOp::Neg => Opcode::Neg,
            };
            module.code.push(Instruction::new(opcode));
        }
        Expr::Call { name, args } => {
            for a in args {
                emit_expr(module, a);
            }
            module.code.push(Instruction::with_operand(
                Opcode::Call,
                Operand::Str(name.clone()),
            ));
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            emit_expr(module, cond);
            let jmp_false = module.code.len();
            module
                .code
                .push(Instruction::with_operand(Opcode::JmpIfNot, Operand::U32(0)));
            emit_expr(module, then_branch);
            let jmp_end = module.code.len();
            module
                .code
                .push(Instruction::with_operand(Opcode::Jmp, Operand::U32(0)));
            let else_start = module.code.len() as u32;
            if let Some(Instruction {
                operand: Some(Operand::U32(ref mut off)),
                ..
            }) = module.code.get_mut(jmp_false)
            {
                *off = else_start;
            }
            emit_expr(module, else_branch);
            let end = module.code.len() as u32;
            if let Some(Instruction {
                operand: Some(Operand::U32(ref mut off)),
                ..
            }) = module.code.get_mut(jmp_end)
            {
                *off = end;
            }
        }
        Expr::Let { name, value, body } => {
            emit_expr(module, value);
            let slot = module.constants.len() as u32;
            module.constants.push(name.clone());
            module
                .code
                .push(Instruction::with_operand(Opcode::StoreLocal, Operand::U32(slot)));
            emit_expr(module, body);
        }
        Expr::Match { scrutinee, arms } => {
            emit_expr(module, scrutinee);
            module.code.push(Instruction::new(Opcode::Dup));
            let mut end_jumps = Vec::new();
            for (i, arm) in arms.iter().enumerate() {
                if i > 0 {
                    module.code.push(Instruction::new(Opcode::Dup));
                }
                emit_expr(module, &Expr::Literal(arm.pattern.clone()));
                module.code.push(Instruction::new(Opcode::Eq));
                let jmp_next = module.code.len();
                module
                    .code
                    .push(Instruction::with_operand(Opcode::JmpIfNot, Operand::U32(0)));
                module.code.push(Instruction::new(Opcode::Pop));
                emit_expr(module, &arm.body);
                let jmp_end = module.code.len();
                module
                    .code
                    .push(Instruction::with_operand(Opcode::Jmp, Operand::U32(0)));
                end_jumps.push(jmp_end);
                let next_arm = module.code.len() as u32;
                if let Some(Instruction {
                    operand: Some(Operand::U32(ref mut off)),
                    ..
                }) = module.code.get_mut(jmp_next)
                {
                    *off = next_arm;
                }
            }
            module.code.push(Instruction::new(Opcode::Pop));
            let end = module.code.len() as u32;
            for jmp in end_jumps {
                if let Some(Instruction {
                    operand: Some(Operand::U32(ref mut off)),
                    ..
                }) = module.code.get_mut(jmp)
                {
                    *off = end;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avm_bytecode::load_axc;
    use avm_bytecode::save_axc;

    const TZ: &str = r#"source "sensor_data"
|> filter(temperature > 30.0)
|> window(tumbling, size=5s)
   aggregate(avg_temp = avg(temperature), count = count(*))
|> sink "alerts""#;

    #[test]
    fn compile_tz_pipeline() {
        let result = compile(TZ).unwrap();
        assert!(!result.module.code.is_empty());
        assert_eq!(result.module.sources, vec!["sensor_data".to_string()]);
        assert_eq!(result.module.sinks, vec!["alerts".to_string()]);
        let mut buf = Vec::new();
        save_axc(&result.module, &mut buf).unwrap();
        let loaded = load_axc(&mut buf.as_slice()).unwrap();
        assert_eq!(loaded.sinks, result.module.sinks);
    }
}
