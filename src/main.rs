use anyhow::{Context, Result};
use clap::Parser;
use csv::Writer;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

// Concurrency
use rayon::prelude::*;

// Optional TUI dependencies. These are disabled when the `no_tui` flag is
// passed. If you enable the TUI, ensure that `ratatui` and `crossterm`
// are added as dependencies in your Cargo.toml.
#[cfg(feature = "tui")]
use {
    crossterm::event::{self, Event as CEvent, KeyCode},
    ratatui::prelude::*,
    ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

/// Command‑line arguments for the improved harness.  A new `--parallel`
/// switch allows users to override the default parallel execution.
#[derive(Parser, Debug, Clone)]
#[command(name = "cogitator", version, about = "Enhanced deterministic evaluation harness for agents.")]
pub struct Args {
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
    /// Enable or disable parallel execution.  Parallelism dramatically
    /// reduces evaluation time by distributing runs across cores.  The
    /// HAL harness demonstrates how orchestrating parallel evaluations
    /// across hundreds of machines can reduce evaluation from weeks to
    /// hours【343845492165604†L26-L31】, and this flag enables a similar
    /// effect locally.
    #[arg(long, default_value_t = true)]
    parallel: bool,
}

/// High‑level representation of a single test case.
#[derive(Debug, Clone)]
pub struct CaseResult {
    pub run_id: u32,
    pub case_id: String,
    pub difficulty: f32,
    pub score: f32,
    pub passed: bool,
    pub thoughts: Vec<ThoughtEvent>,
}

/// A thought or action emitted during evaluation.  Each event may
/// correspond to a reasoning step, tool call, observation or any
/// intermediate output.  Exposing thought traces is important for
/// auditable agentic systems that must trace the complete lifecycle
/// from intent through reasoning to outcome【356664820513273†L58-L70】.
#[derive(Debug, Clone)]
pub struct ThoughtEvent {
    pub step: usize,
    pub role: String,
    pub content: String,
}

/// Summary statistics aggregated over all runs.
#[derive(Debug, Clone)]
pub struct Summary {
    pub pass_rate: f32,
    pub avg_score: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let results = if args.parallel {
        run_evaluation_parallel(args.seed, args.runs)
    } else {
        run_evaluation_sequential(args.seed, args.runs)
    };
    write_results(&args.output, &results).with_context(|| "failed to write results")?;
    let summary = summarize(&results);
    if !args.no_tui {
        #[cfg(feature = "tui")]
        render_tui(&args, &results, &summary)?;
        #[cfg(not(feature = "tui"))]
        println!("Terminal UI disabled or missing tui feature. Summary: {:?}", summary);
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

/// Sequential evaluation used when `--parallel=false`.
fn run_evaluation_sequential(seed: u64, runs: u32) -> Vec<CaseResult> {
    (0..runs)
        .map(|run_id| evaluate_case(seed, run_id))
        .collect()
}

/// Parallel evaluation that distributes cases across available cores.  The
/// inspiration comes from the HAL evaluation harness, which reduces
/// evaluation time by orchestrating parallel evaluations across VMs【343845492165604†L26-L31】.
fn run_evaluation_parallel(seed: u64, runs: u32) -> Vec<CaseResult> {
    (0..runs)
        .into_par_iter()
        .map(|run_id| evaluate_case(seed, run_id))
        .collect()
}

/// Deterministically evaluate a single test case.  In addition to the
/// original computation of case ID, difficulty, score and pass/fail
/// determination, this function synthesizes a small thought trace to
/// demonstrate how reasoning and actions might be captured.  In a
/// production harness, these would be real agent thoughts/actions
/// collected from loggers as described in the HAL log‑inspection
/// framework【947121429427184†L110-L124】.
fn evaluate_case(seed: u64, run_id: u32) -> CaseResult {
    let digest = hash_seed(seed, run_id);
    let case_id = to_hex(&digest);
    let difficulty = digest[0] as f32 / 255.0;
    let rng_seed = u64::from_le_bytes(digest[..8].try_into().unwrap());
    let mut rng = StdRng::seed_from_u64(rng_seed);
    let base = 0.45 + rng.gen_range(0.0..0.55);
    let score = (base * (1.0 - difficulty)).clamp(0.0, 1.0);
    let passed = score >= 0.5;
    // Synthesize a simple thought trace demonstrating reasoning and action.
    let thoughts = vec![
        ThoughtEvent {
            step: 0,
            role: "system".into(),
            content: format!("Initializing evaluation with difficulty {:.2}", difficulty),
        },
        ThoughtEvent {
            step: 1,
            role: "assistant".into(),
            content: format!("Generated score {:.3}", score),
        },
        ThoughtEvent {
            step: 2,
            role: "assistant".into(),
            content: if passed {
                "Decision: PASS".to_string()
            } else {
                "Decision: FAIL".to_string()
            },
        },
    ];
    CaseResult {
        run_id,
        case_id,
        difficulty,
        score,
        passed,
        thoughts,
    }
}

/// Persist evaluation results to a CSV file.
fn write_results(path: &PathBuf, results: &[CaseResult]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(["run_id", "case_id", "difficulty", "score", "passed"])?;
    for result in results {
        writer.write_record([
            result.run_id.to_string(),
            result.case_id.clone(),
            format!("{:.3}", result.difficulty),
            format!("{:.3}", result.score),
            result.passed.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

/// Compute summary statistics.
fn summarize(results: &[CaseResult]) -> Summary {
    let total = results.len() as f32;
    let pass_count = results.iter().filter(|r| r.passed).count() as f32;
    let avg_score = results.iter().map(|r| r.score).sum::<f32>() / total.max(1.0);
    Summary {
        pass_rate: pass_count / total.max(1.0),
        avg_score,
    }
}

/// Hash the seed and run_id to derive a deterministic digest.
fn hash_seed(seed: u64, run_id: u32) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_le_bytes());
    hasher.update(run_id.to_le_bytes());
    hasher.finalize().into()
}

/// Convert a 32‑byte digest into a hexadecimal string.
fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Placeholder TUI renderer.  This function demonstrates how to use
/// `ratatui` to build a structured terminal dashboard with multiple
/// panels.  It displays summary statistics, hardware information,
/// aggregated results, and a scrollable list of thoughts for the
/// currently selected run.  Real‑time updates and interactive
/// navigation would require additional state management and event
/// handling.
#[cfg(feature = "tui")]
fn render_tui(args: &Args, results: &[CaseResult], summary: &Summary) -> Result<()> {
    // Build a simple application state.  In a full implementation,
    // this would track the selected run and scroll offsets.
    let mut selected = 0usize;
    // Initialize the terminal via ratatui helper.
    ratatui::run(|mut terminal| {
        loop {
            // Draw the UI.
            terminal.draw(|f| {
                let size = f.size();
                // Split the screen into vertical chunks.
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                    .split(size);
                // Left panel: summary and list of runs.
                let left_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(5), // summary
                        Constraint::Min(0),    // runs list
                    ])
                    .split(chunks[0]);
                // Summary block.
                let summary_text = vec![
                    Line::from(format!("Pass rate: {:.2}%", summary.pass_rate * 100.0)),
                    Line::from(format!("Avg score: {:.3}", summary.avg_score)),
                ];
                let summary_paragraph = Paragraph::new(summary_text)
                    .block(Block::default().borders(Borders::ALL).title("Summary"));
                f.render_widget(summary_paragraph, left_chunks[0]);
                // Runs list.
                let items: Vec<ListItem> = results
                    .iter()
                    .map(|r| {
                        ListItem::new(format!("#{:04}  {:.3}  {}", r.run_id, r.score, if r.passed { "✓" } else { "✗" }))
                    })
                    .collect();
                let runs_list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Runs"))
                    .highlight_symbol("▶ ");
                f.render_stateful_widget(runs_list, left_chunks[1], &mut ListState::default());
                // Right panel: details of selected run.
                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // header
                        Constraint::Min(0),    // thoughts list
                    ])
                    .split(chunks[1]);
                let selected_result = results.get(selected).unwrap_or(&results[0]);
                let header_text = vec![
                    Line::from(format!("Run {} — score {:.3} — {}", selected_result.run_id, selected_result.score, if selected_result.passed { "PASS" } else { "FAIL" })),
                    Line::from(format!("Difficulty: {:.2}", selected_result.difficulty)),
                ];
                let header_block = Paragraph::new(header_text)
                    .block(Block::default().borders(Borders::ALL).title("Run Details"));
                f.render_widget(header_block, right_chunks[0]);
                // Thoughts list.
                let thought_items: Vec<ListItem> = selected_result
                    .thoughts
                    .iter()
                    .map(|ev| {
                        ListItem::new(format!("{}: {}", ev.role, ev.content))
                    })
                    .collect();
                let thoughts_list = List::new(thought_items)
                    .block(Block::default().borders(Borders::ALL).title("Thoughts / Actions"));
                f.render_widget(thoughts_list, right_chunks[1]);
            })?;
            // Handle input events.
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Down => {
                        if selected + 1 < results.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    })
}
