// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing some predefined useful metrics.

pub use prometheus_client::metrics::{counter, gauge};
use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder},
    metrics::{gauge::Gauge, MetricType},
};
use tokio::process::Command;

/// Type used to track and update metrics about an specific OS process.
#[derive(Clone)]
pub struct ProcessMetrics {
    mem: Gauge,
    cpu: Gauge<f64>,
    pid: String,
}

impl ProcessMetrics {
    /// Creates a new set of metrics for the desired PID.
    pub fn new(pid: u32) -> Self {
        Self {
            mem: Gauge::default(),
            cpu: Gauge::default(),
            pid: pid.to_string(),
        }
    }

    /// Obtains the metrics tracked by this value.
    pub fn metrics(&self) -> (MemoryUsage, CpuUsage) {
        (MemoryUsage(self.mem.clone()), CpuUsage(self.cpu.clone()))
    }

    /// Updates the metrics values.
    ///
    /// The value is not updated if the command used to retrieve the new value:
    ///  - cannot be spawned,
    ///  - returns an unsuccessful exit code,
    ///  - its output cannot be parsed.
    pub async fn update(&self) {
        if cfg!(unix) {
            if let Ok(output) = Command::new("ps")
                .arg("-o")
                .arg("%cpu= rss=")
                .arg("--pid")
                .arg(&self.pid)
                .output()
                .await
            {
                if output.status.success() {
                    if let Ok(stdout) = String::from_utf8(output.stdout) {
                        if let Some((cpu_str, rss_str)) = stdout.trim_start().trim_end_matches('\n').split_once(' ') {
                            if let Ok(value) = cpu_str.parse::<f64>() {
                                self.cpu.set(value);
                            }

                            if let Ok(value) = rss_str.parse::<u64>() {
                                self.mem.set(value);
                            }
                        }
                    }
                }
            }
        } else {
            // FIXME: handle windows.
        }
    }
}

/// Metric that tracks the memory used by a process in Kilobytes.
///
/// On unix-like platforms this metric takes the RSS (resident set size) reported by the `ps`
/// command.
///
/// This metric can be created from a [`ProcessMetrics`] value.
pub struct MemoryUsage(Gauge);

impl EncodeMetric for MemoryUsage {
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.0.encode(encoder)
    }

    fn metric_type(&self) -> MetricType {
        self.0.metric_type()
    }
}

/// Metric that tracks the CPU usage as a percentage.
///
/// On unix-like platforms this metric takes the %CPU (cpu utilization of the process) reported by
/// the `ps` command.
///
/// This metric can be created from a [`ProcessMetrics`] value.
pub struct CpuUsage(Gauge<f64>);

impl EncodeMetric for CpuUsage {
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.0.encode(encoder)
    }

    fn metric_type(&self) -> MetricType {
        self.0.metric_type()
    }
}
