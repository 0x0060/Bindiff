use crate::arch::FunctionIR;
use petgraph::graph::Graph;
use petgraph::algo::is_isomorphic_matching;

// Very lightweight basic block graph based only on RET boundaries.
// This is a stub for future sophisticated CFG recovery.
pub fn cfg_isomorphic(a: &FunctionIR, b: &FunctionIR) -> bool {
    let to_blocks = |f: &FunctionIR| {
        let mut blocks = vec![0usize];
        for (i, insn) in f.insns.iter().enumerate() {
            if insn.mnemonic.starts_with("RET") {
                blocks.push(i + 1);
            }
        }
        blocks
    };
    let ab = to_blocks(a);
    let bb = to_blocks(b);
    // Build chain graphs
    let mut ga: Graph<usize, ()> = Graph::new();
    let mut last = None;
    for (i, _) in ab.iter().enumerate() {
        let n = ga.add_node(i);
        if let Some(l) = last { ga.add_edge(l, n, ()); }
        last = Some(n);
    }
    let mut gb: Graph<usize, ()> = Graph::new();
    last = None;
    for (i, _) in bb.iter().enumerate() {
        let n = gb.add_node(i);
        if let Some(l) = last { gb.add_edge(l, n, ()); }
        last = Some(n);
    }
    is_isomorphic_matching(&ga, &gb, |a, b| a == b, |_, _| true)
}
