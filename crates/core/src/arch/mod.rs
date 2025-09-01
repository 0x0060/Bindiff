pub mod x86_64;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperandKind {
    Reg(String),
    Mem,      // normalized memory operand
    Imm,      // immediate literal normalized
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insn {
    pub addr: u64,
    pub mnemonic: String,
    pub op_kinds: Vec<OperandKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionIR {
    pub name: String,
    pub start: u64,
    pub size: u64,
    pub insns: Vec<Insn>,
}
