//! Rendering: semantic blocks -> pre-wrapped styled lines -> TUI or ANSI.

pub mod ansi;
pub mod block;
pub mod highlight;
pub mod inline;
pub mod links;
pub mod mermaid;
pub mod table;
pub mod theme;
pub mod wrap;

use ratatui::text::{Line, Span};

use crate::markdown::model::Block;
use block::BlockRenderer;
use highlight::Highlighter;
use inline::LinkMode;
use links::{LinkRegistry, LinkSpan};
use theme::Theme;

/// Anchor recorded on the first visual line of each heading (feeds the TOC).
#[derive(Debug, Clone)]
pub struct HeadingAnchor {
    pub level: u8,
    pub text: String,
}

/// One terminal row of rendered output.
#[derive(Debug, Clone)]
pub struct RenderedLine {
    pub line: Line<'static>,
    pub heading: Option<HeadingAnchor>,
    /// Clickable link ranges within this line (byte offsets into plain text).
    pub links: Vec<LinkSpan>,
}

impl RenderedLine {
    pub fn plain(line: Line<'static>) -> Self {
        Self {
            line,
            heading: None,
            links: Vec::new(),
        }
    }

    pub fn blank() -> Self {
        Self::plain(Line::default())
    }

    pub fn is_blank(&self) -> bool {
        self.line.spans.iter().all(|s| s.content.trim().is_empty())
    }

    /// Concatenated unstyled text (search, TOC labels).
    pub fn plain_text(&self) -> String {
        self.line.spans.iter().map(|s| s.content.as_ref()).collect()
    }
}

/// Rendering configuration + entry point shared by the TUI and print mode.
pub struct Renderer {
    pub theme: Theme,
    pub highlighter: Highlighter,
    pub link_mode: LinkMode,
    /// Prose column cap (readability on wide terminals). 0 = no cap.
    pub prose_cap: usize,
    /// Left margin applied to every line.
    pub margin: usize,
    /// Viewer frontend (clickable links exist). Print mode keeps output
    /// free of affordances that cannot be acted on.
    pub interactive: bool,
}

impl Renderer {
    pub fn render(&self, blocks: &[Block], width: usize) -> Vec<RenderedLine> {
        let width = width.max(20);
        let margin = if width >= 50 { self.margin } else { 0 };
        let avail = width - 2 * margin.min(width / 4);
        let prose = if self.prose_cap == 0 {
            avail
        } else {
            avail.min(self.prose_cap)
        };

        let registry = LinkRegistry::new();
        let renderer = BlockRenderer::new(
            &self.theme,
            &self.highlighter,
            self.link_mode,
            avail,
            prose,
            &registry,
            self.interactive,
        );
        let mut lines = renderer.render_document(blocks);

        if margin > 0 {
            let pad = " ".repeat(margin);
            for rl in &mut lines {
                if rl.is_blank() && rl.line.spans.is_empty() {
                    continue;
                }
                let mut spans = vec![Span::raw(pad.clone())];
                spans.extend(std::mem::take(&mut rl.line.spans));
                rl.line = Line::from(spans);
            }
        }
        // Last: byte offsets are final only after every layout pass above.
        links::extract_links(&mut lines, &registry);
        lines
    }
}
