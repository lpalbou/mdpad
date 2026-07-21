//! Link tracking through the render pipeline.
//!
//! Link destinations live in the semantic model but the renderer flattens
//! everything to styled spans, and the wrapper rebuilds spans from scratch
//! keyed on `Style` equality alone. The one channel that survives wrapping,
//! table layout and prefixing untouched is the style itself — so each link
//! occurrence is stamped with a unique id encoded in `Style::underline_color`
//! (a field mdpad never uses for painting and print mode never emits).
//!
//! Style-equality merging gives exactly the right semantics for free:
//! fragments of one link merge, distinct links never merge, separator spaces
//! inside a link inherit its id, and a hard-broken URL keeps its id on every
//! visual line.
//!
//! After the final layout pass, [`extract_links`] converts the markers into
//! per-line [`LinkSpan`] ranges (byte offsets into the line's plain text,
//! the same convention search matches and selections use) and strips the
//! transport color so it can never reach the terminal.

use std::cell::RefCell;

use ratatui::style::Color;

use crate::render::RenderedLine;

/// A clickable range within one rendered line: byte offsets into the line's
/// plain text plus the link destination.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkSpan {
    pub start: usize,
    pub end: usize,
    pub target: String,
}

/// Collects link destinations during a render pass and hands out style
/// markers. Interior mutability because inline renderers share `&self`.
#[derive(Default)]
pub struct LinkRegistry {
    targets: RefCell<Vec<String>>,
}

/// 24 bits of id space (RGB payload). More links than this in one document
/// simply stop being clickable — they still render normally.
const MAX_LINKS: usize = 1 << 24;

impl LinkRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one link occurrence and return the style marker carrying its
    /// id. Every occurrence gets a fresh id, so two adjacent links to the
    /// same destination remain distinct clickable ranges.
    pub fn marker(&self, dest: &str) -> Option<Color> {
        let mut targets = self.targets.borrow_mut();
        if targets.len() >= MAX_LINKS {
            return None;
        }
        let id = targets.len();
        targets.push(dest.to_string());
        Some(encode(id))
    }

    fn target(&self, id: usize) -> Option<String> {
        self.targets.borrow().get(id).cloned()
    }
}

fn encode(id: usize) -> Color {
    Color::Rgb((id >> 16) as u8, (id >> 8) as u8, id as u8)
}

fn decode(color: Color) -> Option<usize> {
    match color {
        Color::Rgb(r, g, b) => Some(((r as usize) << 16) | ((g as usize) << 8) | b as usize),
        _ => None,
    }
}

/// Harvest link markers from finished lines: record contiguous same-id runs
/// as [`LinkSpan`]s and strip the transport color from every span. Must run
/// after all layout passes (prefixes, margins) so byte offsets are final.
pub fn extract_links(lines: &mut [RenderedLine], registry: &LinkRegistry) {
    for rl in lines {
        let mut links: Vec<LinkSpan> = Vec::new();
        // (id, start, end) of the run being accumulated.
        let mut run: Option<(usize, usize, usize)> = None;
        let mut offset = 0usize;
        for span in rl.line.spans.iter_mut() {
            let len = span.content.len();
            let id = span.style.underline_color.take().and_then(decode);
            match (id, &mut run) {
                (Some(id), Some((rid, _, end))) if *rid == id && *end == offset => {
                    *end = offset + len;
                }
                (id, run_slot) => {
                    flush(run_slot.take(), registry, &mut links);
                    if let Some(id) = id {
                        *run_slot = Some((id, offset, offset + len));
                    }
                }
            }
            offset += len;
        }
        flush(run.take(), registry, &mut links);
        rl.links = links;
    }
}

fn flush(run: Option<(usize, usize, usize)>, registry: &LinkRegistry, out: &mut Vec<LinkSpan>) {
    if let Some((id, start, end)) = run
        && let Some(target) = registry.target(id)
    {
        out.push(LinkSpan { start, end, target });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parser::parse;
    use crate::render::Renderer;
    use crate::render::highlight::Highlighter;
    use crate::render::inline::LinkMode;
    use crate::render::theme::{CharSet, Theme};

    fn render(src: &str, width: usize, link_mode: LinkMode, margin: usize) -> Vec<RenderedLine> {
        let renderer = Renderer {
            theme: Theme::dark(CharSet::unicode()),
            highlighter: Highlighter::new(false, false),
            link_mode,
            prose_cap: 0,
            margin,
            interactive: true,
        };
        renderer.render(&parse(src).blocks, width)
    }

    /// The text each LinkSpan covers, per line, resolved against plain text.
    fn covered(lines: &[RenderedLine]) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for rl in lines {
            let text = rl.plain_text();
            for l in &rl.links {
                out.push((text[l.start..l.end].to_string(), l.target.clone()));
            }
        }
        out
    }

    fn assert_no_marker_leak(lines: &[RenderedLine]) {
        for rl in lines {
            for span in &rl.line.spans {
                assert_eq!(
                    span.style.underline_color, None,
                    "transport color leaked into {:?}",
                    span.content
                );
            }
        }
    }

    #[test]
    fn named_link_range_matches_text() {
        let lines = render(
            "see [the docs](https://example.com/docs) now",
            80,
            LinkMode::TextOnly,
            0,
        );
        let cov = covered(&lines);
        assert_eq!(cov.len(), 1, "{cov:?}");
        assert_eq!(cov[0].0, "the docs");
        assert_eq!(cov[0].1, "https://example.com/docs");
        assert_no_marker_leak(&lines);
    }

    #[test]
    fn margin_shifts_offsets_correctly() {
        // Margin prepends a pad span after block layout; extraction must see
        // final offsets. Width >= 50 so the margin actually applies.
        let lines = render(
            "see [the docs](https://example.com/docs) now",
            60,
            LinkMode::TextOnly,
            2,
        );
        let cov = covered(&lines);
        assert_eq!(cov.len(), 1);
        assert_eq!(cov[0].0, "the docs");
    }

    #[test]
    fn adjacent_links_stay_distinct() {
        let lines = render("[a](x.md) [b](y.md)", 80, LinkMode::TextOnly, 0);
        let cov = covered(&lines);
        assert_eq!(cov.len(), 2, "{cov:?}");
        assert_eq!(cov[0], ("a".into(), "x.md".into()));
        assert_eq!(cov[1], ("b".into(), "y.md".into()));
    }

    #[test]
    fn wrapped_link_keeps_target_on_every_line() {
        // A long link text wraps across lines; each visual line must stay
        // clickable with the same target.
        let src = "[this is a very long link text that will definitely wrap](https://example.com)";
        let lines = render(src, 20, LinkMode::TextOnly, 0);
        let cov = covered(&lines);
        assert!(cov.len() > 1, "expected the link to wrap: {cov:?}");
        assert!(cov.iter().all(|(_, t)| t == "https://example.com"));
        assert_no_marker_leak(&lines);
    }

    #[test]
    fn hard_broken_autolink_keeps_target() {
        let src = "<https://example.com/a/very/long/path/that/cannot/fit>";
        let lines = render(src, 20, LinkMode::TextOnly, 0);
        let cov = covered(&lines);
        assert!(cov.len() > 1, "expected hard break: {cov:?}");
        let joined: String = cov.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(
            joined,
            "https://example.com/a/very/long/path/that/cannot/fit"
        );
    }

    #[test]
    fn with_url_suffix_is_clickable_too() {
        let lines = render("[docs](https://example.com/docs)", 80, LinkMode::WithUrl, 0);
        let all: String = covered(&lines).iter().map(|(s, _)| s.as_str()).collect();
        assert!(all.contains("docs"), "{all}");
        assert!(
            all.contains("example.com"),
            "url suffix not clickable: {all}"
        );
    }

    #[test]
    fn table_cell_links_are_registered() {
        let src = "| doc | link |\n|---|---|\n| api | [ref](docs/api.md) |\n";
        let lines = render(src, 60, LinkMode::TextOnly, 0);
        let cov = covered(&lines);
        assert_eq!(cov.len(), 1, "{cov:?}");
        assert_eq!(cov[0], ("ref".into(), "docs/api.md".into()));
        assert_no_marker_leak(&lines);
    }

    #[test]
    fn heading_and_list_prefixes_keep_offsets_valid() {
        let src = "### see [docs](a.md)\n\n- item with [link](b.md) inside\n";
        let lines = render(src, 80, LinkMode::TextOnly, 0);
        let cov = covered(&lines);
        assert!(cov.contains(&("docs".into(), "a.md".into())), "{cov:?}");
        assert!(cov.contains(&("link".into(), "b.md".into())), "{cov:?}");
    }

    #[test]
    fn image_destination_is_clickable() {
        let lines = render("![alt text](img/pic.png)", 80, LinkMode::TextOnly, 0);
        let cov = covered(&lines);
        assert_eq!(cov.len(), 1, "{cov:?}");
        assert_eq!(cov[0].1, "img/pic.png");
    }

    #[test]
    fn documents_without_links_have_no_spans() {
        let lines = render("plain **bold** text\n\n- a list", 80, LinkMode::TextOnly, 0);
        assert!(covered(&lines).is_empty());
    }

    #[test]
    fn encode_decode_round_trip() {
        for id in [0usize, 1, 255, 256, 65_535, 65_536, MAX_LINKS - 1] {
            assert_eq!(decode(encode(id)), Some(id));
        }
    }
}
