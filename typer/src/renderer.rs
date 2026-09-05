//! Live character-by-character rendering for the typing test.

use crate::diff::{compute_diff, CharStatus};
use core::terminal::color_enabled;
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{size, Clear, ClearType},
};
use std::io::{self, Write};

pub struct Renderer {
    use_color: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            use_color: color_enabled(),
        }
    }

    /// Clears the screen and prepares for drawing.
    pub fn clear(&self, out: &mut impl Write) -> io::Result<()> {
        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;
        Ok(())
    }

    /// Renders the current state of the typing test. Returns the final Y position.
    pub fn render(
        &self,
        out: &mut impl Write,
        expected: &str,
        typed: &str,
        wpm: f64,
        accuracy: f64,
        show_footer: bool,
    ) -> io::Result<u16> {
        let diff = compute_diff(expected, typed);

        // Header: WPM and Accuracy
        queue!(
            out,
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            Print(format!(
                "WPM: {:>5.1}   Accuracy: {:>5.1}%",
                wpm, accuracy
            ))
        )?;

        let (cols, _) = size().unwrap_or((80, 24));
        let width = cols.saturating_sub(1);
        let mut x = 0;
        let mut y = 2;

        for word_chunk in diff.split_inclusive(|d| d.expected == ' ') {
            let word_len = word_chunk.len() as u16;
            if x > 0 && x + word_len > width {
                x = 0;
                y += 1;
            }

            for d in word_chunk {
                queue!(out, MoveTo(x, y))?;
                x += 1;
                if x >= width {
                    x = 0;
                    y += 1;
                }

                match d.status {
                    CharStatus::Correct => {
                        if self.use_color {
                            queue!(out, SetForegroundColor(Color::Green), Print(d.expected), ResetColor)?;
                        } else {
                            queue!(out, Print(d.expected))?;
                        }
                    }
                    CharStatus::Incorrect => {
                        let c = d.actual.unwrap_or(d.expected);
                        let display_c = if c == ' ' { '█' } else { c };
                        if self.use_color {
                            queue!(out, SetForegroundColor(Color::Red), Print(display_c), ResetColor)?;
                        } else {
                            queue!(out, Print(display_c))?;
                        }
                    }
                    CharStatus::Pending => {
                        if self.use_color {
                            queue!(out, SetForegroundColor(Color::DarkGrey), Print(d.expected), ResetColor)?;
                        } else {
                            queue!(out, Print(d.expected))?;
                        }
                    }
                }
            }
        }

        if show_footer {
            y += 2;
            queue!(
                out,
                MoveTo(0, y),
                SetForegroundColor(Color::DarkGrey),
                Print("Ctrl+R to restart | Esc to quit"),
                ResetColor
            )?;
        }

        out.flush()?;
        Ok(y)
    }
}
