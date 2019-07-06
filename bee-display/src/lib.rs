//! Display

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(release, deny(warnings))]

mod constants;

use crate::constants::{HEART, HEART_BEAT, HEART_BEAT_POSITION, LOGO};

use bee_core::constants::{NAME, VERSION};

use crossterm::{style, ClearType, Color, Colored, Terminal, TerminalCursor};

use std::thread;
use std::time::Duration;

/// A terminal display.
///
/// # Example
/// ```
/// use bee_display::Display;
/// let display = Display::new();
/// ```
pub struct Display {
    cursor: TerminalCursor,
    terminal: Terminal,
    width: u16,
}

impl Display {
    /// Create a new display.
    pub fn new() -> Self {
        let terminal = crossterm::terminal();
        let (width, _) = terminal.terminal_size();

        Self { cursor: crossterm::cursor(), terminal, width }
    }

    /// Clear the terminal.
    pub fn clear(&self) {
        self.terminal.clear(ClearType::All).expect("couldn't clear screen");

        self.cursor.hide().unwrap();
    }

    /// Print a header section with relevant node live data.
    pub fn header(&mut self) {
        let width = usize::from(self.width) - 1;
        let name_version = format!(" {} v{}", NAME, VERSION);
        let name_version_left = width as u16 / 2 - name_version.len() as u16 / 2;
        let logo_lines = LOGO.split("\n").collect::<Vec<_>>();

        print_topline(width);
        for line in logo_lines.iter() {
            print_colored_line(line, Color::White, width);
        }
        print_text_at_pos(
            &name_version,
            name_version_left,
            7,
            Color::Yellow,
            &self.cursor,
        );
        print_botline(width);

        self.save();
    }

    /// Display a heartbeat animation.
    pub fn heartbeat(&self) {
        let mut tick = true;
        let cursor = crossterm::cursor();
        let terminal = crossterm::terminal();

        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(HEART_BEAT));
            let heart = if tick {
                tick = false;
                HEART
            } else {
                tick = true;
                " "
            };

            cursor.save_position().unwrap();
            cursor
                .goto(HEART_BEAT_POSITION.0, HEART_BEAT_POSITION.1)
                .expect("error moving cursor");

            terminal
                .write(style(heart).with(Color::Red))
                .expect("error writing to terminal");
            cursor.reset_position().unwrap();
        });
    }

    /// Shutdown the display.
    pub fn close(&self) {
        self.cursor.show().unwrap();
    }

    fn save(&self) {
        self.cursor.save_position().unwrap();
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
fn print_colored_line(text: &str, color: Color, width: usize) {
    print!("║{}", Colored::Fg(color));
    print!("{: <1$}", text, width);
    println!("{}║", Colored::Fg(Color::Reset));
}

#[inline]
fn print_text_at_pos(text: &str, x: u16, y: u16, color: Color, cursor: &TerminalCursor) {
    let (old_x, old_y) = cursor.pos();
    cursor.goto(x, y).expect("couldn't move cursor");

    print!("{}{}{}", Colored::Fg(color), text, Colored::Fg(Color::Reset));

    cursor.goto(old_x, old_y).expect("couldn't move cursor");
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
