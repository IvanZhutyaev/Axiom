//! Bytecode disassembler for debugging.

use avm_bytecode::module::AxcModule;
use avm_bytecode::opcode::Operand;
use std::fmt::Write;

pub fn disassemble(module: &AxcModule) -> String {
    let mut out = String::new();
    for (i, instr) in module.code.iter().enumerate() {
        let _ = writeln!(
            out,
            "{i:04}: {:?} {:?}",
            instr.op,
            instr.operand.as_ref().map(operand_fmt)
        );
    }
    out
}

fn operand_fmt(op: &Operand) -> String {
    format!("{op:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use avm_bytecode::module::AxcModule;
    use avm_bytecode::opcode::{Instruction, Opcode};

    #[test]
    fn disasm_smoke() {
        let mut m = AxcModule::default();
        m.code.push(Instruction::new(Opcode::Halt));
        let text = disassemble(&m);
        assert!(text.contains("Halt"));
    }
}
