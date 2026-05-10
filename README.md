# cost-meter

[![crates.io](https://img.shields.io/crates/v/cost-meter.svg)](https://crates.io/crates/cost-meter)
[![docs.rs](https://img.shields.io/docsrs/cost-meter)](https://docs.rs/cost-meter)

Aggregate LLM API cost across providers, models, and time windows.
Provider-agnostic — pairs with
[`claude-cost`](https://crates.io/crates/claude-cost),
[`openai-cost`](https://crates.io/crates/openai-cost),
[`gemini-cost`](https://crates.io/crates/gemini-cost), and
[`bedrock-cost`](https://crates.io/crates/bedrock-cost).

## Why

Every team that runs LLMs in production rebuilds the same dashboard: a
counter of calls and dollars, broken down by provider and model. This is
the small, dependency-free piece that does the counting.

## Usage

```rust
use cost_meter::{Meter, Call};

let mut meter = Meter::new();

meter.record(Call {
    provider: "anthropic",
    model: "claude-sonnet-4-5",
    input_tokens: 1_000,
    output_tokens: 500,
    cost_usd: 0.0105,
});
meter.record(Call {
    provider: "openai",
    model: "gpt-5",
    input_tokens: 2_000,
    output_tokens: 800,
    cost_usd: 0.0105,
});

let s = meter.snapshot();
println!("total spend: ${:.4} over {} calls", s.total_cost_usd, s.total_calls);

for (provider, b) in meter.by_provider() {
    println!("{provider}: {} calls, ${:.4}", b.calls, b.cost_usd);
}
```

Pair it with one of the pricing crates:

```rust,ignore
use claude_cost::{Usage, default_pricing};
use cost_meter::{Meter, Call};

let model = "claude-sonnet-4-5";
let pricing = default_pricing(model).unwrap();
let usage = Usage { input_tokens: 1000, output_tokens: 500, ..Default::default() };
let cost = pricing.cost_for(&usage);

let mut m = Meter::new();
m.record(Call {
    provider: "anthropic",
    model,
    input_tokens: usage.input_tokens,
    output_tokens: usage.output_tokens,
    cost_usd: cost,
});
```

## Features

- `serde` — derive `Serialize`/`Deserialize` on `Bucket` and `Snapshot`.

## License

MIT or Apache-2.0.
