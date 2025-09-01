pub mod report;
pub mod matching;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchKind {
    Exact,
    Fuzzy { hamming: u32 },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDelta {
    pub name_a: Option<String>,
    pub name_b: Option<String>,
    pub start_a: Option<u64>,
    pub start_b: Option<u64>,
    pub kind: MatchKind,
    pub changed: bool,
    pub insn_count_a: Option<usize>,
    pub insn_count_b: Option<usize>,
    pub unified_diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub added: Vec<FunctionDelta>,
    pub removed: Vec<FunctionDelta>,
    pub modified: Vec<FunctionDelta>,
    pub unchanged: Vec<FunctionDelta>,
}
