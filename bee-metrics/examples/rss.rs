use bee_metrics::{metrics::ResidentSetSize, registry::Registry, serve_metrics};
use tokio::time::Duration;

async fn update_rss(rss: ResidentSetSize) {
    let mut vec = Vec::new();

    loop {
        // Add 10KiB of data every second and update the RSS metric.
        vec.extend_from_slice(&[0u8; 10240]);

        rss.update().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    let registry = Registry::default();

    let rss = ResidentSetSize::new(std::process::id());
    registry.register("RSS", "Resident set size", rss.clone());

    let handle = { tokio::spawn(async move { update_rss(rss).await }) };

    let serve_fut = serve_metrics("0.0.0.0:3030".parse().unwrap(), registry);

    let (serve_res, handle_res) = tokio::join!(serve_fut, handle);

    serve_res.unwrap();
    handle_res.unwrap();
}
