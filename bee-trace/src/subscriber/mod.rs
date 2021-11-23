pub mod layer;

use tracing_log::LogTracer;

pub fn collect_logs() {
    LogTracer::init().expect("unable to set the logger");
}
