// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing some predefined useful metrics.

pub use prometheus_client::metrics::{counter, gauge};
use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder},
    metrics::{gauge::Gauge, MetricType},
};
use tokio::process::Command;

/// Metric that tracks the memory used by a process in Kilobytes.
///
/// On unix-like platforms this metric takes the RSS (resident set size) reported by the `ps`
/// command.
#[derive(Clone)]
pub struct MemoryUsage {
    gauge: Gauge,
    pid: String,
}

impl MemoryUsage {
    /// Creates a new metric for the desired PID.
    pub fn new(pid: u32) -> Self {
        Self {
            gauge: Gauge::default(),
            pid: pid.to_string(),
        }
    }

    /// Updates the memory value tracked by the metric.
    ///
    /// The value is not updated if the command used to retrieve the new value:
    ///  - cannot be spawned,
    ///  - returns an unsuccessful exit code,
    ///  - its output cannot be parsed.
    pub async fn update(&self) {
        if cfg!(unix) {
            if let Ok(output) = Command::new("ps")
                .arg("-o")
                .arg("rss=")
                .arg("--pid")
                .arg(&self.pid)
                .output()
                .await
            {
                if output.status.success() {
                    if let Ok(stdout) = String::from_utf8(output.stdout) {
                        if let Ok(value) = stdout.trim_start().trim_end_matches('\n').parse::<u64>() {
                            self.gauge.set(value);
                        }
                    }
                }
            }
        } else {
            // FIXME: handle windows.
        }
    }
}

impl EncodeMetric for MemoryUsage {
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.gauge.encode(encoder)
    }

    fn metric_type(&self) -> MetricType {
        self.gauge.metric_type()
    }
}
