use anyhow::{Result, bail};
use goblin::{Object, elf, pe};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSymbol {
    pub name: String,
    pub start: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryImage {
    pub path: String,
    pub arch: String,
    pub bits: u8,
    pub functions: Vec<FunctionSymbol>,
    pub data: Vec<u8>,
}

fn map_file(path: &Path) -> Result<Vec<u8>> {
    let f = File::open(path)?;
    let m = unsafe { Mmap::map(&f)? };
    Ok(m.as_ref().to_vec())
}

fn collect_elf(elf: &elf::Elf, bytes: &[u8]) -> Vec<FunctionSymbol> {
    let mut funs = Vec::new();
    for sym in elf.syms.iter() {
        let is_func = sym.st_type() == elf::sym::STT_FUNC;
        if !is_func || sym.st_size == 0 { continue; }
        if let Some(Ok(name)) = elf.strtab.get(sym.st_name) {
            funs.push(FunctionSymbol {
                name: name.to_string(),
                start: sym.st_value,
                size: sym.st_size,
            });
        }
    }
    funs
}

fn collect_pe(pe: &pe::PE, _bytes: &[u8]) -> Vec<FunctionSymbol> {
    // For PE, rely on export table and symbols if present. Fallbacks possible later.
    let mut funs = Vec::new();
    if let Some(exports) = &pe.exports {
        for e in exports {
            let name = e.name.unwrap_or_else(|| format!("ord_{}", e.rva));
            funs.push(FunctionSymbol {
                name,
                start: e.rva as u64,
                size: 0, // unknown; will rely on disassembler to bound
            });
        }
    }
    funs
}

pub fn load(path: &Path) -> Result<BinaryImage> {
    let data = map_file(path)?;
    match Object::parse(&data)? {
        Object::Elf(elf) => {
            let funs = collect_elf(&elf, &data);
            Ok(BinaryImage {
                path: path.display().to_string(),
                arch: "x86".to_string(),
                bits: if elf.is_64 { 64 } else { 32 },
                functions: funs,
                data,
            })
        },
        Object::PE(pe) => {
            let funs = collect_pe(&pe, &data);
            // Assume 64-bit if optional header says so
            let bits = if let Some(opt) = pe.header.optional_header {
                if opt.standard_fields.magic == 0x20b { 64 } else { 32 }
            } else { 64 };
            Ok(BinaryImage {
                path: path.display().to_string(),
                arch: "x86".to_string(),
                bits,
                functions: funs,
                data,
            })
        },
        other => bail!("Unsupported object format: {:?}", other),
    }
}
