# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- ## Unreleased - YYYY-MM-DD

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security -->

## 0.1.0 - 2022-04-12

### Added

- `Registry` type to register metrics;
- `ProcessMetrics`, `MemoryUsage` and `CpuUsage` types to track OS process related metrics;
- `serve_metrics` function to serve the metrics to Prometheus;
- Re-exported `prometheus_client` types to write custom metrics;
