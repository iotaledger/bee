use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder},
    metrics::{gauge::Gauge, MetricType},
};
use tokio::process::Command;

#[derive(Clone)]
pub struct ResidentSetSize {
    gauge: Gauge,
    pid: String,
}

impl ResidentSetSize {
    pub fn new(pid: u32) -> Self {
        Self {
            gauge: Gauge::default(),
            pid: pid.to_string(),
        }
    }

    pub async fn update(&self) {
        let output = Command::new("ps")
            .arg("-o")
            .arg("rss=")
            .arg("--pid")
            .arg(&self.pid)
            .output()
            .await
            .unwrap();

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let value: u64 = stdout.trim_start().trim_end_matches('\n').parse().unwrap();
            println!("Setting gauge metric to {}", value);
            self.gauge.set(value);
        }
    }
}

impl EncodeMetric for ResidentSetSize {
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.gauge.encode(encoder)
    }

    fn metric_type(&self) -> MetricType {
        self.gauge.metric_type()
    }
}
