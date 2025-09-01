# Bindiff

A fast, memory-safe **function-level binary diffing** tool written in Rust (Linux + Windows).

## Features
- Parses **ELF** and **PE** (via `goblin`)
- Disassembles **x86_64** using **pure-Rust** `yaxpeax-x86`
- Normalizes operands (REG/MEM/IMM) to reduce false deltas
- Computes **BLAKE3** hash and **SimHash** per function
- Matches functions by name; classifies **unchanged / modified / added / removed**
- Generates **JSON** and **HTML** reports
- Parallel-ready and memory-safe by design

> Note: Capstone integration and CFG-based graph matching can be added later as optional features.

## Build
```bash
cargo build --release
```

## Usage
```bash
# Diff two x86_64 binaries
bindiff ./old.bin ./new.bin -o result.json -H result.html
```

## Roadmap
- Add optional **capstone** feature for more architectures
- Improve function discovery for stripped binaries
- CFG recovery and VF2 isomorphism-based matching
- String/call-reference signature matching
