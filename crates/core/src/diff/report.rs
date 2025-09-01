use serde::{Serialize, Deserialize};
use crate::diff::{DiffResult, FunctionDelta, MatchKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonReport {
    pub added: Vec<FunctionDelta>,
    pub removed: Vec<FunctionDelta>,
    pub modified: Vec<FunctionDelta>,
    pub unchanged: Vec<FunctionDelta>,
}

impl From<DiffResult> for JsonReport {
    fn from(d: DiffResult) -> Self {
        Self {
            added: d.added,
            removed: d.removed,
            modified: d.modified,
            unchanged: d.unchanged,
        }
    }
}
