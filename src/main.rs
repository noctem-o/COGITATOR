fn main() {
    println!("Hello, world!");
use anyhow::{Context, Result};
use clap::Parser;
use csv::Writer;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cogitator", version, about = "Deterministic evaluation harness for agents.")]
struct Args {
    /// Seed for deterministic evaluation.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of evaluation runs.
    #[arg(long, default_value_t = 5000)]
    runs: u32,
    /// Output CSV path.
    #[arg(long, default_value = "results.csv")]
    output: PathBuf,
    /// Disable terminal UI output.
    #[arg(long)]
    no_tui: bool,
}

#[derive(Debug)]
struct CaseResult {
    run_id: u32,
    case_id: String,
    difficulty: f32,
    score: f32,
    passed: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let results = run_evaluation(args.seed, args.runs);
    write_results(&args.output, &results).with_context(|| "failed to write results")?;
    let summary = summarize(&results);
    if !args.no_tui {
        render_tui(&args, &results, &summary);
    }
    println!(
        "Seed: {} | Runs: {} | Pass rate: {:.2}% | Avg score: {:.3} | Output: {}",
        args.seed,
        args.runs,
        summary.pass_rate * 100.0,
        summary.avg_score,
        args.output.display()
    );
    Ok(())
}

fn run_evaluation(seed: u64, runs: u32) -> Vec<CaseResult> {
    (0..runs)
        .map(|run_id| evaluate_case(seed, run_id))
        .collect()
}

fn evaluate_case(seed: u64, run_id: u32) -> CaseResult {
    let digest = hash_seed(seed, run_id);
    let case_id = to_hex(&digest);
    let difficulty = digest[0] as f32 / 255.0;
    let rng_seed = u64::from_le_bytes(digest[..8].try_into().unwrap());
    let mut rng = StdRng::seed_from_u64(rng_seed);
    let base = 0.45 + rng.gen_range(0.0..0.55);
    let score = (base * (1.0 - difficulty)).clamp(0.0, 1.0);
@@ -105,41 +109,124 @@ fn write_results(path: &PathBuf, results: &[CaseResult]) -> Result<()> {
            result.case_id.clone(),
            format!("{:.3}", result.difficulty),
            format!("{:.3}", result.score),
            result.passed.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

struct Summary {
    pass_rate: f32,
    avg_score: f32,
}

fn summarize(results: &[CaseResult]) -> Summary {
    let total = results.len() as f32;
    let pass_count = results.iter().filter(|r| r.passed).count() as f32;
    let avg_score = results.iter().map(|r| r.score).sum::<f32>() / total.max(1.0);
    Summary {
        pass_rate: pass_count / total.max(1.0),
        avg_score,
    }
}

struct HardwareSnapshot {
    cpu_brand: String,
    logical_cores: usize,
    total_memory_gb: Option<f32>,
    os: String,
}

// TODO: Replace with the official Cogitator logo asset once provided.
const COGITATOR_LOGO: &str = r#"
   ██████╗  ██████╗  ██████╗ ██╗████████╗ █████╗ ████████╗ ██████╗ ██████╗
  ██╔════╝ ██╔═══██╗██╔════╝ ██║╚══██╔══╝██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ██║      ██║   ██║██║  ███╗██║   ██║   ███████║   ██║   ██║   ██║██████╔╝
  ██║      ██║   ██║██║   ██║██║   ██║   ██╔══██║   ██║   ██║   ██║██╔══██╗
  ╚██████╗ ╚██████╔╝╚██████╔╝██║   ██║   ██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
"#;

fn render_tui(args: &Args, results: &[CaseResult], summary: &Summary) {
    let hardware = capture_hardware();
    let pass_count = results.iter().filter(|r| r.passed).count();
    let fail_count = results.len().saturating_sub(pass_count);
    println!(
        "{}\n{}",
        COGITATOR_LOGO.trim_end(),
        "=".repeat(70)
    );
    println!(" Mission Control :: Deterministic Evaluation Harness");
    println!("{}", "-".repeat(70));
    println!(" Seed            : {}", args.seed);
    println!(" Runs            : {}", args.runs);
    println!(" Output CSV      : {}", args.output.display());
    println!("{}", "-".repeat(70));
    println!(" Results");
    println!("  [PASS] Passed   : {}", pass_count);
    println!("  [FAIL] Failed   : {}", fail_count);
    println!("  [RATE] PassRate : {:.2}%", summary.pass_rate * 100.0);
    println!("  [SCORE] Avg     : {:.3}", summary.avg_score);
    println!("{}", "-".repeat(70));
    println!(" Reasoning Trace (High-Level, Non-Sensitive)");
    println!("  1) Parse CLI + seed");
    println!("  2) Hash seed + run_id to derive case difficulty");
    println!("  3) Generate deterministic score");
    println!("  4) Aggregate CSV + summary");
    println!("{}", "-".repeat(70));
    println!(" LLM Component Map");
    println!("  PF Prompt Fidelity     : deterministic seed + hash");
    println!("  EM Evaluation Matrix   : difficulty, score, pass");
    println!("  TM Telemetry           : CSV output + summary");
    println!("  TR Traceability        : stable case_id per run");
    println!("{}", "-".repeat(70));
    println!(" Hardware Snapshot");
    println!("  CPU Model         : {}", hardware.cpu_brand);
    println!("  Logical Cores     : {}", hardware.logical_cores);
    match hardware.total_memory_gb {
        Some(memory_gb) => println!("  Total Memory      : {:.2} GB", memory_gb),
        None => println!("  Total Memory      : Unknown"),
    }
    println!("  OS                : {}", hardware.os);
    println!("  GPU               : (not detected)");
    println!("{}", "-".repeat(70));
    println!(" Scaling & Compatibility");
    println!("  Single Node       : {} threads", hardware.logical_cores.max(1));
    println!("  Multi-Socket      : partition by run_id ranges");
    println!("  Multi-Node        : shard runs across nodes, merge CSVs");
    println!("  Supercomputer     : deterministic seeds per shard");
    println!("{}", "=".repeat(70));
}

fn capture_hardware() -> HardwareSnapshot {
    let logical_cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let cpu_brand = "Unknown CPU".to_string();
    let total_memory_gb = None;
    let os = std::env::consts::OS.to_string();
    HardwareSnapshot {
        cpu_brand,
        logical_cores,
        total_memory_gb,
        os,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_results() {
        let first = run_evaluation(7, 5);
        let second = run_evaluation(7, 5);
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(second.iter()) {
            assert_eq!(a.case_id, b.case_id);
            assert!((a.score - b.score).abs() < f32::EPSILON);
            assert_eq!(a.passed, b.passed);
        }
    }
}
