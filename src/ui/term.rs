//! Terminal lifecycle: raw mode, alternate screen, panic-safe restore.

use std::io::{Stdout, stdout};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

pub type Term = Terminal<CrosstermBackend<Stdout>>;

pub struct TermGuard {
    pub mouse: bool,
}

impl TermGuard {
    /// Toggle mouse capture at runtime. Capture gives wheel scrolling but
    /// steals native text selection; readers need both at different moments.
    pub fn set_mouse(&mut self, on: bool) {
        if on == self.mouse {
            return;
        }
        let mut out = stdout();
        if on {
            let _ = execute!(out, EnableMouseCapture);
        } else {
            let _ = execute!(out, DisableMouseCapture);
        }
        self.mouse = on;
    }
}

impl TermGuard {
    /// Enter TUI mode. The panic hook restores the terminal before the panic
    /// message prints — otherwise a crash leaves the shell in raw mode.
    pub fn enter(mouse: bool) -> std::io::Result<(Self, Term)> {
        enable_raw_mode()?;
        // From here on, any failure must undo raw mode before returning, or
        // the error message prints into a broken shell.
        let init = || -> std::io::Result<Term> {
            let mut out = stdout();
            execute!(out, EnterAlternateScreen)?;
            if mouse {
                let _ = execute!(out, EnableMouseCapture);
            }
            Terminal::new(CrosstermBackend::new(stdout()))
        };
        let terminal = match init() {
            Ok(t) => t,
            Err(e) => {
                restore();
                return Err(e);
            }
        };

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            hook(info);
        }));

        Ok((Self { mouse }, terminal))
    }

    /// Temporarily hand the terminal to another program ($EDITOR).
    pub fn suspend(&self) {
        restore();
    }

    /// Re-enter TUI mode after `suspend`.
    pub fn resume(&self) -> std::io::Result<()> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;
        if self.mouse {
            let _ = execute!(out, EnableMouseCapture);
        }
        Ok(())
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        restore();
    }
}

/// Unconditionally undo everything `enter` may have set. Disabling mouse
/// capture when it was never enabled is harmless, and being unconditional
/// means a stale captured flag can never leave the shell broken.
fn restore() {
    let mut out = stdout();
    let _ = execute!(out, DisableMouseCapture);
    let _ = execute!(out, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
