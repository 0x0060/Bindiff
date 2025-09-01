use crate::analysis::hash::{FunctionHash, hamming};
use crate::arch::FunctionIR;
use crate::diff::{FunctionDelta, MatchKind};
use similar::{TextDiff, ChangeTag};
use std::collections::HashMap;

fn insn_lines(f: &FunctionIR) -> Vec<String> {
    f.insns.iter().map(|i| {
        let ops = i.op_kinds.iter().map(|k| match k {
            crate::arch::OperandKind::Reg(r) => format!("REG({})", r),
            crate::arch::OperandKind::Mem => "MEM".to_string(),
            crate::arch::OperandKind::Imm => "IMM".to_string(),
            crate::arch::OperandKind::Other => "O".to_string(),
        }).collect::<Vec<_>>().join(",");
        format!("{} {}", i.mnemonic, ops)
    }).collect()
}

fn unified_diff(a: &FunctionIR, b: &FunctionIR) -> String {
    let a_lines = insn_lines(a);
    let b_lines = insn_lines(b);
    let diff = TextDiff::from_slices(&a_lines.join("
"), &b_lines.join("
"));
    let mut out = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        out.push_str(sign);
        out.push_str(change.to_string().as_str());
        out.push('
');
    }
    out
}

pub fn match_functions(
    fa: &[FunctionIR],
    fb: &[FunctionIR],
    ha: &[FunctionHash],
    hb: &[FunctionHash],
) -> (Vec<FunctionDelta>, Vec<FunctionDelta>, Vec<FunctionDelta>, Vec<FunctionDelta>) {
    let mut by_name_b: HashMap<&str, usize> = HashMap::new();
    for (i, h) in hb.iter().enumerate() {
        by_name_b.insert(&h.name, i);
    }
    let mut used_b = vec![false; hb.len()];

    let mut unchanged = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();
    let mut added = Vec::new();

    // First pass: exact name matches
    for (i, ha_i) in ha.iter().enumerate() {
        if let Some(&j) = by_name_b.get(ha_i.name.as_str()) {
            used_b[j] = true;
            let fb_i = &fb[j];
            let fa_i = &fa[i];
            if ha_i.blake3 == hb[j].blake3 {
                unchanged.push(FunctionDelta {
                    name_a: Some(ha_i.name.clone()),
                    name_b: Some(hb[j].name.clone()),
                    start_a: Some(ha_i.start),
                    start_b: Some(hb[j].start),
                    kind: MatchKind::Exact,
                    changed: false,
                    insn_count_a: Some(ha_i.n_insn),
                    insn_count_b: Some(hb[j].n_insn),
                    unified_diff: None,
                });
            } else {
                let ham = hamming(ha_i.simhash, hb[j].simhash);
                modified.push(FunctionDelta {
                    name_a: Some(ha_i.name.clone()),
                    name_b: Some(hb[j].name.clone()),
                    start_a: Some(ha_i.start),
                    start_b: Some(hb[j].start),
                    kind: MatchKind::Fuzzy { hamming: ham },
                    changed: true,
                    insn_count_a: Some(ha_i.n_insn),
                    insn_count_b: Some(hb[j].n_insn),
                    unified_diff: Some(unified_diff(&fa[i], fb_i)),
                });
            }
        } else {
            removed.push(FunctionDelta {
                name_a: Some(ha_i.name.clone()),
                name_b: None,
                start_a: Some(ha_i.start),
                start_b: None,
                kind: MatchKind::None,
                changed: true,
                insn_count_a: Some(ha_i.n_insn),
                insn_count_b: None,
                unified_diff: None,
            });
        }
    }

    // Second pass: functions in B not used yet are "added"
    for (j, hb_j) in hb.iter().enumerate() {
        if !used_b[j] {
            added.push(FunctionDelta {
                name_a: None,
                name_b: Some(hb_j.name.clone()),
                start_a: None,
                start_b: Some(hb_j.start),
                kind: MatchKind::None,
                changed: true,
                insn_count_a: None,
                insn_count_b: Some(hb_j.n_insn),
                unified_diff: None,
            });
        }
    }

    (added, removed, modified, unchanged)
}
