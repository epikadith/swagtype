//! Terminal raw-mode guard and environment helpers.
//!
//! [`RawModeGuard`] enables raw mode (and optionally the alternate screen) on
//! creation and restores the original state on [`Drop`], so even panics or
//! Ctrl-C won't leave the user's terminal broken.

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

/// RAII guard that enables crossterm raw mode on creation and disables it on
/// drop.  Optionally enters/leaves the alternate screen.
pub struct RawModeGuard {
    alternate_screen: bool,
}

impl RawModeGuard {
    /// Enable raw mode.  If `alternate_screen` is `true`, also switch to the
    /// alternate screen buffer (clears the visible terminal).
    pub fn new(alternate_screen: bool) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        if alternate_screen {
            execute!(io::stdout(), EnterAlternateScreen)?;
        }
        Ok(Self { alternate_screen })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.alternate_screen {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        }
        let _ = terminal::disable_raw_mode();
    }
}

/// Returns `true` when colored output should be used.
///
/// Respects the [`NO_COLOR`](https://no-color.org/) convention and checks
/// whether stdout is a real TTY (not piped).
pub fn color_enabled() -> bool {
    use crossterm::tty::IsTty;

    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    io::stdout().is_tty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_enabled_respects_no_color_env() {
        // In the test environment NO_COLOR may or may not be set, but setting
        // it should always disable color.
        unsafe { std::env::set_var("NO_COLOR", "1") };
        assert!(!color_enabled());
        unsafe { std::env::remove_var("NO_COLOR") };
    }

    // NOTE: RawModeGuard is intentionally not tested here because enabling raw
    // mode in a CI/test runner that may not have a real TTY is unreliable and
    // can break output.  The guard is kept deliberately thin (two crossterm
    // calls) so that manual verification suffices.
}
