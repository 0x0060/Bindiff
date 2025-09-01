use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use bindiff_core::{loader, arch::x86_64, analysis::hash::hash_function, diff::{matching::match_functions, DiffResult}};
use serde_json;

#[derive(Parser, Debug)]
#[command(name = "bindiff", about = "Function-level binary diffing (x86_64, ELF & PE)")]
struct Args {
    /// Old/left binary path
    a: PathBuf,
    /// New/right binary path
    b: PathBuf,

    /// Output JSON report to file
    #[arg(short, long)]
    out_json: Option<PathBuf>,

    /// Output HTML report to file
    #[arg(short='H', long)]
    out_html: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg}")?);
    pb.set_message("Loading binaries...");
    let bin_a = loader::load(&args.a)?;
    let bin_b = loader::load(&args.b)?;

    pb.set_message("Disassembling functions...");
    let fa = x86_64::disassemble_functions(&bin_a)?;
    let fb = x86_64::disassemble_functions(&bin_b)?;

    pb.set_message("Hashing...");
    let ha: Vec<_> = fa.iter().map(hash_function).collect();
    let hb: Vec<_> = fb.iter().map(hash_function).collect();

    pb.set_message("Matching...");
    let (added, removed, modified, unchanged) = match_functions(&fa, &fb, &ha, &hb);
    pb.finish_and_clear();

    println!("{}", "=== Summary ===".bold());
    println!("  {} {}", "Unchanged:".green(), unchanged.len());
    println!("  {} {}", "Modified:".yellow(), modified.len());
    println!("  {} {}", "Added:".blue(), added.len());
    println!("  {} {}", "Removed:".red(), removed.len());
    println!();

    for m in &modified {
        let name = format!("{} -> {}", m.name_a.as_deref().unwrap_or("?"), m.name_b.as_deref().unwrap_or("?"));
        println!("{} {}", "MOD".yellow().bold(), name);
    }

    let result = DiffResult { added, removed, modified, unchanged };

    if let Some(json_path) = args.out_json {
        let jr: bindiff_report::JsonReport = result.clone().into();
        std::fs::write(&json_path, serde_json::to_string_pretty(&jr)?)?;
        println!("Wrote JSON report to {}", json_path.display());
    }

    if let Some(html_path) = args.out_html {
        let html = bindiff_report::render_html(&result)?;
        std::fs::write(&html_path, html)?;
        println!("Wrote HTML report to {}", html_path.display());
    }

    Ok(())
}
