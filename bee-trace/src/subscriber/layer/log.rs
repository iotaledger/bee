use crate::Error;

use bee_common::logger::{LoggerConfig, LoggerOutputConfig};

use colored::{ColoredString, Colorize};
use parking_lot::{Mutex, MutexGuard};
use tracing::{field::Visit, metadata::LevelFilter, Event, Level, Metadata, Subscriber};
use tracing_log::{AsTrace, NormalizeEvent};
use tracing_subscriber::{
    filter::{self, Targets},
    fmt::MakeWriter,
    layer::{Context, Filter, Layer},
    registry::LookupSpan,
};

use std::{
    fs::{File, OpenOptions},
    io::{self, Stdout, StdoutLock},
};

pub enum LogOutput<'a> {
    Stdout(StdoutLock<'a>),
    File(MutexGuard<'a, File>),
}

impl<'a> io::Write for LogOutput<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(lock) => lock.write(buf),
            Self::File(lock) => lock.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout(lock) => lock.flush(),
            Self::File(lock) => lock.flush(),
        }
    }
}

pub struct LogTarget {
    filter: Targets,
    dest: LogDest,
}

pub enum LogDest {
    Stdout,
    File(Mutex<File>),
}

pub struct LogTargetMakeWriter {
    stdout: Stdout,
    target: LogTarget,
}

impl LogTargetMakeWriter {
    pub fn new(target: LogTarget) -> Self {
        Self {
            stdout: io::stdout(),
            target,
        }
    }

    pub fn enabled<S>(&self, meta: &Metadata<'_>, ctx: &Context<'_, S>) -> bool
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        Filter::enabled(&self.target.filter, meta, ctx)
    }
}

impl<'a> MakeWriter<'a> for LogTargetMakeWriter {
    type Writer = LogOutput<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        match &self.target.dest {
            LogDest::Stdout => LogOutput::Stdout(self.stdout.lock()),
            LogDest::File(file) => LogOutput::File(file.lock()),
        }
    }
}

pub struct LogLayer {
    make_writers: Vec<LogTargetMakeWriter>,
    fmt_events: LogFormatter,
}

impl<S> Layer<S> for LogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        if let Some(metadata) = event.normalized_metadata() {
            let mut buf = String::new();

            for make_writer in &self.make_writers {
                if make_writer.enabled(&metadata, &ctx) {
                    let mut writer = make_writer.make_writer();

                    if self.fmt_events.format_event(&mut buf, &writer, event).is_ok() {
                        let _ = io::Write::write(&mut writer, buf.as_bytes());
                    }

                    buf.clear();
                }
            }
        }
    }
}

impl LogLayer {
    pub fn new(config: LoggerConfig) -> Result<Self, Error> {
        let fmt_events = LogFormatter {
            color_enabled: config.color_enabled(),
            target_width: config.target_width(),
            level_width: config.level_width(),
        };

        let make_writers = config
            .outputs()
            .iter()
            .map(|output_config: &LoggerOutputConfig| {
                let level = output_config.level_filter().as_trace();

                let mut targets = if output_config.target_filters().is_empty() {
                    filter::Targets::default().with_default(level)
                } else {
                    let mut targets = filter::Targets::default().with_default(LevelFilter::OFF);

                    for filter in output_config.target_filters() {
                        targets = targets.with_target(filter.clone().to_lowercase(), level);
                    }

                    targets
                };

                for exclusion in output_config.target_exclusions() {
                    targets = targets.with_target(exclusion.clone().to_lowercase(), LevelFilter::OFF);
                }

                let dest = match output_config.name().as_str() {
                    "stdout" => LogDest::Stdout,
                    name => {
                        let file = OpenOptions::new().write(true).create(true).append(true).open(name)?;
                        LogDest::File(Mutex::new(file))
                    }
                };

                Ok(LogTargetMakeWriter::new(LogTarget { filter: targets, dest }))
            })
            .collect::<Result<_, _>>()
            .map_err(|err: io::Error| Error::LogLayer(err.into()))?;

        Ok(Self {
            make_writers,
            fmt_events,
        })
    }
}

trait ColorFormat {
    fn color(self, enabled: bool) -> ColoredString;
}

impl ColorFormat for Level {
    fn color(self, enabled: bool) -> ColoredString {
        let text = self.to_string();

        if !enabled {
            return text.as_str().into();
        }

        match self {
            Level::TRACE => text.bright_magenta(),
            Level::DEBUG => text.bright_blue(),
            Level::INFO => text.bright_green(),
            Level::WARN => text.bright_yellow(),
            Level::ERROR => text.bright_red(),
        }
    }
}

#[derive(Default)]
pub(crate) struct LogVisitor(String);

impl Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }
}

pub struct LogFormatter {
    color_enabled: bool,
    target_width: usize,
    level_width: usize,
}

impl LogFormatter {
    fn format_event<W>(&self, writer: &mut W, output: &LogOutput, event: &tracing::Event<'_>) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        if let Some(metadata) = event.normalized_metadata() {
            let level = *metadata.level();
            let target = metadata.target();

            let mut visitor = LogVisitor::default();
            event.record(&mut visitor);

            let time = bee_common::time::format(&bee_common::time::now_utc());

            let level = match output {
                LogOutput::File(_) => ColoredString::from(level.to_string().as_str()),
                LogOutput::Stdout(_) => level.color(self.color_enabled),
            };

            write!(
                writer,
                "{} {:target_width$} {:level_width$} {}",
                time,
                target,
                level,
                visitor.0,
                target_width = self.target_width,
                level_width = self.level_width,
            )?;

            writeln!(writer)?;
        }

        Ok(())
    }
}
