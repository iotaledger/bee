// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use env_logger::fmt::Color;
use log::{debug, error, info, log_enabled, trace, warn};

use std::io::Write;

pub fn init(level_filter: log::LevelFilter) {
    pretty_env_logger::formatted_timed_builder()
        .format_indent(None)
        .format(|f, record| {
            let ts = f.timestamp();

            let col = match record.level() {
                log::Level::Trace => Color::Magenta,
                log::Level::Debug => Color::Blue,
                log::Level::Info => Color::Green,
                log::Level::Warn => Color::Yellow,
                log::Level::Error => Color::Red,
            };

            let mut level_style = f.style();
            level_style.set_color(col).set_bold(true);

            writeln!(f, "[{} {:>7}] {}", ts, level_style.value(record.level()), record.args())
        })
        //.format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .format_timestamp_secs()
        .filter_level(level_filter)
        .init();
}

pub fn trace(message: &str, context: &str) {
    if log_enabled!(log::Level::Trace) {
        trace!("{} {}", context, message);
    }
}

pub fn debug(message: &str, context: &str) {
    if log_enabled!(log::Level::Debug) {
        debug!("{} {}", context, message);
    }
}

pub fn info(message: &str, context: &str) {
    if log_enabled!(log::Level::Info) {
        info!("{} {}", context, message);
    }
}

pub fn warn(message: &str, context: &str) {
    if log_enabled!(log::Level::Warn) {
        warn!("{} {}", context, message);
    }
}

pub fn error(message: &str, context: &str) {
    if log_enabled!(log::Level::Error) {
        error!("{} {}", context, message);
    }
}
