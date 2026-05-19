//! Stack machine interpreter.

use avm_bytecode::module::AxcModule;
use avm_bytecode::opcode::{Instruction, Operand, Opcode};
use crate::value::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("stack underflow")]
    Underflow,
    #[error("unknown opcode at pc {0}")]
    BadOpcode(usize),
    #[error("invalid jump target {0}")]
    BadJump(u32),
    #[error("no event available")]
    NoEvent,
}

#[derive(Debug, Default)]
pub struct RunResult {
    pub emitted: Vec<Value>,
    pub halted: bool,
}

pub struct Interpreter {
    stack: Vec<Value>,
    locals: [Value; 256],
    pc: usize,
    module: AxcModule,
    current_event: Option<Value>,
    input_queue: Vec<Value>,
}

impl Interpreter {
    pub fn new(module: AxcModule) -> Self {
        Self {
            stack: Vec::new(),
            locals: std::array::from_fn(|_| Value::Null),
            pc: 0,
            module,
            current_event: None,
            input_queue: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: Value) {
        self.input_queue.push(event);
    }

    pub fn run(&mut self) -> Result<RunResult, RunError> {
        let mut result = RunResult::default();
        while self.pc < self.module.code.len() {
            let instr = self.module.code[self.pc].clone();
            self.pc += 1;
            match instr.op {
                Opcode::Push => {
                    let v = self.operand_value(instr.operand.as_ref())?;
                    self.stack.push(v);
                }
                Opcode::Pop => {
                    self.stack.pop().ok_or(RunError::Underflow)?;
                }
                Opcode::Dup => {
                    let v = self.stack.last().ok_or(RunError::Underflow)?.clone();
                    self.stack.push(v);
                }
                Opcode::Swap => {
                    let n = self.stack.len();
                    if n < 2 {
                        return Err(RunError::Underflow);
                    }
                    self.stack.swap(n - 1, n - 2);
                }
                Opcode::Add => self.bin_op(|a, b| num_add(a, b))?,
                Opcode::Sub => self.bin_op(|a, b| num_sub(a, b))?,
                Opcode::Mul => self.bin_op(|a, b| num_mul(a, b))?,
                Opcode::Div => self.bin_op(|a, b| num_div(a, b))?,
                Opcode::Mod => self.bin_op(|a, b| num_mod(a, b))?,
                Opcode::Neg => {
                    let v = self.stack.pop().ok_or(RunError::Underflow)?;
                    self.stack.push(neg(v));
                }
                Opcode::Eq => self.bin_op(|a, b| Value::Bool(cmp_eq(&a, &b)))?,
                Opcode::Ne => self.bin_op(|a, b| Value::Bool(!cmp_eq(&a, &b)))?,
                Opcode::Lt => self.bin_op(|a, b| Value::Bool(cmp_lt(&a, &b)))?,
                Opcode::Gt => self.bin_op(|a, b| Value::Bool(cmp_lt(&b, &a)))?,
                Opcode::Le => {
                    self.bin_op(|a, b| Value::Bool(cmp_lt(&a, &b) || cmp_eq(&a, &b)))?
                }
                Opcode::Ge => {
                    self.bin_op(|a, b| Value::Bool(cmp_lt(&b, &a) || cmp_eq(&a, &b)))?
                }
                Opcode::And => self.bin_op(|a, b| Value::Bool(a.as_bool() && b.as_bool()))?,
                Opcode::Or => self.bin_op(|a, b| Value::Bool(a.as_bool() || b.as_bool()))?,
                Opcode::Not => {
                    let v = self.stack.pop().ok_or(RunError::Underflow)?;
                    self.stack.push(Value::Bool(!v.as_bool()));
                }
                Opcode::Jmp => {
                    let off = match instr.operand {
                        Some(Operand::U32(o)) => o,
                        _ => return Err(RunError::BadJump(0)),
                    };
                    self.pc = off as usize;
                }
                Opcode::JmpIf | Opcode::JmpIfNot => {
                    let off = match instr.operand {
                        Some(Operand::U32(o)) => o as usize,
                        _ => return Err(RunError::BadJump(0)),
                    };
                    let cond = self.stack.pop().ok_or(RunError::Underflow)?;
                    let take = cond.as_bool();
                    let branch = if instr.op == Opcode::JmpIf {
                        take
                    } else {
                        !take
                    };
                    if branch {
                        self.pc = off;
                    }
                }
                Opcode::GetField => {
                    let field = match instr.operand {
                        Some(Operand::Str(s)) => s,
                        Some(Operand::U32(idx)) => self
                            .module
                            .constants
                            .get(idx as usize)
                            .cloned()
                            .unwrap_or_default(),
                        _ => return Err(RunError::Underflow),
                    };
                    let obj = self
                        .stack
                        .pop()
                        .or_else(|| self.current_event.clone())
                        .unwrap_or(Value::Null);
                    let v = obj.field(&field).unwrap_or(Value::Null);
                    self.stack.push(v);
                }
                Opcode::LoadLocal => {
                    let idx = match instr.operand {
                        Some(Operand::U8(i)) => i as usize,
                        Some(Operand::U16(i)) => i as usize,
                        Some(Operand::U32(i)) => i as usize,
                        _ => 0,
                    };
                    if idx < 256 {
                        self.stack.push(self.locals[idx].clone());
                    }
                }
                Opcode::StoreLocal => {
                    let idx = match instr.operand {
                        Some(Operand::U8(i)) => i as usize,
                        Some(Operand::U32(i)) => i as usize,
                        _ => 0,
                    };
                    let v = self.stack.pop().ok_or(RunError::Underflow)?;
                    if idx < 256 {
                        self.locals[idx] = v;
                    }
                }
                Opcode::NextEvent => {
                    if let Some(ev) = self.input_queue.first().cloned() {
                        self.input_queue.remove(0);
                        self.current_event = Some(ev.clone());
                        self.stack.push(ev);
                    } else {
                        return Ok(result);
                    }
                }
                Opcode::Emit => {
                    let v = self
                        .stack
                        .pop()
                        .or_else(|| self.current_event.clone())
                        .unwrap_or(Value::Null);
                    result.emitted.push(v);
                }
                Opcode::Halt => {
                    result.halted = true;
                    return Ok(result);
                }
                Opcode::Predict => {
                    // stub until ML phase
                    self.stack.push(Value::Float(0.0));
                }
                Opcode::Serialize | Opcode::Deserialize => {
                    let v = self.stack.pop().ok_or(RunError::Underflow)?;
                    self.stack.push(v);
                }
                _ => {
                    // unimplemented ops are no-ops in phase 0
                }
            }
        }
        Ok(result)
    }

    fn operand_value(&self, op: Option<&Operand>) -> Result<Value, RunError> {
        Ok(match op {
            Some(Operand::I64(i)) => Value::Int(*i),
            Some(Operand::F64(f)) => Value::Float(*f),
            Some(Operand::Bool(b)) => Value::Bool(*b),
            Some(Operand::Str(s)) => Value::Str(s.clone()),
            Some(Operand::U32(idx)) => {
                let name = self.module.constants.get(*idx as usize).cloned();
                if let Some(n) = name {
                    self.current_event
                        .as_ref()
                        .and_then(|e| e.field(&n))
                        .unwrap_or(Value::Str(n))
                } else {
                    Value::Null
                }
            }
            None => Value::Null,
            _ => Value::Null,
        })
    }

    fn bin_op<F>(&mut self, f: F) -> Result<(), RunError>
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let b = self.stack.pop().ok_or(RunError::Underflow)?;
        let a = self.stack.pop().ok_or(RunError::Underflow)?;
        self.stack.push(f(a, b));
        Ok(())
    }
}

fn num_add(a: Value, b: Value) -> Value {
    Value::Float(to_f64(&a) + to_f64(&b))
}

fn num_sub(a: Value, b: Value) -> Value {
    Value::Float(to_f64(&a) - to_f64(&b))
}

fn num_mul(a: Value, b: Value) -> Value {
    Value::Float(to_f64(&a) * to_f64(&b))
}

fn num_div(a: Value, b: Value) -> Value {
    Value::Float(to_f64(&a) / to_f64(&b))
}

fn num_mod(a: Value, b: Value) -> Value {
    Value::Float(to_f64(&a) % to_f64(&b))
}

fn neg(v: Value) -> Value {
    Value::Float(-to_f64(&v))
}

fn to_f64(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => 0.0,
    }
}

fn cmp_eq(a: &Value, b: &Value) -> bool {
    to_f64(a) == to_f64(b)
}

fn cmp_lt(a: &Value, b: &Value) -> bool {
    to_f64(a) < to_f64(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aql_compile::compile;
    use std::collections::HashMap;

    #[test]
    fn run_filter_pipeline() {
        const SRC: &str = r#"source "s"
|> filter(x > 1.0)
|> sink "out""#;
        let compiled = compile(SRC).unwrap();
        let mut vm = Interpreter::new(compiled.module);
        let mut ev = HashMap::new();
        ev.insert("x".into(), Value::Float(2.0));
        vm.push_event(Value::Struct(ev));
        let res = vm.run().unwrap();
        assert!(!res.emitted.is_empty() || res.halted);
    }
}
