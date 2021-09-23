# `tokio-console` feature
The `bee-node` crate includes a `tokio-console` feature, which can help with debugging and providing diagnostics for the multiple async tasks that the node is executing at any given time. Building with this feature enabled and running a separate executable will provide realtime diagnostics in your terminal.

# What is it?
The `tokio-console` feature provides instrumentation on all tasks spawned in the `BeeNode::spawn` method, and aggregates information provided by this instrumentation in a "subscriber", that allows logging and further processing. This is extremely useful for Bee particularly, since it makes use of a task-heavy architecture that can be difficult to debug and diagnose: having realtime task diagnostics should help to alleviate this.

`tokio-console` provides the following statistics for each worker task:

 - Total execution time (time since task spawn)
 - Total "idle" time (time spent `await`ing, blocked, etc)
 - Total "busy" time ("total" - "idle")
 - Customisable fields, represented as a string

Here, we add the file and line of the calling function to the task fields, so that we can determine which particular worker has spawned a task. As mentioned above, additional information can be added with the `span` macro from the `tracing` crate:

```rust
// Creates a span with the name "task" and fields "file", "line", "content", and "extra".
let _span = span!(
    "task",
    file = file!(),
    line = line!(),
    content = "example field",
    extra = "extra field"
);
```

`tokio-console` is interactive, and continually updates as your process runs:

![Capture](https://user-images.githubusercontent.com/22496597/118669528-bb279e00-b7ed-11eb-88a6-e8f535643fdd.PNG)

**Note**: `tokio-console` is highly unstable, so this feature may be very changeable.

# How do I use it?
To build/run with the instrumentation, make sure the `tokio-console` feature is enabled, and your `RUSTFLAGS` environment variable contains the `--cfg tokio_unstable` options:
```
$ RUSTFLAGS="--cfg tokio_unstable" cargo run --release --features tokio-console
```
Once the node is up and running, clone a local copy of the [`tokio-console`](https://github.com/tokio-rs/console) project, and run the `console` crate from its directory:
```console
user@tokio-console/console:~$ cargo run
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
