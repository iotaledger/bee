// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tracing::trace_span;
use tracing_futures::{Instrument, Instrumented};

/// The field name of the [`Span`](tracing::Span) location file.
pub(crate) const FILE_FIELD_NAME: &str = "loc.file";

/// The field name of the [`Span`](tracing::Span) location line.
pub(crate) const LINE_FIELD_NAME: &str = "loc.line";

/// The target of the wrapping [`Span`](tracing::Span).
pub const SPAN_TARGET: &str = "bee::observe";

/// The name of the wrapping [`Span`](tracing::Span).
pub const SPAN_NAME: &str = "observed";

/// Instruments a future with a `tracing` span.
///
/// This span is given the `bee::observe` target, so that it can be more easily filtered
/// in any subscribers or subscriber layers. It also records the future's calling location
/// in its fields.
pub trait Observe: Sized {
    #[track_caller]
    /// Instruments `Self` with a `tracing` span, returning [`Instrumented<Self>`].
    fn observe(self, name: &str) -> Instrumented<Self>;
}

impl<T: Instrument> Observe for T {
    #[track_caller]
    fn observe(self, name: &str) -> Instrumented<Self> {
        let location = std::panic::Location::caller();

        let span = trace_span!(
            target: SPAN_TARGET,
            SPAN_NAME,
            observed.name = name,
            loc.file = location.file(),
            loc.line = location.line(),
            loc.col = location.column(),
        );

        self.instrument(span)
    }
}
