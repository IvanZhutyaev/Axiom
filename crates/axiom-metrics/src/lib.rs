//! Prometheus-compatible metrics registry (TZ §3.3 / §3.9).

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct Metrics {
    pub events_processed: AtomicU64,
    pub gc_pause_ms: AtomicU64,
    pub watermark_lag_ms: AtomicU64,
    pub raft_lag_entries: AtomicU64,
    pub latency_sum_us: AtomicU64,
    pub latency_count: AtomicU64,
}

impl Metrics {
    pub fn inc_events(&self, n: u64) {
        self.events_processed.fetch_add(n, Ordering::Relaxed);
    }

    pub fn record_latency_us(&self, us: u64) {
        self.latency_sum_us.fetch_add(us, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_watermark_lag_ms(&self, lag: u64) {
        self.watermark_lag_ms.store(lag, Ordering::Relaxed);
    }

    pub fn set_raft_lag(&self, entries: u64) {
        self.raft_lag_entries.store(entries, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let count = self.latency_count.load(Ordering::Relaxed);
        let sum = self.latency_sum_us.load(Ordering::Relaxed);
        MetricsSnapshot {
            events_processed: self.events_processed.load(Ordering::Relaxed),
            gc_pause_ms: self.gc_pause_ms.load(Ordering::Relaxed),
            watermark_lag_ms: self.watermark_lag_ms.load(Ordering::Relaxed),
            raft_lag_entries: self.raft_lag_entries.load(Ordering::Relaxed),
            latency_avg_us: if count > 0 { sum / count } else { 0 },
        }
    }

    pub fn render_prometheus(&self) -> String {
        let s = self.snapshot();
        format!(
            "# HELP axiom_events_processed_total Events processed\n\
             # TYPE axiom_events_processed_total counter\n\
             axiom_events_processed_total {}\n\
             # HELP axiom_gc_pause_ms_total GC pause milliseconds\n\
             # TYPE axiom_gc_pause_ms_total counter\n\
             axiom_gc_pause_ms_total {}\n\
             # HELP axiom_watermark_lag_ms Watermark lag\n\
             # TYPE axiom_watermark_lag_ms gauge\n\
             axiom_watermark_lag_ms {}\n\
             # HELP axiom_raft_lag_entries Raft replication lag\n\
             # TYPE axiom_raft_lag_entries gauge\n\
             axiom_raft_lag_entries {}\n\
             # HELP axiom_latency_avg_us Average processing latency\n\
             # TYPE axiom_latency_avg_us gauge\n\
             axiom_latency_avg_us {}\n",
            s.events_processed,
            s.gc_pause_ms,
            s.watermark_lag_ms,
            s.raft_lag_entries,
            s.latency_avg_us,
        )
    }
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub events_processed: u64,
    pub gc_pause_ms: u64,
    pub watermark_lag_ms: u64,
    pub raft_lag_entries: u64,
    pub latency_avg_us: u64,
}
