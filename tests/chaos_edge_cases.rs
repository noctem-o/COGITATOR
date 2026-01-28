use cogitator::chaos::{
    apply_fault, ChaosEngine, ChaosProfile, FaultKind, FaultParams, FaultRates, FaultRecord,
};
use cogitator::tooling::{ToolRequest, ToolResponse};
use serde_json::Value;

#[test]
fn corrupt_value_does_not_panic_on_empty_or_non_ascii_strings() {
    let req = ToolRequest {
        tool_name: "dummy".to_string(),
        arguments: Value::Null,
    };

    for s in ["", "你好世界", "🚀🚀🚀"] {
        let resp = ToolResponse {
            tool_name: "dummy".to_string(),
            output: Value::String(s.to_string()),
            success: true,
            simulated_latency_ms: None,
        };

        let fault = FaultRecord {
            kind: FaultKind::Corrupt,
            step: 0,
            tool_call_idx: 0,
            domain: "dummy".to_string(),
            params: FaultParams {
                mask: Some(0xdead_beef),
                latency_ms: None,
            },
        };

        let out = apply_fault(&req, resp, &fault).expect("apply_fault should not fail");
        assert!(matches!(out.output, Value::String(_)));
    }
}

#[test]
fn decide_fault_caps_total_rate_at_per_million() {
    let profile = ChaosProfile {
        schema_version: 1,
        schedule_version: 1,
        enabled: true,
        profile: "test".to_string(),
        seed: 42,
        rates: FaultRates {
            timeout_per_million: 900_000,
            drop_per_million: 900_000,
            corrupt_per_million: 900_000,
            latency_sim_per_million: 900_000,
        },
    };

    let engine = ChaosEngine::new(profile, 0);
    let fault = engine.decide_fault(1, 2, "dummy_tool");
    assert!(
        fault.is_some(),
        "fault should always be selected when total caps to 1,000,000"
    );
}
