//! Mouse text selection over the rendered document.
//!
//! Coordinates are *document* cells: (rendered-line index, display column).
//! Every rendered line occupies exactly one terminal row and the left margin
//! is baked into its spans, so screen row/column map to document line/column
//! by pure arithmetic — and a selection survives scrolling untouched because
//! it is anchored to lines, not to the screen.
//!
//! Both endpoints are inclusive cells (the cell under the cursor belongs to
//! the selection), matching how terminal emulators select natively. What you
//! copy is what you see: this is screen-faithful selection, complementary to
//! a future semantic block copy.

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthChar;

use crate::render::RenderedLine;

/// One selection endpoint in document cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

/// An in-progress or finished mouse selection.
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    anchor: Pos,
    head: Pos,
}

impl Selection {
    pub fn begin(line: usize, col: usize) -> Self {
        let p = Pos { line, col };
        Self { anchor: p, head: p }
    }

    pub fn drag_to(&mut self, line: usize, col: usize) {
        self.head = Pos { line, col };
    }

    /// A click without a drag selects nothing.
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Endpoints in document order (dragging upward/leftward is normalized).
    fn ordered(&self) -> (Pos, Pos) {
        if (self.head.line, self.head.col) < (self.anchor.line, self.anchor.col) {
            (self.head, self.anchor)
        } else {
            (self.anchor, self.head)
        }
    }

    /// Byte range this selection covers within one line's plain text.
    /// `None` when the line is outside the selection; `Some((n, n))` when the
    /// line is inside but contributes no text (blank line, sweep past EOL).
    pub fn byte_range(&self, line_idx: usize, text: &str) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }
        let (a, b) = self.ordered();
        if line_idx < a.line || line_idx > b.line {
            return None;
        }
        let start = if line_idx == a.line {
            col_to_byte_floor(text, a.col)
        } else {
            0
        };
        let end = if line_idx == b.line {
            col_to_byte_ceil(text, b.col)
        } else {
            text.len()
        };
        Some((start, end))
    }

    /// Selected text, newline-joined. Segments reaching the visual end of
    /// their line are right-trimmed: rendered lines carry background padding
    /// (code blocks) and fill spaces a paste buffer should not inherit.
    pub fn extract(&self, lines: &[RenderedLine]) -> String {
        if lines.is_empty() {
            return String::new();
        }
        let (a, b) = self.ordered();
        let last = b.line.min(lines.len() - 1);
        let mut out = String::new();
        for (idx, rl) in lines.iter().enumerate().take(last + 1).skip(a.line) {
            let text = rl.plain_text();
            let Some((s, e)) = self.byte_range(idx, &text) else {
                continue;
            };
            let seg = if e == text.len() {
                text[s..e].trim_end()
            } else {
                &text[s..e]
            };
            if idx > a.line {
                out.push('\n');
            }
            out.push_str(seg);
        }
        out
    }
}

/// Byte offset of the character whose display cells contain column `col`
/// (selection start: round down to the char boundary). Past-EOL -> len.
fn col_to_byte_floor(text: &str, col: usize) -> usize {
    let mut cells = 0usize;
    for (i, ch) in text.char_indices() {
        let w = ch.width().unwrap_or(0);
        if w > 0 && cells + w > col {
            return i;
        }
        cells += w;
    }
    text.len()
}

/// Byte offset just past the character whose display cells contain column
/// `col` (selection end: the cell under the cursor is included). Trailing
/// zero-width characters (combining marks) travel with their base character
/// so a slice never orphans an accent.
fn col_to_byte_ceil(text: &str, col: usize) -> usize {
    let mut cells = 0usize;
    let mut iter = text.char_indices().peekable();
    while let Some((i, ch)) = iter.next() {
        let w = ch.width().unwrap_or(0);
        if w > 0 && cells + w > col {
            let mut end = i + ch.len_utf8();
            while let Some(&(j, next)) = iter.peek() {
                if next.width().unwrap_or(0) == 0 {
                    end = j + next.len_utf8();
                    iter.next();
                } else {
                    break;
                }
            }
            return end;
        }
        cells += w;
    }
    text.len()
}

/// Repaint `start..end` (byte offsets into the line's plain text) with
/// REVERSED video, preserving the underlying styling. REVERSED adapts to any
/// theme (dark, light, plain) without hardcoding a selection color.
pub fn highlight_selection(line: &Line<'static>, start: usize, end: usize) -> Line<'static> {
    if start >= end {
        return line.clone();
    }
    let mut out: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 2);
    let mut offset = 0usize;
    for span in &line.spans {
        let text = span.content.as_ref();
        let len = text.len();
        let s = start.saturating_sub(offset).min(len);
        let e = end.saturating_sub(offset).min(len);
        // Defense in depth, mirroring search highlighting: never slice
        // mid-char even if offsets drift.
        if s >= e || !text.is_char_boundary(s) || !text.is_char_boundary(e) {
            out.push(span.clone());
        } else {
            if s > 0 {
                out.push(Span::styled(text[..s].to_string(), span.style));
            }
            out.push(Span::styled(
                text[s..e].to_string(),
                span.style.add_modifier(Modifier::REVERSED),
            ));
            if e < len {
                out.push(Span::styled(text[e..].to_string(), span.style));
            }
        }
        offset += len;
    }
    Line::from(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl(s: &str) -> RenderedLine {
        RenderedLine::plain(Line::from(Span::raw(s.to_string())))
    }

    #[test]
    fn single_line_ascii() {
        let lines = vec![rl("hello world")];
        let mut sel = Selection::begin(0, 6);
        sel.drag_to(0, 10); // cells 6..=10 = "world"
        assert_eq!(sel.extract(&lines), "world");
    }

    #[test]
    fn reversed_drag_normalizes() {
        let lines = vec![rl("hello world")];
        let mut sel = Selection::begin(0, 10);
        sel.drag_to(0, 6);
        assert_eq!(sel.extract(&lines), "world");
    }

    #[test]
    fn cjk_cells_map_to_char_boundaries() {
        // Each ideograph spans 2 cells: 日=0-1, 本=2-3, 語=4-5.
        let lines = vec![rl("日本語")];
        let mut sel = Selection::begin(0, 2);
        sel.drag_to(0, 3); // both cells of 本
        assert_eq!(sel.extract(&lines), "本");

        let mut sel = Selection::begin(0, 1); // second cell of 日
        sel.drag_to(0, 2); // first cell of 本
        assert_eq!(sel.extract(&lines), "日本");
    }

    #[test]
    fn multiline_with_blank_line_and_trailing_padding() {
        let lines = vec![rl("first line   "), rl(""), rl("last")];
        let mut sel = Selection::begin(0, 6);
        sel.drag_to(2, 1); // from "line" through the blank into "la"
        assert_eq!(sel.extract(&lines), "line\n\nla");
    }

    #[test]
    fn sweep_past_eol_trims_padding() {
        let lines = vec![rl("abc   ")];
        let mut sel = Selection::begin(0, 0);
        sel.drag_to(0, 40);
        assert_eq!(sel.extract(&lines), "abc");
    }

    #[test]
    fn click_without_drag_is_empty() {
        let sel = Selection::begin(3, 7);
        assert!(sel.is_empty());
        assert_eq!(sel.byte_range(3, "whatever"), None);
    }

    #[test]
    fn middle_lines_select_fully_and_outside_lines_dont() {
        let mut sel = Selection::begin(0, 3);
        sel.drag_to(2, 1);
        assert_eq!(sel.byte_range(1, "middle"), Some((0, 6)));
        assert_eq!(sel.byte_range(3, "after"), None);
    }

    #[test]
    fn combining_mark_travels_with_base() {
        let lines = vec![rl("e\u{301}xy")]; // é (e + combining acute), x, y
        let mut sel = Selection::begin(0, 0);
        sel.drag_to(0, 1); // cells of é and x
        assert_eq!(sel.extract(&lines), "e\u{301}x");
    }

    #[test]
    fn highlight_splits_spans_and_reverses() {
        let line = Line::from(vec![Span::raw("foo"), Span::raw("bar")]);
        let out = highlight_selection(&line, 2, 4); // "ob" across the boundary
        let reversed: String = out
            .spans
            .iter()
            .filter(|s| s.style.add_modifier.contains(Modifier::REVERSED))
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(reversed, "ob");
        // Text content must be preserved byte-for-byte.
        let all: String = out.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(all, "foobar");
    }
}
