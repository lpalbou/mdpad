//! Search state and match highlighting over rendered lines.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// One match: rendered line index + byte range within that line's plain text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Match {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Default)]
pub struct SearchState {
    pub query: String,
    pub matches: Vec<Match>,
    pub current: usize,
}

impl SearchState {
    pub fn is_active(&self) -> bool {
        !self.query.is_empty()
    }

    /// Recompute matches (case-insensitive when the query is all-lowercase,
    /// "smart case" like less/vim). `haystacks` holds (original, lowercased)
    /// plain text per rendered line, cached by the caller so incremental
    /// search does not re-derive the document on every keystroke.
    pub fn run(&mut self, haystacks: &[(String, String)]) {
        self.matches.clear();
        self.current = 0;
        if self.query.is_empty() {
            return;
        }
        let smart_insensitive = self.query.chars().all(|c| !c.is_uppercase());
        let needle = if smart_insensitive {
            self.query.to_lowercase()
        } else {
            self.query.clone()
        };
        for (idx, (text, lower)) in haystacks.iter().enumerate() {
            let hay: &str = if smart_insensitive { lower } else { text };
            // Offsets in the folded haystack drift from the original when
            // case folding changes byte lengths (İ grows, KELVIN SIGN
            // shrinks) — translation is needed unless the fold was a no-op.
            let needs_translation = smart_insensitive && lower != text;
            let mut from = 0;
            while let Some(pos) = hay[from..].find(&needle) {
                let fold_start = from + pos;
                let fold_end = fold_start + needle.len();
                let (start, end) = if needs_translation {
                    map_folded_range(text, fold_start, fold_end)
                } else {
                    (fold_start, fold_end)
                };
                if start < end && end <= text.len() {
                    self.matches.push(Match {
                        line: idx,
                        start,
                        end,
                    });
                }
                from = fold_end.max(from + 1);
                if from >= hay.len() {
                    break;
                }
            }
        }
    }

    pub fn next(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + 1) % self.matches.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + self.matches.len() - 1) % self.matches.len();
        }
    }

    pub fn current_match(&self) -> Option<Match> {
        self.matches.get(self.current).copied()
    }

    /// Jump to the first match at or after `line` (called right after `/`).
    pub fn seek_from(&mut self, line: usize) {
        if let Some(i) = self.matches.iter().position(|m| m.line >= line) {
            self.current = i;
        } else {
            self.current = 0;
        }
    }
}

/// Translate a byte range in the case-folded string back to the original.
/// Walks the original once, accumulating each char's folded length; ranges
/// that start or end inside one char's fold expand outward to cover the
/// whole char (safe over-highlight rather than a wrong slice).
fn map_folded_range(text: &str, fold_start: usize, fold_end: usize) -> (usize, usize) {
    let mut start = None;
    let mut fold_pos = 0usize;
    for (orig_pos, ch) in text.char_indices() {
        let fold_len: usize = ch.to_lowercase().map(|c| c.len_utf8()).sum();
        let fold_next = fold_pos + fold_len;
        if start.is_none() && fold_start < fold_next {
            start = Some(orig_pos);
        }
        if fold_end <= fold_next {
            return (start.unwrap_or(orig_pos), orig_pos + ch.len_utf8());
        }
        fold_pos = fold_next;
    }
    (start.unwrap_or(text.len()), text.len())
}

const MATCH_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Indexed(220))
    .add_modifier(Modifier::BOLD);
const CURRENT_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Indexed(208))
    .add_modifier(Modifier::BOLD);

/// Overlay match highlights onto a rendered line.
/// `ranges` are (start, end, is_current) byte ranges in the line's plain text.
pub fn highlight_line(line: &Line<'static>, ranges: &[(usize, usize, bool)]) -> Line<'static> {
    if ranges.is_empty() {
        return line.clone();
    }
    let mut out: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + ranges.len() * 2);
    let mut offset = 0usize; // byte offset in the concatenated plain text
    for span in &line.spans {
        let text = span.content.as_ref();
        let len = text.len();
        let span_start = offset;
        let span_end = offset + len;

        // Collect cut points within this span.
        let mut cursor = 0usize; // local byte index
        let mut segments: Vec<(usize, usize, Option<bool>)> = Vec::new();
        let mut overlapping: Vec<(usize, usize, bool)> = ranges
            .iter()
            .filter(|(s, e, _)| *s < span_end && *e > span_start)
            .map(|(s, e, c)| {
                (
                    s.saturating_sub(span_start).min(len),
                    (e - span_start).min(len),
                    *c,
                )
            })
            .collect();
        overlapping.sort_by_key(|r| r.0);
        for (s, e, current) in overlapping {
            if s > cursor {
                segments.push((cursor, s, None));
            }
            segments.push((s.max(cursor), e, Some(current)));
            cursor = cursor.max(e);
        }
        if cursor < len {
            segments.push((cursor, len, None));
        }
        if segments.is_empty() {
            segments.push((0, len, None));
        }

        let mut emitted = 0usize;
        for (s, e, highlight) in segments {
            if s >= e {
                continue;
            }
            // Guard char boundaries (defense in depth; offsets should be
            // valid). Emit the not-yet-emitted remainder — pushing the whole
            // span here would duplicate already-emitted segments.
            if !text.is_char_boundary(s) || !text.is_char_boundary(e) {
                out.push(Span::styled(text[emitted..].to_string(), span.style));
                break;
            }
            let seg = text[s..e].to_string();
            emitted = e;
            match highlight {
                Some(true) => out.push(Span::styled(seg, CURRENT_STYLE)),
                Some(false) => out.push(Span::styled(seg, MATCH_STYLE)),
                None => out.push(Span::styled(seg, span.style)),
            }
        }
        offset = span_end;
    }
    Line::from(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hay(items: &[&str]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|s| (s.to_string(), s.to_lowercase()))
            .collect()
    }

    #[test]
    fn smart_case_matches_insensitively() {
        let lines = hay(&["Hello World", "hello again"]);
        let mut st = SearchState {
            query: "hello".into(),
            ..Default::default()
        };
        st.run(&lines);
        assert_eq!(st.matches.len(), 2);
    }

    #[test]
    fn uppercase_query_is_sensitive() {
        let lines = hay(&["Hello World", "hello again"]);
        let mut st = SearchState {
            query: "Hello".into(),
            ..Default::default()
        };
        st.run(&lines);
        assert_eq!(st.matches.len(), 1);
        assert_eq!(st.matches[0].line, 0);
    }

    #[test]
    fn length_changing_case_fold_maps_offsets() {
        // 'İ' lowercases to two chars: the folded haystack is longer, so
        // offsets must be translated back, and other words on the same line
        // must stay case-insensitively findable.
        let lines = hay(&["İstanbul CITY here"]);
        let mut st = SearchState {
            query: "city".into(),
            ..Default::default()
        };
        st.run(&lines);
        assert_eq!(st.matches.len(), 1);
        let m = st.matches[0];
        assert_eq!(&lines[0].0[m.start..m.end], "CITY");
    }

    #[test]
    fn net_zero_drift_still_maps_correctly() {
        // KELVIN SIGN (3B -> 1B) + two İ (2B -> 3B each) = same total length
        // but internally drifted offsets; a naive equal-length shortcut
        // would mis-highlight. The 'x' sits at drifted positions.
        let text = "\u{212A}\u{0130}\u{0130}x";
        let lines = hay(&[text]);
        let mut st = SearchState {
            query: "x".into(),
            ..Default::default()
        };
        st.run(&lines);
        assert_eq!(st.matches.len(), 1);
        let m = st.matches[0];
        assert_eq!(&text[m.start..m.end], "x");
    }

    #[test]
    fn highlight_splits_spans() {
        let line = Line::from(vec![Span::raw("foo bar baz")]);
        let out = highlight_line(&line, &[(4, 7, true)]);
        let texts: Vec<&str> = out.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(texts, vec!["foo ", "bar", " baz"]);
    }

    #[test]
    fn highlight_across_span_boundary() {
        let line = Line::from(vec![Span::raw("foo b"), Span::raw("ar baz")]);
        let out = highlight_line(&line, &[(4, 7, false)]);
        let text: String = out.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "foo bar baz");
        // The match "b" + "ar" should be styled in both fragments.
        assert!(
            out.spans
                .iter()
                .any(|s| s.content.as_ref() == "b" && s.style == MATCH_STYLE)
        );
        assert!(
            out.spans
                .iter()
                .any(|s| s.content.as_ref() == "ar" && s.style == MATCH_STYLE)
        );
    }
}
