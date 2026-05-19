//! Prometheus-compatible metrics registry.

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct Metrics {
    pub events_processed: AtomicU64,
    pub gc_pause_ms: AtomicU64,
}

impl Metrics {
    pub fn inc_events(&self, n: u64) {
        self.events_processed.fetch_add(n, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            events_processed: self.events_processed.load(Ordering::Relaxed),
            gc_pause_ms: self.gc_pause_ms.load(Ordering::Relaxed),
        }
    }

    pub fn render_prometheus(&self) -> String {
        let s = self.snapshot();
        format!(
            "# HELP axiom_events_processed_total Events processed\n\
             # TYPE axiom_events_processed_total counter\n\
             axiom_events_processed_total {}\n",
            s.events_processed
        )
    }
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub events_processed: u64,
    pub gc_pause_ms: u64,
}
