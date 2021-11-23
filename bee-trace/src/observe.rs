use tracing::trace_span;
use tracing_futures::{Instrument, Instrumented};

pub trait Observe: Sized {
    #[track_caller]
    fn observe(self, name: &str) -> Instrumented<Self>;
}

impl<T: Instrument> Observe for T {
    #[track_caller]
    fn observe(self, name: &str) -> Instrumented<Self> {
        let location = std::panic::Location::caller();

        let span = trace_span!(
            target: "bee::observe",
            "observed",
            observed.name = name,
            loc.file = location.file(),
            loc.line = location.line(),
            loc.col = location.column(),
        );

        self.instrument(span)
    }
}
