//! Display

#![deny(bad_style, missing_docs, unsafe_code)]
#![cfg_attr(release, deny(warnings))]

mod constants;

use crate::constants::{HEADER_SECTION_HEIGHT, HEART, HEART_BEAT, HEART_BEAT_POSITION};

use bee_core::constants::{NAME, VERSION};

use crossterm::{style, ClearType, Color, Colored, Terminal, TerminalCursor};

use std::thread;
use std::time::Duration;

/// A terminal display.
pub struct Display {
    cursor: TerminalCursor,
    terminal: Terminal,
    width: u16,
}

impl Display {
    /// Creates a new display.
    ///
    /// # Example
    /// ```
    /// use bee_display::Display;
    /// let display = Display::new();
    /// ```
    pub fn new() -> Self {
        let terminal = crossterm::terminal();
        let (width, _) = terminal.terminal_size();

        Self { cursor: crossterm::cursor(), terminal, width }
    }

    /// Clears the display.
    pub fn clear(&self) {
        self.terminal.clear(ClearType::All).expect("couldn't clear screen");

        self.cursor.hide().unwrap();
    }

    /// Prints text in the specified color.
    pub fn header(&mut self) {
        let num = usize::from(self.width) - 2;
        let s = format!(" {} v{}", NAME, VERSION);

        println!("╔{:═<1$}╗", "", num);
        print!("║{}", Colored::Fg(Color::Yellow));
        print!("{: <1$}", s, num);
        println!("{}║", Colored::Fg(Color::Reset));
        for _ in 0..HEADER_SECTION_HEIGHT {
            println!("║{: <1$}║", "", num);
        }
        println!("╚{:═<1$}╝", "", num);

        self.save();
    }

    /// Displays a heartbeat.
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

    /// Closes the display.
    pub fn close(&self) {
        self.cursor.show().unwrap();
    }

    fn save(&self) {
        self.cursor.save_position().unwrap();
    }
}
