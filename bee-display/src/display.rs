//! Display module.

use crate::constants::{LOGO, UPDATE_INTERVAL};

use bee_core::constants::{NAME, VERSION};
use bee_core::messaging::{Effect, Entity};

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::{ClearType, Color, Colored, Terminal, TerminalCursor};
use futures::prelude::*;
use tokio_timer::Interval;

/// A terminal display.
///
/// # Example
/// ```
/// use bee_display::Display;
///
/// let display = Display::new();
/// ```
#[derive(Clone)]
pub struct Display {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    /// A reference to the terminal itself.
    terminal: Terminal,

    /// A reference to the terminal cursor.
    cursor: TerminalCursor,

    /// the current terminal width.
    terminal_width: u16,

    /// The current terminal height.
    terminal_height: u16,

    /// The update interval.
    update_interval: Interval,

    /// The current network address.
    address: Option<Arc<String>>,
}

impl Display {
    /// Creates a new display.
    pub fn new() -> Self {
        let terminal = crossterm::terminal();
        let (w, h) = terminal.terminal_size();

        let inner = Inner {
            terminal: crossterm::terminal(),
            cursor: crossterm::cursor(),
            terminal_width: w,
            terminal_height: h,
            update_interval: Interval::new(
                Instant::now(),
                Duration::from_millis(UPDATE_INTERVAL),
            ),
            address: None,
        };

        Self { inner: Arc::new(Mutex::new(inner)) }
    }

    /// Clears the terminal.
    ///
    /// This method should be called early.
    pub fn clear(&self) {
        let inner = self.inner.lock().unwrap();

        inner.terminal.clear(ClearType::All).expect("couldn't clear screen");
        inner.cursor.hide().unwrap();
    }

    /// Prints a header section with relevant node live data.
    pub fn header(&mut self) {
        let inner = self.inner.lock().unwrap();

        let width = usize::from(inner.terminal_width) - 1;
        let name_version = format!(" {} v{}", NAME, VERSION);
        let name_version_left = width as u16 / 2 - name_version.len() as u16 / 2;
        let logo_lines = LOGO.split("\n").collect::<Vec<_>>();

        print_topline(width);
        for line in logo_lines.iter() {
            color_println(line, Color::White, width);
        }
        color_print_at_pos(
            &name_version,
            name_version_left,
            7,
            Color::Yellow,
            &inner.cursor,
        );
        print_botline(width);

        inner.cursor.save_position().unwrap();
    }

    /// Shutdown the display.
    pub fn close(&self) {
        let inner = self.inner.lock().unwrap();
        inner.cursor.show().unwrap();
    }

    fn save(&self) {
        let inner = self.inner.lock().unwrap();
        inner.cursor.save_position().unwrap();
    }
}

#[inline]
fn print_topline(width: usize) {
    println!("╔{:═<1$}╗", "", width);
}

#[inline]
fn print_botline(width: usize) {
    println!("╚{:═<1$}╝", "", width);
}

#[inline]
fn color_println(text: &str, color: Color, width: usize) {
    print!("║{}", Colored::Fg(color));
    print!("{: <1$}", text, width);
    println!("{}║", Colored::Fg(Color::Reset));
}

fn color_print_at_pos(text: &str, x: u16, y: u16, color: Color, cursor: &TerminalCursor) {
    let (old_x, old_y) = cursor.pos();
    cursor.goto(x, y).expect("couldn't move cursor");

    print!("{}{}{}", Colored::Fg(color), text, Colored::Fg(Color::Reset));

    cursor.goto(old_x, old_y).expect("couldn't move cursor");
}

// The display represents an [`Entity`] in the EEE model, that has joined one or many
// environments to receive live updated node data.
impl Entity for Display {
    /// Processes the effect depending on the environment in came from
    fn process_effect(&mut self, effect: Effect, environment: &str) -> Effect {
        let mut inner = self.inner.lock().unwrap();
        match environment {
            "NETWORK_STATS" => match effect {
                Effect::String(s) => {
                    inner.address.replace(s);
                    Effect::Empty
                }
                _ => Effect::Empty,
            },
            _ => Effect::Empty,
        }
    }
}

// The display constantly updates itself. Each update produces an empty result.
impl Stream for Display {
    type Item = ();
    type Error = String;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut inner = self.inner.lock().unwrap();
        //
        match inner.update_interval.poll() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(Some(_))) => (),
            Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
            Err(_) => return Err(String::from("error polling interval")),
        }
        if let Some(address) = &inner.address {
            color_print_at_pos(address, 20, 10, Color::Blue, &inner.cursor);
        }
        Ok(Async::Ready(Some(())))
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_display() {
        let mut display = Display::new();

        display.header();
        display.heartbeat();
        display.close();
    }
}
