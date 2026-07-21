//! Following links and navigating between documents.
//!
//! Classifies link destinations (external URL / local file / in-document
//! anchor), resolves local paths against the current document, opens
//! external targets through the OS handler, and keeps the back-history
//! entries used by Backspace. The app layer owns the state; this module
//! owns the logic.

use std::path::{Path, PathBuf};

use crate::render::RenderedLine;
use crate::ui::selection::col_to_byte_floor;
use crate::ui::toc::TocState;

/// One document the reader can go back to. The source is kept in memory so
/// back is instant and works for documents without a path (stdin) or whose
/// file has since disappeared.
pub struct HistoryEntry {
    pub source: String,
    pub path: Option<PathBuf>,
    pub scroll: usize,
}

/// What a link destination means for navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkKind {
    /// Has a URI scheme: hand off to the OS (browser, mail client, ...).
    External(String),
    /// A file to open in the viewer, with an optional `#fragment` to jump to.
    Local {
        path: String,
        anchor: Option<String>,
    },
    /// `#fragment` within the current document.
    Anchor(String),
}

/// Classify a raw markdown destination.
pub fn classify(dest: &str) -> LinkKind {
    let dest = dest.trim();
    if let Some(anchor) = dest.strip_prefix('#') {
        return LinkKind::Anchor(anchor.to_string());
    }
    if has_scheme(dest) {
        return LinkKind::External(dest.to_string());
    }
    let (path, anchor) = match dest.split_once('#') {
        Some((p, a)) if !a.is_empty() => (p, Some(a.to_string())),
        Some((p, _)) => (p, None),
        None => (dest, None),
    };
    LinkKind::Local {
        path: percent_decode(path),
        anchor,
    }
}

/// RFC 3986 scheme: ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ) followed by
/// ":". Single-letter "schemes" are treated as paths so Windows drive
/// letters (`C:\notes.md`) stay local.
fn has_scheme(dest: &str) -> bool {
    let Some((scheme, _)) = dest.split_once(':') else {
        return false;
    };
    scheme.len() >= 2
        && scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Decode `%XX` escapes (links in READMEs commonly encode spaces as `%20`).
/// Invalid escapes pass through literally; only valid UTF-8 results are
/// kept, otherwise the original string is returned unchanged.
fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2]))
        {
            out.push(hi << 4 | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Resolve a local link target against the current document's directory
/// (stdin documents resolve against the working directory).
pub fn resolve_local(doc_path: Option<&Path>, target: &str) -> PathBuf {
    let target = Path::new(target);
    if target.is_absolute() {
        return target.to_path_buf();
    }
    match doc_path.and_then(Path::parent) {
        Some(dir) => dir.join(target),
        None => target.to_path_buf(),
    }
}

/// The link destination under a document cell, if any.
pub fn link_at(lines: &[RenderedLine], line: usize, col: usize) -> Option<&str> {
    let rl = lines.get(line)?;
    if rl.links.is_empty() {
        return None;
    }
    let text = rl.plain_text();
    let byte = col_to_byte_floor(&text, col);
    rl.links
        .iter()
        .find(|l| l.start <= byte && byte < l.end)
        .map(|l| l.target.as_str())
}

/// Rendered line of the heading matching `#anchor`, GitHub slug style.
pub fn find_anchor_line(toc: &TocState, anchor: &str) -> Option<usize> {
    let wanted = slugify(&percent_decode(anchor));
    toc.entries
        .iter()
        .find(|e| slugify(&e.text) == wanted)
        .map(|e| e.line)
}

/// GitHub-style heading slug: lowercase, spaces to hyphens, punctuation
/// dropped (hyphens kept). "CLI & keys" -> "cli--keys".
pub fn slugify(text: &str) -> String {
    text.trim()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c.to_lowercase().next().unwrap_or(c))
            } else if c == ' ' {
                Some('-')
            } else if c == '-' || c == '_' {
                Some(c)
            } else {
                None
            }
        })
        .collect()
}

/// Open an external target with the OS handler (browser, mail client, ...).
/// Spawn-and-forget with all stdio nulled: the TUI owns the terminal, and
/// waiting on `xdg-open` can block until the launched application exits.
pub fn open_external(url: &str) -> Result<(), String> {
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut cmd = Command::new("open");
        cmd.arg(url);
        cmd
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(url);
        cmd
    };

    #[cfg(windows)]
    let mut command = {
        // `start` is a cmd builtin; the empty "" is the window title slot so
        // a quoted URL is not mistaken for it. Quoting keeps `&` intact.
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("cmd");
        cmd.raw_arg(format!("/C start \"\" \"{}\"", url.replace('"', "%22")));
        cmd
    };

    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("could not open {url}: {e}"))?;
    // Reap in a detached thread: a dropped Child would linger as a zombie
    // until mdpad exits (one per followed link). Launchers exit in
    // milliseconds, so the thread is short-lived; never block the UI on it.
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::links::LinkSpan;
    use ratatui::text::{Line, Span};

    #[test]
    fn classify_external_schemes() {
        assert_eq!(
            classify("https://example.com/x?a=1"),
            LinkKind::External("https://example.com/x?a=1".into())
        );
        assert_eq!(
            classify("mailto:a@b.c"),
            LinkKind::External("mailto:a@b.c".into())
        );
    }

    #[test]
    fn classify_anchors_and_local_paths() {
        assert_eq!(
            classify("#getting-started"),
            LinkKind::Anchor("getting-started".into())
        );
        assert_eq!(
            classify("docs/api.md"),
            LinkKind::Local {
                path: "docs/api.md".into(),
                anchor: None
            }
        );
        assert_eq!(
            classify("docs/api.md#viewer-keys"),
            LinkKind::Local {
                path: "docs/api.md".into(),
                anchor: Some("viewer-keys".into())
            }
        );
    }

    #[test]
    fn windows_drive_letters_are_local() {
        assert_eq!(
            classify(r"C:\notes\todo.md"),
            LinkKind::Local {
                path: r"C:\notes\todo.md".into(),
                anchor: None
            }
        );
    }

    #[test]
    fn percent_encoded_paths_decode() {
        assert_eq!(
            classify("my%20notes.md"),
            LinkKind::Local {
                path: "my notes.md".into(),
                anchor: None
            }
        );
        // Invalid escapes survive literally.
        assert_eq!(percent_decode("50%zz"), "50%zz");
    }

    #[test]
    fn resolve_against_document_directory() {
        let doc = PathBuf::from("/home/me/proj/README.md");
        assert_eq!(
            resolve_local(Some(&doc), "docs/api.md"),
            PathBuf::from("/home/me/proj/docs/api.md")
        );
        assert_eq!(
            resolve_local(None, "docs/api.md"),
            PathBuf::from("docs/api.md")
        );
        #[cfg(unix)]
        assert_eq!(
            resolve_local(Some(&doc), "/etc/hosts"),
            PathBuf::from("/etc/hosts")
        );
    }

    #[test]
    fn slugs_match_github_style() {
        assert_eq!(slugify("Getting Started"), "getting-started");
        assert_eq!(slugify("CLI & keys"), "cli--keys");
        assert_eq!(slugify("What's new?"), "whats-new");
    }

    #[test]
    fn link_at_end_to_end_through_real_render() {
        // Full pipeline with the TUI's default config (margin, prose cap):
        // a click column inside the link text must resolve to its target.
        use crate::markdown::parser::parse;
        use crate::render::Renderer;
        use crate::render::highlight::Highlighter;
        use crate::render::inline::LinkMode;
        use crate::render::theme::{CharSet, Theme};
        let renderer = Renderer {
            theme: Theme::dark(CharSet::unicode()),
            highlighter: Highlighter::new(true, false),
            link_mode: LinkMode::TextOnly,
            prose_cap: 100,
            margin: 2,
            interactive: true,
        };
        let lines = renderer.render(&parse("[go here](target.md) and text\n").blocks, 80);
        // Margin puts "go here" at columns 2..9; column 5 is inside.
        assert_eq!(link_at(&lines, 0, 5), Some("target.md"));
        assert_eq!(link_at(&lines, 0, 1), None, "margin is not clickable");
        assert_eq!(link_at(&lines, 0, 12), None, "prose after link");
    }

    #[test]
    fn link_at_maps_columns_through_wide_chars() {
        // "日本 link": 日=cols 0-1, 本=cols 2-3, space=col 4, link=cols 5-8.
        // Plain-text bytes: 日本=0..6, space=6, link=7..11.
        let mut rl = RenderedLine::plain(Line::from(Span::raw("日本 link")));
        rl.links = vec![LinkSpan {
            start: 7,
            end: 11,
            target: "x.md".into(),
        }];
        let lines = vec![rl];
        assert_eq!(link_at(&lines, 0, 5), Some("x.md"));
        assert_eq!(link_at(&lines, 0, 8), Some("x.md"));
        assert_eq!(link_at(&lines, 0, 4), None, "space before link");
        assert_eq!(link_at(&lines, 0, 20), None, "past end of line");
        assert_eq!(link_at(&lines, 1, 0), None, "line out of range");
    }
}
