use crate::{util::Flamegrapher, Error};

use lazy_static::lazy_static;
use parking_lot::RwLock;
use tracing::{callsite, field::Visit, span, subscriber, Metadata, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    fmt::{Display, Write as _},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    time::{Duration, Instant},
};

lazy_static! {
    static ref START: Instant = Instant::now();
}

thread_local! {
    static LAST_EVENT: Cell<Instant> = Cell::new(*START);
}

pub struct FlamegraphLayer {
    callsites: RwLock<HashSet<callsite::Identifier>>,
    span_locations: RwLock<HashMap<span::Id, Option<SpanLocation>>>,
    out_file: RwLock<BufWriter<File>>,
}

impl FlamegraphLayer {
    pub(crate) fn new<P: AsRef<Path>>(stack_filename: P) -> Result<(Self, Flamegrapher), Error> {
        let _ = *START;

        let stack_filename = stack_filename.as_ref().with_extension("folded");
        let out_file = File::create(stack_filename.clone()).map_err(|err| Error::FlamegraphLayer(err.into()))?;

        let flamegrapher = Flamegrapher::new()
            .with_stack_file(stack_filename)
            .expect("stack file does not exist");

        let layer = Self {
            callsites: RwLock::new(HashSet::new()),
            span_locations: RwLock::new(HashMap::new()),
            out_file: RwLock::new(BufWriter::new(out_file)),
        };

        Ok((layer, flamegrapher))
    }

    fn is_tracked_callsite(&self, callsite: &callsite::Identifier) -> bool {
        self.callsites.read().contains(callsite)
    }

    fn is_tracked<S>(&self, id: &span::Id, ctx: &Context<'_, S>) -> bool
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        ctx.span(id)
            .map(|span| self.is_tracked_callsite(&span.metadata().callsite()))
            .unwrap_or(false)
    }

    fn stack_string_on_enter<S>(&self, id: &span::Id, ctx: &Context<'_, S>) -> String
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        self.stack_string(id, ctx, true)
    }

    fn stack_string_on_exit<S>(&self, id: &span::Id, ctx: &Context<'_, S>) -> String
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        self.stack_string(id, ctx, false)
    }

    fn stack_string<S>(&self, id: &span::Id, ctx: &Context<'_, S>, skip_current_span: bool) -> String
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let delta = self.time_since_last_event();
        let mut stack_str = "all-spans".to_string();

        let mut leaf_span = Some(ctx.span(id).expect("span is not in registry"));
        if skip_current_span {
            leaf_span = leaf_span.and_then(|span| span.parent());
        }

        if let Some(span) = leaf_span {
            for span in span.scope().from_root() {
                let location = match self
                    .span_locations
                    .read()
                    .get(&span.id())
                    .and_then(|location| location.as_ref())
                {
                    Some(location) => location.to_string(),
                    None => "unknown".to_string(),
                };

                write!(stack_str, "; {}", location).expect("writing to String should never fail");
            }
        }

        let _ = write!(stack_str, " {}", delta.as_micros());
        stack_str
    }

    fn time_since_last_event(&self) -> Duration {
        let now = Instant::now();

        let last_event = LAST_EVENT.with(|time| {
            let last_event = time.get();
            time.set(now);
            last_event
        });

        now - last_event
    }
}

impl<S> Layer<S> for FlamegraphLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> subscriber::Interest {
        match (metadata.name(), metadata.target()) {
            ("runtime.spawn", _) | ("task", "tokio::task") | (_, "bee::observe") => {
                self.callsites.write().insert(metadata.callsite());
            }
            (_, _) => {}
        }

        subscriber::Interest::always()
    }

    fn new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, _ctx: Context<'_, S>) {
        if !self.is_tracked_callsite(&attrs.metadata().callsite()) {
            return;
        }

        let location = SpanLocation::from_attributes(attrs);
        self.span_locations.write().insert(id.clone(), location);
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        if !self.is_tracked(id, &ctx) {
            return;
        }

        let stack_str = self.stack_string_on_enter(id, &ctx);
        let _ = writeln!(*self.out_file.write(), "{}", stack_str);
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        if !self.is_tracked(id, &ctx) {
            return;
        }

        let stack_str = self.stack_string_on_exit(id, &ctx);
        let _ = writeln!(*self.out_file.write(), "{}", stack_str);
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        if !self.is_tracked(&id, &ctx) {
            return;
        }

        let _ = self.span_locations.write().remove(&id);
    }
}

pub(crate) struct SpanLocation {
    file: String,
    line: u32,
}

impl SpanLocation {
    pub(crate) fn from_attributes(attrs: &span::Attributes) -> Option<Self> {
        let mut visitor = LocationVisitor::new();
        attrs.record(&mut visitor);
        visitor.try_into().ok()
    }
}

impl Display for SpanLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

impl TryFrom<LocationVisitor> for SpanLocation {
    type Error = ();

    fn try_from(visitor: LocationVisitor) -> Result<Self, Self::Error> {
        let LocationVisitor { file, line } = visitor;

        file.zip(line).map(|(file, line)| SpanLocation { file, line }).ok_or(())
    }
}

#[derive(Default)]
pub(crate) struct LocationVisitor {
    file: Option<String>,
    line: Option<u32>,
}

impl LocationVisitor {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Visit for LocationVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {}

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "loc.line" => self.line = Some(value as u32),
            _ => {}
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "loc.file" {
            self.file = Some(value.to_string());
        }
    }
}
