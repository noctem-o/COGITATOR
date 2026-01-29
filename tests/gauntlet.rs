use cogitator::gauntlet::{GauntletConfig, TaskSuite, GAUNTLET_TASKS_PATH};
use cogitator::model::WitnessedMetadata;
use cogitator::report::DriftIssue;
use cogitator::tooling::ToolTranscript;
use cogitator::{drift, trace};
use rayon::ThreadPoolBuilder;

fn run_gauntlet_root(seed: u64, pass_threshold: &str) -> String {
    let suite = TaskSuite::load(std::path::Path::new(GAUNTLET_TASKS_PATH)).expect("suite");
    let mut transcript = ToolTranscript::new_live(None);
    let config = GauntletConfig {
        seed,
        run_id: 0,
        case_id: "case".to_string(),
        pass_threshold_f32: pass_threshold.parse().expect("threshold"),
        pass_threshold_witnessed: format!(
            "f32:0x{:08X}",
            pass_threshold.parse::<f32>().expect("threshold").to_bits()
        ),
        regress: false,
    };
    let output =
        cogitator::gauntlet::run_gauntlet(&suite, &config, &mut transcript).expect("gauntlet run");
    let record = transcript.into_record();
    let metadata = WitnessedMetadata {
        schema_version: cogitator::model::TRACE_SCHEMA_VERSION,
        seed,
        requested_runs: 1,
        executed_runs: 1,
        parallel: false,
        parallel_strategy: "sequential".to_string(),
        case_filter: Some(0),
        entropy_sources: vec![
            "rng:StdRng(seed)".to_string(),
            "tooling:stubbed-or-replay".to_string(),
        ],
        total_rng_calls: output.total_rng_calls,
        chaos_profile: None,
        pass_threshold: Some(format!(
            "f32:0x{:08X}",
            pass_threshold.parse::<f32>().expect("threshold").to_bits()
        )),
    };
    trace::compute_agent_witness_root(&metadata, &output.agent_trace, &record.entries)
        .expect("witness root")
}

#[test]
fn gauntlet_task_loader_validates_count() {
    let suite = TaskSuite::load(std::path::Path::new(GAUNTLET_TASKS_PATH)).expect("suite");
    assert_eq!(suite.tasks.len(), 50);
    assert_eq!(suite.tasks.first().map(|task| task.task_id), Some(0));
    assert_eq!(suite.tasks.last().map(|task| task.task_id), Some(49));
}

#[test]
fn gauntlet_witness_root_thread_invariant() {
    let roots: Vec<String> = [1usize, 16]
        .iter()
        .map(|threads| {
            ThreadPoolBuilder::new()
                .num_threads(*threads)
                .build()
                .unwrap()
                .install(|| run_gauntlet_root(42, "0.5"))
        })
        .collect();
    assert!(roots.iter().all(|root| root == &roots[0]));
}

#[test]
fn gauntlet_witness_root_changes_with_threshold() {
    let root_a = run_gauntlet_root(42, "0.346");
    let root_b = run_gauntlet_root(42, "0.5");
    assert_ne!(root_a, root_b);
}

#[test]
fn gauntlet_replay_regression_reports_drift() {
    let suite = TaskSuite::load(std::path::Path::new(GAUNTLET_TASKS_PATH)).expect("suite");
    let config = GauntletConfig {
        seed: 42,
        run_id: 0,
        case_id: "case".to_string(),
        pass_threshold_f32: 0.5,
        pass_threshold_witnessed: "f32:0x3F000000".to_string(),
        regress: false,
    };
    let mut live_transcript = ToolTranscript::new_live(None);
    let _live_output =
        cogitator::gauntlet::run_gauntlet(&suite, &config, &mut live_transcript).expect("live");
    let live_record = live_transcript.into_record();

    let mut replay_transcript = ToolTranscript::new_replay(live_record.clone());
    let config_regressed = GauntletConfig {
        regress: true,
        ..config
    };
    let replay_output =
        cogitator::gauntlet::run_gauntlet(&suite, &config_regressed, &mut replay_transcript)
            .expect("replay");
    let replay_record = replay_transcript.into_record();

    let mut report = drift::detect_transcript_drift(&live_record, &replay_record);
    report.issues.extend(replay_output.issues);
    report.drifted = report.drifted || !report.issues.is_empty();

    assert!(report.drifted);
    let has_gauntlet_issue = report.issues.iter().any(|issue| match issue {
        DriftIssue::GauntletOutputMismatch {
            step,
            tool_name,
            json_pointer,
            issue_kind,
            expected,
            actual,
            ..
        } => {
            *step == 0
                && tool_name == "gauntlet.lookup"
                && json_pointer == "/payload/tags/0"
                && issue_kind == "missing"
                && !expected.is_empty()
                && actual == "missing"
        }
        _ => false,
    });
    assert!(has_gauntlet_issue);
}
