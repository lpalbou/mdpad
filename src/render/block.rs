//! Block layout: turns the semantic model into pre-wrapped visual lines.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::markdown::model::{Block, Inline, ListItem, inlines_to_string};
use crate::render::highlight::Highlighter;
use crate::render::inline::{InlineRenderer, LinkMode};
use crate::render::table::TableRenderer;
use crate::render::theme::Theme;
use crate::render::wrap::{display_width, spans_width, wrap_spans};
use crate::render::{HeadingAnchor, RenderedLine};

/// Layout context threaded through nested blocks.
#[derive(Clone, Copy)]
struct Ctx {
    /// Columns available at this nesting level.
    width: usize,
    /// List nesting depth (drives bullet glyph rotation).
    list_depth: usize,
}

pub struct BlockRenderer<'a> {
    theme: &'a Theme,
    highlighter: &'a Highlighter,
    inline: InlineRenderer<'a>,
    /// Prose is capped for readability; tables may use the full width.
    prose_width: usize,
    full_width: usize,
}

impl<'a> BlockRenderer<'a> {
    pub fn new(
        theme: &'a Theme,
        highlighter: &'a Highlighter,
        link_mode: LinkMode,
        full_width: usize,
        prose_width: usize,
    ) -> Self {
        Self {
            theme,
            highlighter,
            inline: InlineRenderer::new(theme, link_mode),
            prose_width: prose_width.max(10),
            full_width: full_width.max(10),
        }
    }

    pub fn render_document(&self, blocks: &[Block]) -> Vec<RenderedLine> {
        let ctx = Ctx {
            width: self.prose_width,
            list_depth: 0,
        };
        let mut out = Vec::new();
        self.render_blocks(blocks, ctx, &mut out);
        // Trim leading/trailing blanks left by block separation.
        let lead = out.iter().position(|l| !l.is_blank()).unwrap_or(out.len());
        out.drain(..lead);
        while out.last().is_some_and(RenderedLine::is_blank) {
            out.pop();
        }
        out
    }

    fn render_blocks(&self, blocks: &[Block], ctx: Ctx, out: &mut Vec<RenderedLine>) {
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                out.push(RenderedLine::blank());
            }
            self.render_block(block, ctx, out);
        }
    }

    fn render_block(&self, block: &Block, ctx: Ctx, out: &mut Vec<RenderedLine>) {
        match block {
            Block::Heading { level, content } => self.heading(*level, content, ctx, out),
            Block::Paragraph(inlines) => self.paragraph(inlines, Style::default(), ctx, out),
            Block::List { start, items } => self.list(*start, items, ctx, out),
            Block::CodeBlock { lang, code } => self.code(lang.as_deref(), code, ctx, out),
            Block::Quote(children) => self.quote(children, ctx, out),
            Block::Table {
                alignments,
                head,
                rows,
            } => {
                // Tables are the space-hungry element: they may exceed the
                // prose cap and use the width the prose gave up, adjusted
                // for the current nesting indentation.
                let extra = self.full_width.saturating_sub(self.prose_width);
                let width = ctx.width + extra;
                let tr = TableRenderer::new(self.theme);
                for line in tr.render(alignments, head, rows, width) {
                    out.push(RenderedLine::plain(line));
                }
            }
            Block::Rule => {
                let w = ctx.width.min(self.prose_width);
                out.push(RenderedLine::plain(Line::from(Span::styled(
                    self.theme.chars.rule.repeat(w),
                    self.theme.rule,
                ))));
            }
            Block::Html(raw) => {
                // Verbatim means verbatim: hard-wrap without collapsing the
                // indentation that word wrap would eat.
                for l in raw.lines() {
                    for chunk in
                        hard_chunks(&[Span::styled(l.to_string(), self.theme.html)], ctx.width)
                    {
                        out.push(RenderedLine::plain(Line::from(chunk)));
                    }
                }
            }
            Block::FootnoteDef { label, blocks } => {
                let marker = Span::styled(format!("[^{label}] "), self.theme.footnote);
                let indent = display_width(&marker.content);
                let mut inner = Vec::new();
                let inner_ctx = Ctx {
                    width: ctx.width.saturating_sub(indent),
                    ..ctx
                };
                self.render_blocks(blocks, inner_ctx, &mut inner);
                prefix_lines(inner, marker, " ".repeat(indent), Style::default(), out);
            }
        }
    }

    fn heading(&self, level: u8, content: &[Inline], ctx: Ctx, out: &mut Vec<RenderedLine>) {
        let idx = (level.clamp(1, 6) - 1) as usize;
        let style = self.theme.headings[idx];
        let anchor = HeadingAnchor {
            level,
            text: inlines_to_string(content),
        };

        // H3+ carry a dim hash prefix so the level is always legible.
        let prefix = if level >= 3 {
            Some(Span::styled(
                format!("{} ", "#".repeat(level as usize)),
                self.theme.heading_underline,
            ))
        } else {
            None
        };
        let prefix_width = prefix.as_ref().map_or(0, |p| display_width(&p.content));

        let spans = self.inline.render(content, style);
        let wrapped = wrap_spans(&spans, ctx.width.saturating_sub(prefix_width));
        let mut max_w = 0usize;
        let mut first = true;
        for mut line_spans in wrapped {
            if let Some(p) = &prefix {
                if first {
                    line_spans.insert(0, p.clone());
                } else {
                    // Continuation lines align under the text, not under the
                    // hashes — repeating the prefix would read as a new
                    // heading per wrapped line.
                    line_spans.insert(0, Span::raw(" ".repeat(prefix_width)));
                }
            }
            max_w = max_w.max(spans_width(&line_spans));
            let mut rl = RenderedLine::plain(Line::from(line_spans));
            if first {
                rl.heading = Some(anchor.clone());
                first = false;
            }
            out.push(rl);
        }
        // H1/H2 get an underline sized to the text (not the full width): it
        // reads as typography rather than a horizontal rule.
        let underline_char = match level {
            1 => Some(self.theme.chars.h1_underline),
            2 => Some(self.theme.chars.h2_underline),
            _ => None,
        };
        if let Some(ch) = underline_char {
            let w = max_w.clamp(1, ctx.width);
            out.push(RenderedLine::plain(Line::from(Span::styled(
                ch.repeat(w),
                self.theme.heading_underline,
            ))));
        }
    }

    fn paragraph(&self, inlines: &[Inline], base: Style, ctx: Ctx, out: &mut Vec<RenderedLine>) {
        let spans = self.inline.render(inlines, base);
        // Hard breaks arrive as "\n" spans: wrap each segment independently.
        for segment in split_hard_breaks(spans) {
            for line_spans in wrap_spans(&segment, ctx.width) {
                out.push(RenderedLine::plain(Line::from(line_spans)));
            }
        }
    }

    fn list(&self, start: Option<u64>, items: &[ListItem], ctx: Ctx, out: &mut Vec<RenderedLine>) {
        let ch = &self.theme.chars;
        // Ordered markers right-align to the widest number ("99." / "100.").
        let num_width = start.map(|s| {
            let last = s + items.len().saturating_sub(1) as u64;
            format!("{last}.").len().max(2)
        });

        // Markers compose: an ordered task item keeps BOTH its number and
        // its checkbox ("2. ✔ ship it"). Build every marker first, then pad
        // all of them to one field width so mixed lists stay left-aligned
        // (matters for --ascii where "[ ]" is wider than "*").
        let markers: Vec<(String, ratatui::style::Style)> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let task_glyph = item.task.map(|done| {
                    if done {
                        ch.task_checked
                    } else {
                        ch.task_unchecked
                    }
                });
                let task_style = match item.task {
                    Some(true) => self.theme.task_done,
                    _ => self.theme.task_open,
                };
                match (num_width, task_glyph) {
                    (Some(w), Some(glyph)) => (
                        format!("{:>w$} {glyph}", format!("{}.", start.unwrap() + i as u64)),
                        task_style,
                    ),
                    (Some(w), None) => (
                        format!("{:>w$}", format!("{}.", start.unwrap() + i as u64)),
                        self.theme.ordered,
                    ),
                    (None, Some(glyph)) => (glyph.to_string(), task_style),
                    (None, None) => (
                        ch.bullets[ctx.list_depth % 3].to_string(),
                        self.theme.bullet,
                    ),
                }
            })
            .collect();
        let field = markers
            .iter()
            .map(|(m, _)| display_width(m))
            .max()
            .unwrap_or(1);

        // A list is "loose" if any item holds multiple blocks; loose lists get
        // blank lines between items, tight ones don't.
        let loose = items.iter().any(|it| it.blocks.len() > 1);

        for (i, item) in items.iter().enumerate() {
            if i > 0 && loose {
                out.push(RenderedLine::blank());
            }
            let (marker, marker_style) = &markers[i];
            let pad = field - display_width(marker);
            let marker_text = format!("{marker}{} ", " ".repeat(pad));
            let indent = display_width(&marker_text);
            let inner_ctx = Ctx {
                width: ctx.width.saturating_sub(indent),
                list_depth: ctx.list_depth + 1,
            };
            let mut inner = Vec::new();
            self.render_blocks(&item.blocks, inner_ctx, &mut inner);
            if inner.is_empty() {
                inner.push(RenderedLine::blank());
            }
            prefix_lines(
                inner,
                Span::styled(marker_text, *marker_style),
                " ".repeat(indent),
                Style::default(),
                out,
            );
        }
    }

    fn code(&self, lang: Option<&str>, code: &str, ctx: Ctx, out: &mut Vec<RenderedLine>) {
        let bg = self.theme.code_bg;
        let content_width = ctx.width;
        // Padding shrinks before code does: at very narrow widths the code
        // must still fit inside the context, not overflow it.
        let pad = if content_width >= 24 { 2 } else { 0 };
        let inner_width = content_width.saturating_sub(pad * 2).max(1);

        let bgify = |mut s: Style| -> Style {
            s.bg = Some(bg);
            s
        };

        // Language label line (only when we know the language). Exotic long
        // info strings must not break the width invariant.
        if let Some(lang) = lang {
            let mut label = format!(" {lang}");
            if display_width(&label) > content_width {
                label = truncate_to_width(&label, content_width);
            }
            let mut spans = vec![Span::styled(label, bgify(self.theme.code_lang))];
            fill_line(&mut spans, content_width, bgify(Style::default()));
            out.push(RenderedLine::plain(Line::from(spans)));
        }

        let highlighted =
            self.highlighter
                .highlight(code, lang, self.theme.syntax_theme, self.theme.code_text);
        for line in highlighted {
            // Hard-wrap long code lines at the inner width (no word wrap: code).
            let chunks = hard_chunks(&line, inner_width);
            for chunk in chunks {
                let mut spans = vec![Span::styled(" ".repeat(pad), bgify(Style::default()))];
                spans.extend(
                    chunk
                        .into_iter()
                        .map(|s| Span::styled(s.content.into_owned(), bgify(s.style))),
                );
                fill_line(&mut spans, content_width, bgify(Style::default()));
                out.push(RenderedLine::plain(Line::from(spans)));
            }
        }
    }

    fn quote(&self, children: &[Block], ctx: Ctx, out: &mut Vec<RenderedLine>) {
        let bar = format!("{} ", self.theme.chars.quote_bar);
        let indent = display_width(&bar);
        let inner_ctx = Ctx {
            width: ctx.width.saturating_sub(indent),
            ..ctx
        };
        let mut inner = Vec::new();
        self.render_blocks(children, inner_ctx, &mut inner);
        // Every line (including blanks) carries the bar: keeps the quote visually glued.
        for mut rl in inner {
            let mut spans = vec![Span::styled(bar.clone(), self.theme.quote_bar)];
            spans.extend(rl.line.spans);
            rl.line = Line::from(spans);
            out.push(rl);
        }
    }
}

/// Prefix the first line with `marker` and continuation lines with `cont`.
fn prefix_lines(
    lines: Vec<RenderedLine>,
    marker: Span<'static>,
    cont: String,
    cont_style: Style,
    out: &mut Vec<RenderedLine>,
) {
    for (i, mut rl) in lines.into_iter().enumerate() {
        let lead = if i == 0 {
            marker.clone()
        } else if rl.is_blank() {
            // Keep blank separators truly blank (no trailing spaces).
            out.push(rl);
            continue;
        } else {
            Span::styled(cont.clone(), cont_style)
        };
        let mut spans = vec![lead];
        spans.extend(rl.line.spans);
        rl.line = Line::from(spans);
        out.push(rl);
    }
}

/// Split spans at hard-break markers ("\n" content).
fn split_hard_breaks(spans: Vec<Span<'static>>) -> Vec<Vec<Span<'static>>> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    for span in spans {
        if span.content.as_ref() == "\n" {
            out.push(std::mem::take(&mut current));
        } else {
            current.push(span);
        }
    }
    out.push(current);
    out
}

/// Pad a styled line with a background-colored filler to `width`.
fn fill_line(spans: &mut Vec<Span<'static>>, width: usize, fill_style: Style) {
    let used = spans_width(spans);
    if used < width {
        spans.push(Span::styled(" ".repeat(width - used), fill_style));
    }
}

/// Break spans into display-width-bounded chunks without word awareness
/// (code must not reflow words, only fold overlong lines).
fn hard_chunks(spans: &[Span<'static>], width: usize) -> Vec<Vec<Span<'static>>> {
    use unicode_segmentation::UnicodeSegmentation;
    let width = width.max(1);
    let mut chunks = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;
    for span in spans {
        let mut buf = String::new();
        for g in span.content.graphemes(true) {
            let gw = display_width(g);
            if used + gw > width {
                if !buf.is_empty() {
                    current.push(Span::styled(std::mem::take(&mut buf), span.style));
                }
                chunks.push(std::mem::take(&mut current));
                used = 0;
            }
            buf.push_str(g);
            used += gw;
        }
        if !buf.is_empty() {
            current.push(Span::styled(buf, span.style));
        }
    }
    chunks.push(current);
    chunks
}

/// Truncate a string to `width` display columns, ending with an ellipsis.
fn truncate_to_width(s: &str, width: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    let budget = width.saturating_sub(1);
    let mut out = String::new();
    let mut used = 0usize;
    for g in s.graphemes(true) {
        let w = display_width(g);
        if used + w > budget {
            break;
        }
        used += w;
        out.push_str(g);
    }
    out.push('…');
    out
}
