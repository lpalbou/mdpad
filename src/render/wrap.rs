//! Style-preserving word wrap with correct Unicode display widths.
//!
//! ratatui's built-in wrapping can't tell us *how many* rows a paragraph
//! occupies, which we need for exact scrolling/search/TOC arithmetic. So we
//! pre-wrap: every emitted line is exactly one terminal row.

use ratatui::style::Style;
use ratatui::text::Span;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Display width of a string on the terminal (CJK-aware).
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// A word fragment carrying its style (a word can cross span boundaries,
/// e.g. `mark**down**`).
#[derive(Debug, Clone)]
struct Fragment {
    text: String,
    style: Style,
}

#[derive(Debug, Clone)]
struct Word {
    fragments: Vec<Fragment>,
    width: usize,
}

impl Word {
    fn new() -> Self {
        Self {
            fragments: Vec::new(),
            width: 0,
        }
    }

    fn push(&mut self, text: &str, style: Style) {
        self.width += display_width(text);
        // Merge into the previous fragment when the style is unchanged to
        // keep span counts low.
        if let Some(last) = self.fragments.last_mut()
            && last.style == style
        {
            last.text.push_str(text);
            return;
        }
        self.fragments.push(Fragment {
            text: text.to_string(),
            style,
        });
    }

    fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    fn into_spans(self) -> Vec<Span<'static>> {
        self.fragments
            .into_iter()
            .map(|f| Span::styled(f.text, f.style))
            .collect()
    }

    /// Split a word wider than `max` into display-width-bounded chunks along
    /// grapheme cluster boundaries (long URLs, hashes, CJK runs).
    fn hard_break(self, max: usize) -> Vec<Word> {
        let max = max.max(1);
        let mut out = Vec::new();
        let mut current = Word::new();
        for frag in &self.fragments {
            for g in frag.text.graphemes(true) {
                let w = display_width(g);
                if current.width + w > max && !current.is_empty() {
                    out.push(std::mem::replace(&mut current, Word::new()));
                }
                current.push(g, frag.style);
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
        out
    }
}

/// Wrap styled spans to `width` columns. Returns one `Vec<Span>` per visual
/// row. Whitespace runs collapse to single spaces; words wider than the full
/// width are broken on grapheme boundaries.
pub fn wrap_spans(spans: &[Span<'_>], width: usize) -> Vec<Vec<Span<'static>>> {
    let width = width.max(1);
    let words = split_words(spans);

    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    let mut queue: std::collections::VecDeque<Word> = words.into();
    while let Some(word) = queue.pop_front() {
        if word.width > width {
            // Wider than a whole line: start fresh and break into full-width
            // chunks (breaking into leftover-sized chunks would shred the
            // word into tiny pieces).
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current_width = 0;
            }
            let pieces = word.hard_break(width);
            if pieces.len() == 1 {
                // A single grapheme wider than the whole line (CJK/emoji at
                // width 1) is unbreakable: emit it on its own, slightly
                // over-wide line. Re-queueing it would loop forever.
                for piece in pieces {
                    current.extend(piece.into_spans());
                }
                lines.push(std::mem::take(&mut current));
                continue;
            }
            for piece in pieces.into_iter().rev() {
                queue.push_front(piece);
            }
            continue;
        }
        let sep = if current.is_empty() { 0 } else { 1 };
        if current_width + sep + word.width > width {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }
        if !current.is_empty() {
            // A space between two words of the same styled run (inline code
            // background, link underline, strikethrough) must carry that
            // style, or the run renders with visible holes at every space.
            let prev = current.last().map(|s| s.style);
            let next = word.fragments.first().map(|f| f.style);
            let sep_span = match (prev, next) {
                (Some(a), Some(b)) if a == b => Span::styled(" ".to_string(), a),
                _ => Span::raw(" "),
            };
            current.push(sep_span);
            current_width += 1;
        }
        current_width += word.width;
        current.extend(word.into_spans());
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

/// Tokenize spans into whitespace-separated words, preserving styles across
/// span boundaries within a word.
fn split_words(spans: &[Span<'_>]) -> Vec<Word> {
    let mut words = Vec::new();
    let mut current = Word::new();
    for span in spans {
        let style = span.style;
        for part in split_inclusive_whitespace(&span.content) {
            match part {
                Piece::Space => {
                    if !current.is_empty() {
                        words.push(std::mem::replace(&mut current, Word::new()));
                    }
                }
                Piece::Word(w) => current.push(w, style),
            }
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

enum Piece<'a> {
    Word(&'a str),
    Space,
}

/// Split into alternating word / whitespace pieces (whitespace collapsed).
fn split_inclusive_whitespace(s: &str) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    let mut word_start: Option<usize> = None;
    let mut last_was_space = false;
    for (i, ch) in s.char_indices() {
        if ch.is_whitespace() {
            if let Some(start) = word_start.take() {
                out.push(Piece::Word(&s[start..i]));
            }
            if !last_was_space {
                out.push(Piece::Space);
                last_was_space = true;
            }
        } else {
            last_was_space = false;
            if word_start.is_none() {
                word_start = Some(i);
            }
        }
    }
    if let Some(start) = word_start {
        out.push(Piece::Word(&s[start..]));
    }
    out
}

/// Total display width of a span slice.
pub fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|s| display_width(&s.content)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Modifier, Style};

    fn plain(lines: &[Vec<Span<'_>>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| l.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect()
    }

    #[test]
    fn wraps_at_word_boundaries() {
        let spans = [Span::raw("the quick brown fox jumps over the lazy dog")];
        let lines = wrap_spans(&spans, 10);
        for l in plain(&lines) {
            assert!(display_width(&l) <= 10, "line too wide: {l:?}");
        }
        assert_eq!(plain(&lines)[0], "the quick");
    }

    #[test]
    fn preserves_style_across_word_split_spans() {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        // "mark**down**" -> one word from two spans
        let spans = [Span::raw("mark"), Span::styled("down", bold)];
        let lines = wrap_spans(&spans, 20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].len(), 2);
        assert_eq!(lines[0][1].style, bold);
    }

    #[test]
    fn hard_breaks_overlong_words() {
        let spans = [Span::raw("aaaaaaaaaaaaaaaaaaaaaaaaa")]; // 25 chars
        let lines = wrap_spans(&spans, 10);
        assert_eq!(plain(&lines), vec!["aaaaaaaaaa", "aaaaaaaaaa", "aaaaa"]);
    }

    #[test]
    fn cjk_width_counts_double() {
        let spans = [Span::raw("你好 世界 你好 世界")];
        let lines = wrap_spans(&spans, 5);
        for l in plain(&lines) {
            assert!(display_width(&l) <= 5);
        }
    }

    #[test]
    fn collapses_whitespace_runs() {
        let spans = [Span::raw("a   b\t\tc")];
        let lines = wrap_spans(&spans, 80);
        assert_eq!(plain(&lines), vec!["a b c"]);
    }

    #[test]
    fn empty_input_yields_one_empty_line() {
        let lines = wrap_spans(&[], 10);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].is_empty());
    }

    #[test]
    fn spaces_inside_styled_runs_keep_the_style() {
        let bg = Style::default().bg(ratatui::style::Color::Indexed(236));
        let spans = [Span::styled("inline code here", bg)];
        let lines = wrap_spans(&spans, 40);
        assert_eq!(lines.len(), 1);
        for span in &lines[0] {
            assert_eq!(span.style, bg, "space lost its style: {:?}", span);
        }
    }

    #[test]
    fn spaces_between_different_styles_stay_plain() {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let spans = [Span::styled("bold", bold), Span::raw(" plain")];
        let lines = wrap_spans(&spans, 40);
        let sep = &lines[0][1];
        assert_eq!(sep.content.as_ref(), " ");
        assert_eq!(sep.style, Style::default());
    }

    #[test]
    fn wide_grapheme_at_width_one_terminates() {
        // Regression: CJK (width 2) at wrap width 1 must not loop forever.
        let lines = wrap_spans(&[Span::raw("你好世界")], 1);
        assert_eq!(lines.len(), 4, "one over-wide grapheme per line");
        let text: String = lines
            .iter()
            .flat_map(|l| l.iter().map(|s| s.content.as_ref()))
            .collect();
        assert_eq!(text, "你好世界", "no content lost");
    }

    #[test]
    fn emoji_at_tiny_width_terminates() {
        let lines = wrap_spans(&[Span::raw("🚀🚀")], 1);
        assert_eq!(lines.len(), 2);
    }
}
