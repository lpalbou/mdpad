//! Terminal lifecycle: raw mode, alternate screen, panic-safe restore,
//! and tty-health-aware input waiting.

use std::io::{Stdout, stdout};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

/// Outcome of waiting for terminal input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputWait {
    /// An event is ready for `crossterm::event::read`.
    Ready,
    /// Nothing arrived within the timeout (idle tick).
    Timeout,
    /// The terminal is gone (pty/tty hangup): quit, do not read.
    Gone,
}

/// Wait up to `timeout_ms` for terminal input without entering crossterm.
///
/// crossterm 0.28's poll loop treats EOF on the tty (`read() == 0`, the
/// state after a terminal emulator or pty master goes away) as neither
/// data nor error and re-reads forever: `event::poll` never returns and
/// the process spins at full tilt, invisible to the caller. Waiting with
/// plain `poll(2)` first sidesteps that: a dead tty reports POLLHUP /
/// POLLERR / POLLNVAL, which maps to `Gone` before crossterm ever sees
/// the EOF. Fd 0 is always the tty here — main.rs re-attaches /dev/tty
/// over stdin when the document was piped in.
#[cfg(unix)]
pub fn wait_input(timeout_ms: i32) -> std::io::Result<InputWait> {
    let mut pfd = libc::pollfd {
        fd: 0,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: valid pointer to one pollfd; poll does not retain it.
    let n = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
    if n < 0 {
        let err = std::io::Error::last_os_error();
        // A signal (e.g. SIGWINCH on resize) interrupting the wait is an
        // idle tick: the loop re-checks size and quit flags on every pass.
        if err.kind() == std::io::ErrorKind::Interrupted {
            return Ok(InputWait::Timeout);
        }
        return Err(err);
    }
    if n == 0 {
        return Ok(InputWait::Timeout);
    }
    if pfd.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
        return Ok(InputWait::Gone);
    }
    Ok(InputWait::Ready)
}

/// Windows: the console API has no EOF-spin failure mode; defer to
/// crossterm's own poll.
#[cfg(not(unix))]
pub fn wait_input(timeout_ms: i32) -> std::io::Result<InputWait> {
    let ready =
        ratatui::crossterm::event::poll(std::time::Duration::from_millis(timeout_ms as u64))?;
    Ok(if ready {
        InputWait::Ready
    } else {
        InputWait::Timeout
    })
}

/// True once a termination signal (SIGTERM/SIGHUP/SIGINT/SIGQUIT) arrived.
///
/// Raw mode + alternate screen survive a default-action kill (drop
/// handlers don't run on signals), leaving the user's shell broken. A
/// flag checked every loop tick turns `kill` into a normal quit with full
/// terminal restore — the same courtesy less and vim extend.
#[cfg(unix)]
pub fn quit_requested() -> bool {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};

    static FLAG: OnceLock<std::sync::Arc<AtomicBool>> = OnceLock::new();
    let flag = FLAG.get_or_init(|| {
        let flag = std::sync::Arc::new(AtomicBool::new(false));
        for sig in [
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGHUP,
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGQUIT,
        ] {
            // Registration can only fail for invalid/forbidden signals;
            // losing graceful-restore on kill is not worth aborting over.
            let _ = signal_hook::flag::register(sig, flag.clone());
        }
        flag
    });
    flag.load(Ordering::Relaxed)
}

#[cfg(not(unix))]
pub fn quit_requested() -> bool {
    false
}

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
