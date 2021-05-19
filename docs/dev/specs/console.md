# `console` feature
The `bee-node` crate now includes a `console` feature, which can help with debugging and providing diagnostics for the multiple async tasks that Bee is executing at any given time. Building with this feature enabled and running a separate executable will provide realtime diagnostics in your terminal.

**Note**: `tokio-console` is highly unstable, so this feature may be very changeable.



# How do I use it?
To build/run with the instrumentation, make sure the `console` feature is enabled:
```
> cargo run --release --features console
```
Additionally, your `RUSTFLAGS` environment variable must contain the following, in order for `tokio` to be built with the `tracing` feature enabled:
```
--cfg tokio_unstable
```
Once the node is up and running, clone a local copy of the [`tokio-console`](https://github.com/tokio-rs/console) project, and run the `console` crate from its directory:
```
.../tokio-console/console/ > cargo run
```

# How does it work?
This change adds instrumentation to Bee tasks in the form of `span`s, provided by the `tracing` crate. These are helpful for producing diagnostic information for a period of time, rather than a one-off event. They live for the scope of the task that they instrument:
```rust
let span = tracing::info_span!(
    target: "tokio::task", 
    "task", 
    file = caller.file(), 
    line = caller.line(),
);

tokio::spawn(future.instrument(span))
```
In order to collect information and log these spans, `tracing_subscriber` can be used and customised to handle span events (on entry, on exit, etc). The `console_subscriber` crate in the `tokio-console` project provides a way to customise a `tracing_subscriber` to broadcast structured gRPC messages on these events. The following snippet creates a subscriber that does this, constraining the captured spans to `INFO` level:
```rust
let (layer, server) = console_subscriber::TasksLayer::new();

let filter = tracing_subscriber::EnvFilter::from_default_env()
    .add_directive(tracing::Level::INFO.into())
    .add_directive("tokio=info".parse().unwrap());

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(filter)
    .with(layer)
    .init();
```
The `console` project then interprets the messages to display them in real-time in a console UI.
