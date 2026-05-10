use cost_meter::{Call, Meter};

fn call(provider: &'static str, model: &'static str, input: u64, output: u64, cost: f64) -> Call<'static> {
    Call {
        provider,
        model,
        input_tokens: input,
        output_tokens: output,
        cost_usd: cost,
    }
}

#[test]
fn empty_meter_is_zero() {
    let m = Meter::new();
    let s = m.snapshot();
    assert_eq!(s.total_calls, 0);
    assert_eq!(s.total_cost_usd, 0.0);
    assert!(m.by_provider().is_empty());
    assert!(m.by_model().is_empty());
}

#[test]
fn snapshot_accumulates() {
    let mut m = Meter::new();
    m.record(call("anthropic", "claude-sonnet-4-5", 1000, 500, 0.0105));
    m.record(call("openai", "gpt-5", 2000, 800, 0.0105));
    let s = m.snapshot();
    assert_eq!(s.total_calls, 2);
    assert_eq!(s.total_input_tokens, 3000);
    assert_eq!(s.total_output_tokens, 1300);
    assert!((s.total_cost_usd - 0.021).abs() < 1e-9);
}

#[test]
fn by_provider_sums_across_models() {
    let mut m = Meter::new();
    m.record(call("anthropic", "claude-sonnet-4-5", 1000, 500, 0.01));
    m.record(call("anthropic", "claude-haiku-4-5", 100, 50, 0.001));
    m.record(call("openai", "gpt-5", 500, 200, 0.005));
    let v = m.by_provider();
    assert_eq!(v.len(), 2);
    let anthropic = v.iter().find(|(p, _)| p == "anthropic").unwrap();
    assert_eq!(anthropic.1.calls, 2);
    assert!((anthropic.1.cost_usd - 0.011).abs() < 1e-9);
}

#[test]
fn by_provider_sorted_by_cost_desc() {
    let mut m = Meter::new();
    m.record(call("a", "x", 0, 0, 1.0));
    m.record(call("b", "y", 0, 0, 5.0));
    m.record(call("c", "z", 0, 0, 3.0));
    let v = m.by_provider();
    assert_eq!(v[0].0, "b");
    assert_eq!(v[1].0, "c");
    assert_eq!(v[2].0, "a");
}

#[test]
fn by_model_keeps_provider_distinction() {
    let mut m = Meter::new();
    m.record(call("a", "shared", 0, 0, 1.0));
    m.record(call("b", "shared", 0, 0, 2.0));
    let v = m.by_model();
    assert_eq!(v.len(), 2, "two providers should produce two model rows even with same model name");
}

#[test]
fn reset_clears_everything() {
    let mut m = Meter::new();
    m.record(call("a", "x", 100, 50, 0.5));
    m.reset();
    assert_eq!(m.snapshot().total_calls, 0);
    assert!(m.by_model().is_empty());
}
