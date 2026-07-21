//! Inline elements -> styled ratatui spans.
//!
//! Styles compose: bold inside emphasis inside a link accumulates all three.
//! Links render as underlined text; the destination URL is shown inline only
//! when it differs from the text and fits the "compact" rule (short URLs),
//! otherwise readers use the TUI's link mode / footer.

use ratatui::style::Style;
use ratatui::text::Span;

use crate::markdown::model::Inline;
use crate::render::links::LinkRegistry;
use crate::render::theme::Theme;

/// How link destinations are surfaced next to the link text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    /// Only the (underlined) link text.
    TextOnly,
    /// Append ` (url)` after the link text.
    WithUrl,
}

pub struct InlineRenderer<'t> {
    theme: &'t Theme,
    link_mode: LinkMode,
    /// Shared per-render registry; links stamp their id into span styles so
    /// destinations survive wrapping (see `render::links`).
    links: &'t LinkRegistry,
}

impl<'t> InlineRenderer<'t> {
    pub fn new(theme: &'t Theme, link_mode: LinkMode, links: &'t LinkRegistry) -> Self {
        Self {
            theme,
            link_mode,
            links,
        }
    }

    /// Style for one link occurrence: visual link style + transport marker.
    fn mark(&self, base: Style, dest: &str) -> Style {
        let mut style = base;
        style.underline_color = self.links.marker(dest);
        style
    }

    /// Render inlines with `base` as the ambient style.
    pub fn render(&self, inlines: &[Inline], base: Style) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        self.walk(inlines, base, &mut spans);
        spans
    }

    fn walk(&self, inlines: &[Inline], style: Style, out: &mut Vec<Span<'static>>) {
        for inline in inlines {
            match inline {
                Inline::Text(t) => out.push(Span::styled(t.clone(), style)),
                Inline::Code(t) => {
                    // No padding spaces: the wrapper treats spaces as word
                    // separators and would strip the padding (and its
                    // background) at wrap points. Colorless themes re-emit
                    // backticks instead — their only way to mark code.
                    let text = if self.theme.code_delims {
                        format!("`{t}`")
                    } else {
                        t.clone()
                    };
                    out.push(Span::styled(text, style.patch(self.theme.inline_code)));
                }
                Inline::Strong(children) => {
                    self.walk(children, style.patch(self.theme.strong), out)
                }
                Inline::Emph(children) => self.walk(children, style.patch(self.theme.emph), out),
                Inline::Strikethrough(children) => {
                    self.walk(children, style.patch(self.theme.strike), out)
                }
                Inline::Link { dest, children } => {
                    let link_style = self.mark(style.patch(self.theme.link), dest);
                    if children.is_empty() {
                        out.push(Span::styled(dest.clone(), link_style));
                        continue;
                    }
                    self.walk(children, link_style, out);
                    if self.link_mode == LinkMode::WithUrl && !is_redundant_link(dest, children) {
                        out.push(Span::styled(
                            format!(" ({dest})"),
                            self.mark(style.patch(self.theme.link_url), dest),
                        ));
                    }
                }
                Inline::Image { dest, alt } => {
                    let marker = self.theme.chars.image_marker;
                    let label = if alt.is_empty() { "image" } else { alt };
                    out.push(Span::styled(
                        format!("{marker} {label}"),
                        self.mark(style.patch(self.theme.image), dest),
                    ));
                    if self.link_mode == LinkMode::WithUrl {
                        out.push(Span::styled(
                            format!(" ({dest})"),
                            self.mark(style.patch(self.theme.link_url), dest),
                        ));
                    }
                }
                Inline::Html(t) => out.push(Span::styled(t.clone(), style.patch(self.theme.html))),
                Inline::FootnoteRef(label) => out.push(Span::styled(
                    format!("[^{label}]"),
                    style.patch(self.theme.footnote),
                )),
                Inline::SoftBreak => out.push(Span::styled(" ".to_string(), style)),
                Inline::HardBreak => {
                    // Encoded as a newline character; the block renderer
                    // splits on it before wrapping.
                    out.push(Span::styled("\n".to_string(), style));
                }
            }
        }
    }
}

/// `[https://x.y](https://x.y)` — showing the URL twice is noise.
fn is_redundant_link(dest: &str, children: &[Inline]) -> bool {
    if let [Inline::Text(t)] = children {
        let t = t.trim();
        t == dest
            || dest.strip_prefix("https://") == Some(t)
            || dest.strip_prefix("http://") == Some(t)
            || dest.strip_prefix("mailto:") == Some(t)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::model::Block;
    use crate::markdown::parser::parse;
    use crate::render::theme::{CharSet, Theme};

    fn spans_text(spans: &[Span<'_>]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn first_paragraph(src: &str) -> Vec<Inline> {
        match parse(src).blocks.into_iter().next() {
            Some(Block::Paragraph(inlines)) => inlines,
            other => panic!("expected paragraph, got {other:?}"),
        }
    }

    #[test]
    fn nested_styles_compose() {
        let theme = Theme::dark(CharSet::unicode());
        let links = LinkRegistry::new();
        let renderer = InlineRenderer::new(&theme, LinkMode::TextOnly, &links);
        let inlines = first_paragraph("**bold _italic_**");
        let spans = renderer.render(&inlines, Style::default());
        let italic = spans
            .iter()
            .find(|s| s.content.as_ref() == "italic")
            .expect("italic span");
        assert!(
            italic
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::BOLD)
        );
        assert!(
            italic
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC)
        );
    }

    #[test]
    fn redundant_url_not_repeated() {
        let theme = Theme::dark(CharSet::unicode());
        let links = LinkRegistry::new();
        let renderer = InlineRenderer::new(&theme, LinkMode::WithUrl, &links);
        let inlines = first_paragraph("<https://example.com>");
        let spans = renderer.render(&inlines, Style::default());
        let text = spans_text(&spans);
        assert_eq!(text.matches("example.com").count(), 1, "{text}");
    }

    #[test]
    fn named_link_shows_url_in_with_url_mode() {
        let theme = Theme::dark(CharSet::unicode());
        let links = LinkRegistry::new();
        let renderer = InlineRenderer::new(&theme, LinkMode::WithUrl, &links);
        let inlines = first_paragraph("[docs](https://example.com/docs)");
        let spans = renderer.render(&inlines, Style::default());
        let text = spans_text(&spans);
        assert!(text.contains("docs (https://example.com/docs)"), "{text}");
    }
}
