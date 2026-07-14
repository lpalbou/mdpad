//! System clipboard writes, fired through two complementary channels.
//!
//! 1. OSC 52: an escape sequence asking the *terminal emulator* to set the
//!    clipboard. Works over SSH with zero native dependencies, but support
//!    varies (macOS Terminal.app and legacy Windows conhost ignore it) and
//!    failure is silent — undetectable from the application side.
//! 2. Native OS clipboard (arboard: NSPasteboard / Win32 / X11 + Wayland).
//!    Reliable locally, including terminals without OSC 52; useless over
//!    SSH, where the clipboard lives on the other machine.
//!
//! Every copy fires both. OSC 52 goes *first*: some terminals truncate
//! large OSC 52 payloads, so when both channels land the native
//! full-fidelity write must win.
//!
//! The arboard handle lives for the whole process: under X11 the clipboard
//! contents belong to the process that set them, so dropping the handle
//! right after `set_text` could forfeit the selection immediately.

use std::io::{Write, stdout};
use std::sync::{Mutex, OnceLock};

use arboard::Clipboard;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

fn native() -> &'static Mutex<Option<Clipboard>> {
    static NATIVE: OnceLock<Mutex<Option<Clipboard>>> = OnceLock::new();
    // Lazy: connecting to the windowing system costs a few milliseconds and
    // fails in headless/SSH sessions — where OSC 52 is the only channel.
    NATIVE.get_or_init(|| Mutex::new(Clipboard::new().ok()))
}

/// Send `text` to the system clipboard. Ok when at least one channel
/// accepted the write (OSC 52 acceptance means the terminal received the
/// bytes; whether it honors them cannot be observed).
pub fn copy(text: &str) -> std::io::Result<()> {
    let osc = write_osc52(text);
    let native_ok = native()
        .lock()
        .ok()
        .and_then(|mut cb| cb.as_mut().map(|cb| cb.set_text(text.to_string()).is_ok()))
        .unwrap_or(false);
    if native_ok { Ok(()) } else { osc }
}

/// Emitting while ratatui owns the screen is safe: OSC sequences carry no
/// cursor movement or printable output, so the alternate-screen content is
/// untouched.
fn write_osc52(text: &str) -> std::io::Result<()> {
    let mut out = stdout().lock();
    out.write_all(osc52(text).as_bytes())?;
    out.flush()
}

fn osc52(text: &str) -> String {
    // "c" targets the clipboard selection (not the X11 primary selection).
    format!("\x1b]52;c;{}\x07", STANDARD.encode(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_is_well_formed() {
        let seq = osc52("hello");
        assert!(seq.starts_with("\x1b]52;c;"));
        assert!(seq.ends_with('\x07'));
        assert_eq!(&seq[7..seq.len() - 1], "aGVsbG8=");
    }

    #[test]
    fn payload_is_pure_base64_even_for_multiline_unicode() {
        let seq = osc52("línea 1\nline 2\t日本");
        let payload = &seq[7..seq.len() - 1];
        assert!(
            payload
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
        );
    }
}
