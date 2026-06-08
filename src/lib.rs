//! # cost-meter
//!
//! Aggregate LLM API cost across providers, models, and time windows.
//!
//! Provider-agnostic: you compute the per-call cost with whatever pricing
//! crate you like (`claude-cost`, `openai-cost`, `gemini-cost`,
//! `bedrock-cost`, or BYO) and feed `(provider, model, tokens, cost)`
//! into the meter. The meter keeps running totals broken down by
//! provider and model, and exposes them as a sorted snapshot.
//!
//! ## Example
//!
//! ```
//! use cost_meter::{Meter, Call};
//!
//! let mut meter = Meter::new();
//! meter.record(Call {
//!     provider: "anthropic",
//!     model: "claude-sonnet-4-5",
//!     input_tokens: 1_000,
//!     output_tokens: 500,
//!     cost_usd: 0.0105,
//! });
//! meter.record(Call {
//!     provider: "openai",
//!     model: "gpt-5",
//!     input_tokens: 2_000,
//!     output_tokens: 800,
//!     cost_usd: 0.0105,
//! });
//!
//! let snap = meter.snapshot();
//! assert_eq!(snap.total_calls, 2);
//! assert!((snap.total_cost_usd - 0.021).abs() < 1e-9);
//!
//! // Per-provider breakdown is sorted by cost desc.
//! let by_provider = meter.by_provider();
//! assert_eq!(by_provider.len(), 2);
//! ```

#![deny(missing_docs)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

/// A single LLM call to record against the meter.
#[derive(Debug, Clone, Copy)]
pub struct Call<'a> {
    /// Provider name. Free-form; typical values: `anthropic`, `openai`,
    /// `google`, `bedrock`.
    pub provider: &'a str,
    /// Model id as you call it (need not be normalized).
    pub model: &'a str,
    /// Fresh input tokens billed on this call.
    pub input_tokens: u64,
    /// Output tokens billed on this call.
    pub output_tokens: u64,
    /// Pre-computed USD cost for this call.
    pub cost_usd: f64,
}

/// Aggregated counters for a single (provider, model) pair.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bucket {
    /// Number of calls counted.
    pub calls: u64,
    /// Total input tokens.
    pub input_tokens: u64,
    /// Total output tokens.
    pub output_tokens: u64,
    /// Total USD cost.
    pub cost_usd: f64,
}

impl Bucket {
    fn add_call(&mut self, c: &Call<'_>) {
        self.calls += 1;
        self.input_tokens += c.input_tokens;
        self.output_tokens += c.output_tokens;
        self.cost_usd += c.cost_usd;
    }
}

/// Top-level snapshot of running totals.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Snapshot {
    /// Total calls recorded.
    pub total_calls: u64,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Total output tokens.
    pub total_output_tokens: u64,
    /// Total USD cost.
    pub total_cost_usd: f64,
}

/// Cost aggregator. Cheap to construct; cheap to record.
#[derive(Debug, Default)]
pub struct Meter {
    by_pm: BTreeMap<(String, String), Bucket>,
    snap: Snapshot,
}

impl Meter {
    /// Create an empty meter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one call.
    pub fn record(&mut self, c: Call<'_>) {
        let key = (c.provider.to_string(), c.model.to_string());
        let bucket = self.by_pm.entry(key).or_default();
        bucket.add_call(&c);
        self.snap.total_calls += 1;
        self.snap.total_input_tokens += c.input_tokens;
        self.snap.total_output_tokens += c.output_tokens;
        self.snap.total_cost_usd += c.cost_usd;
    }

    /// Current snapshot of grand totals.
    pub fn snapshot(&self) -> Snapshot {
        self.snap.clone()
    }

    /// Buckets grouped by provider, sorted by cost desc.
    pub fn by_provider(&self) -> Vec<(String, Bucket)> {
        let mut acc: BTreeMap<String, Bucket> = BTreeMap::new();
        for ((provider, _), b) in self.by_pm.iter() {
            let target = acc.entry(provider.clone()).or_default();
            target.calls += b.calls;
            target.input_tokens += b.input_tokens;
            target.output_tokens += b.output_tokens;
            target.cost_usd += b.cost_usd;
        }
        let mut v: Vec<(String, Bucket)> = acc.into_iter().collect();
        v.sort_by(|a, b| {
            b.1.cost_usd
                .partial_cmp(&a.1.cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    /// Buckets grouped by (provider, model), sorted by cost desc.
    pub fn by_model(&self) -> Vec<((String, String), Bucket)> {
        let mut v: Vec<((String, String), Bucket)> = self
            .by_pm
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        v.sort_by(|a, b| {
            b.1.cost_usd
                .partial_cmp(&a.1.cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    /// Reset the meter to empty.
    pub fn reset(&mut self) {
        self.by_pm.clear();
        self.snap = Snapshot::default();
    }
}
