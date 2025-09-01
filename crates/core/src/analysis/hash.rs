use crate::arch::FunctionIR;
use blake3::Hasher;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimHash64(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionHash {
    pub name: String,
    pub start: u64,
    pub blake3: [u8; 32],
    pub simhash: SimHash64,
    pub n_insn: usize,
}

fn token_bytes(mnemonic: &str, op_kinds: &[crate::arch::OperandKind]) -> Vec<u8> {
    let mut v = mnemonic.as_bytes().to_vec();
    for ok in op_kinds {
        let tag = match ok {
            crate::arch::OperandKind::Reg(r) => {
                // normalize register class, e.g., rax -> REG64
                if r.starts_with('R') { b"REG64".to_vec() } else { b"REG".to_vec() }
            },
            crate::arch::OperandKind::Mem => b"MEM".to_vec(),
            crate::arch::OperandKind::Imm => b"IMM".to_vec(),
            crate::arch::OperandKind::Other => b"O".to_vec(),
        };
        v.extend_from_slice(&tag);
    }
    v
}

fn u64_from_first8(bytes: &[u8]) -> u64 {
    let mut out = 0u64;
    for (i, b) in bytes.iter().take(8).enumerate() {
        out |= (*b as u64) << (i * 8);
    }
    out
}

pub fn simhash(tokens: &[Vec<u8>]) -> SimHash64 {
    let mut acc = [0i32; 64];
    for t in tokens {
        let h = blake3::hash(&t).as_bytes().to_owned();
        let mut val = u64_from_first8(&h);
        for i in 0..64 {
            let bit = (val >> i) & 1;
            acc[i] += if bit == 1 { 1 } else { -1 };
        }
    }
    let mut out = 0u64;
    for i in 0..64 {
        if acc[i] >= 0 { out |= 1 << i; }
    }
    SimHash64(out)
}

pub fn hash_function(f: &FunctionIR) -> FunctionHash {
    let mut hasher = Hasher::new();
    let mut toks = Vec::with_capacity(f.insns.len());
    for insn in &f.insns {
        let tb = token_bytes(&insn.mnemonic, &insn.op_kinds);
        hasher.update(&tb);
        toks.push(tb);
    }
    FunctionHash {
        name: f.name.clone(),
        start: f.start,
        blake3: *hasher.finalize().as_bytes(),
        simhash: simhash(&toks),
        n_insn: f.insns.len(),
    }
}

pub fn hamming(a: SimHash64, b: SimHash64) -> u32 {
    (a.0 ^ b.0).count_ones()
}
