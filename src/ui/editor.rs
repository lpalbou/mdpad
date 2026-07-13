//! Built-in raw-markdown editor (tui-textarea) and $EDITOR integration.

use std::path::Path;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use tui_textarea::TextArea;

pub struct Editor {
    pub textarea: TextArea<'static>,
    /// Buffer differs from the last saved content.
    pub dirty: bool,
    /// The file mixes CRLF and LF; saving unifies to CRLF (user is warned).
    pub mixed_endings: bool,
    /// Preserved so an open+save cycle never rewrites line endings.
    line_ending: &'static str,
    /// Preserved so files without a trailing newline round-trip unchanged.
    final_newline: bool,
    /// Content at the last save; lets undo back to a clean state clear the
    /// dirty flag.
    last_saved: String,
}

impl Editor {
    pub fn new(source: &str) -> Self {
        // str::lines() strips \r, so CRLF files edit cleanly; remember the
        // original convention to restore it on save.
        let line_ending = if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mixed_endings = line_ending == "\r\n" && source.replace("\r\n", "").contains('\n');
        // NOT `is_empty || ends_with`: a file of exactly "\n" must keep its
        // newline (lines() yields [""] which joins back to "").
        let final_newline = source.ends_with('\n');
        let mut textarea = TextArea::from(source.lines().map(String::from));
        textarea.set_line_number_style(Style::default().fg(Color::Indexed(240)));
        textarea.set_cursor_line_style(Style::default().bg(Color::Indexed(235)));
        textarea.set_selection_style(Style::default().bg(Color::Indexed(238)));
        textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut editor = Self {
            textarea,
            dirty: false,
            mixed_endings,
            line_ending,
            final_newline,
            last_saved: String::new(),
        };
        editor.last_saved = editor.content();
        editor
    }

    pub fn content(&self) -> String {
        let mut out = self.textarea.lines().join(self.line_ending);
        if self.final_newline {
            out.push_str(self.line_ending);
        }
        out
    }

    /// Recompute dirty after undo/redo may have returned to the saved state.
    pub fn refresh_dirty(&mut self) {
        self.dirty = self.content() != self.last_saved;
    }

    /// Atomic save: write a temp file next to the target and rename it into
    /// place, so a crash or full disk mid-write can never destroy the
    /// original. Resolving symlinks first keeps writes going *through* them.
    pub fn save(&mut self, path: &Path) -> std::io::Result<()> {
        let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let dir = target.parent().unwrap_or_else(|| Path::new("."));
        let name = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "save".into());
        let tmp = dir.join(format!(".{name}.mdpad-tmp"));

        std::fs::write(&tmp, self.content())?;
        // Keep the original file's permissions (best effort).
        if let Ok(meta) = std::fs::metadata(&target) {
            let _ = std::fs::set_permissions(&tmp, meta.permissions());
        }
        if let Err(e) = std::fs::rename(&tmp, &target) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        self.last_saved = self.content();
        self.dirty = false;
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.textarea, area);
    }
}

/// Run $VISUAL / $EDITOR on `path`. The caller must suspend/resume the TUI.
///
/// The variable may contain arguments ("code --wait") or a quoted path with
/// spaces, so it is executed through the shell (the same convention git
/// uses) rather than split naively.
pub fn spawn_external(path: &Path) -> Result<(), String> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .ok()
        .filter(|e| !e.trim().is_empty());

    #[cfg(unix)]
    let (editor, mut command) = {
        let Some(editor) = editor else {
            return Err("$EDITOR is not set".into());
        };
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(format!("exec {editor} \"$0\"")).arg(path);
        (editor, cmd)
    };

    #[cfg(not(unix))]
    let (editor, mut command) = {
        // Windows: %EDITOR% is rarely set; notepad is always present.
        let editor = editor.unwrap_or_else(|| "notepad".to_string());
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/C")
            .arg(format!("{editor} \"{}\"", path.display()));
        (editor, cmd)
    };

    let status = command
        .status()
        .map_err(|e| format!("failed to run {editor}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{editor} exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lf_file_round_trips() {
        let src = "# a\n\ntext\n";
        assert_eq!(Editor::new(src).content(), src);
    }

    #[test]
    fn crlf_file_round_trips() {
        let src = "# a\r\n\r\ntext\r\n";
        assert_eq!(Editor::new(src).content(), src);
    }

    #[test]
    fn missing_final_newline_round_trips() {
        let src = "# a\n\ntext";
        assert_eq!(Editor::new(src).content(), src);
    }

    #[test]
    fn empty_file_round_trips() {
        assert_eq!(Editor::new("").content(), "");
    }

    #[test]
    fn newline_only_file_round_trips() {
        // Regression: "\n".lines() == [""], joins to "" — the trailing
        // newline must survive.
        assert_eq!(Editor::new("\n").content(), "\n");
        assert_eq!(Editor::new("\r\n").content(), "\r\n");
    }

    #[test]
    fn mixed_endings_detected() {
        assert!(Editor::new("a\r\nb\nc\r\n").mixed_endings);
        assert!(!Editor::new("a\r\nb\r\n").mixed_endings);
        assert!(!Editor::new("a\nb\n").mixed_endings);
    }

    #[test]
    fn atomic_save_writes_and_round_trips() {
        let dir = std::env::temp_dir().join(format!("mdpad-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc.md");
        std::fs::write(&path, "# hello\n").unwrap();
        let mut ed = Editor::new("# hello\n");
        ed.dirty = true;
        ed.save(&path).unwrap();
        assert!(!ed.dirty);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# hello\n");
        // No temp file left behind.
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
