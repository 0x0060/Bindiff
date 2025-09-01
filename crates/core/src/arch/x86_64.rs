use crate::loader::BinaryImage;
use super::{Insn, OperandKind, FunctionIR};
use anyhow::{Result, bail};
use yaxpeax_arch::{Decoder, LengthedInstruction};
use yaxpeax_x86::long_mode::{Arch as X86_64, InstDecoder, Instruction, Operand};
use std::cmp::min;

fn normalize_operand(op: &Operand) -> OperandKind {
    use OperandKind::*;
    match op {
        Operand::Register(r) => Reg(format!("{:?}", r)),
        Operand::ImmediateI8(_)|Operand::ImmediateI16(_)|Operand::ImmediateI32(_)|Operand::ImmediateI64(_)
        |Operand::ImmediateU8(_)|Operand::ImmediateU16(_)|Operand::ImmediateU32(_)|Operand::ImmediateU64(_)
        => Imm,
        Operand::DisplacementU32(_)|Operand::DisplacementU64(_)|Operand::RegDeref(_)|Operand::RegDeref2(_, _)
        |Operand::RegDisp(_)|Operand::RegDeref3(_, _, _)|Operand::RegDispScaled(_, _, _)
        => Mem,
        _ => Other,
    }
}

fn normalize_instruction(addr: u64, insn: &Instruction) -> Insn {
    let mnemonic = format!("{:?}", insn.opcode());
    let mut op_kinds = Vec::new();
    for i in 0..insn.operand_count() {
        op_kinds.push(normalize_operand(&insn.operand(i)));
    }
    Insn { addr, mnemonic, op_kinds }
}

pub fn disassemble_functions(bin: &BinaryImage) -> Result<Vec<FunctionIR>> {
    if bin.arch != "x86" || bin.bits != 64 {
        bail!("Currently only x86_64 is implemented with yaxpeax");
    }
    let dec = InstDecoder::default();
    let bytes = &bin.data;
    let mut out = Vec::new();
    for f in &bin.functions {
        let start = f.start as usize;
        if start >= bytes.len() { continue; }
        // Determine a conservative slice length
        let max_len = if f.size > 0 { f.size as usize } else { min(bytes.len() - start, 4096) };
        let mut cursor = 0usize;
        let mut insns = Vec::new();
        while cursor < max_len {
            let off = start + cursor;
            match dec.decode(&bytes[off..]) {
                Ok(insn) => {
                    let len = insn.len() as usize;
                    if len == 0 { break; }
                    insns.push(normalize_instruction((off) as u64, &insn));
                    cursor += len;
                    // crude stop on RET to prevent falling into padding
                    if format!("{:?}", insn.opcode()).starts_with("RET") { break; }
                },
                Err(_) => break,
            }
        }
        if !insns.is_empty() {
            out.push(FunctionIR {
                name: f.name.clone(),
                start: f.start,
                size: f.size,
                insns,
            });
        }
    }
    Ok(out)
}
