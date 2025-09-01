//! bindiff-core: parsing, disassembly, hashing, matching, and diff models.

pub mod loader;
pub mod arch;
pub mod analysis;
pub mod diff;

pub use analysis::hash::{FunctionHash, SimHash64};
pub use diff::{DiffResult, FunctionDelta, MatchKind};
