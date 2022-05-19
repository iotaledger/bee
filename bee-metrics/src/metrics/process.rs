// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Useful metrics when working with processes.

use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder},
    metrics::{gauge::Gauge, MetricType},
};
use tokio::process::Command;

/// Type used to track and update metrics about a specific OS process.
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

    /// Updates the metrics values. Returns an error if it was not possible to fetch the values
    /// from the respective OS command:
    /// - `ps` for unix-like platforms.
    pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug)]
        enum UpdateError {
            UnsuccessfulTermination(Option<i32>),
            UnsupportedPlatform(&'static str),
            CannotParseOutput(String),
        }

        impl std::error::Error for UpdateError {}

        impl std::fmt::Display for UpdateError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::UnsuccessfulTermination(code) => {
                        write!(f, "unsuccessful termination with exit code: {code:?}")
                    }
                    Self::UnsupportedPlatform(platform) => write!(f, "unsupported platform: {platform:?}"),
                    Self::CannotParseOutput(output) => write!(f, "cannot parse output: {output:?}"),
                }
            }
        }

        if cfg!(unix) {
            let output = Command::new("ps")
                .arg("-o")
                .arg("%cpu= rss=")
                .arg("--pid")
                .arg(&self.pid)
                .output()
                .await?;

            if !output.status.success() {
                return Err(Box::new(UpdateError::UnsuccessfulTermination(output.status.code())));
            }

            let stdout = String::from_utf8(output.stdout)?;
            let trimmed_stdout = stdout.trim_start().trim_end_matches('\n');

            let (cpu_str, rss_str) = trimmed_stdout
                .split_once(' ')
                .ok_or_else(|| UpdateError::CannotParseOutput(trimmed_stdout.to_string()))?;

            let cpu_value = cpu_str.parse::<f64>()?;
            self.cpu.set(cpu_value);

            let mem_value = rss_str.parse::<u64>()?;
            self.mem.set(mem_value);

            Ok(())
        } else {
            // FIXME: handle windows.
            Err(Box::new(UpdateError::UnsupportedPlatform("")))
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
