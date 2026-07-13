//! pulldown-cmark event stream -> semantic block model.
//!
//! The event stream is flat with Start/End markers; we rebuild the tree with
//! an explicit stack of "containers under construction". This keeps the logic
//! iterative (no recursion limits from pathological nesting).

use pulldown_cmark::{
    Alignment as CmAlignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};

use super::model::{Alignment, Block, Cell, Inline, ListItem};

/// A container being built while its Start..End events stream through.
enum Frame {
    Blocks {
        kind: BlockKind,
        children: Vec<Block>,
    },
    Inlines {
        kind: InlineKind,
        children: Vec<Inline>,
    },
    List {
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    Item {
        task: Option<bool>,
        blocks: Vec<Block>,
    },
    Table {
        alignments: Vec<Alignment>,
        head: Vec<Cell>,
        rows: Vec<Vec<Cell>>,
        current_row: Vec<Cell>,
        in_head: bool,
    },
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    /// Raw HTML block: pulldown emits one Html event per source line; they
    /// accumulate here so the block renders contiguously, not one block
    /// (with blank-line separation) per line.
    HtmlBlock {
        html: String,
    },
}

enum BlockKind {
    Quote,
    FootnoteDef(String),
    /// Depth-cap sentinel: children merge into the parent container instead
    /// of nesting deeper. Bounds model depth so the recursive renderer (and
    /// the recursive `Drop` of the model itself) cannot overflow the stack
    /// on adversarial input like 20k `>` levels.
    Transparent,
}

/// Beyond this container depth the document structure flattens. Real
/// documents stay under ~10; only generated/adversarial input goes deeper.
const MAX_MODEL_DEPTH: usize = 64;

enum InlineKind {
    Paragraph,
    Heading(u8),
    Strong,
    Emph,
    Strikethrough,
    Link(String),
    Image(String),
    TableCell,
}

pub struct ParseOutput {
    pub blocks: Vec<Block>,
}

pub fn parse(source: &str) -> ParseOutput {
    // A UTF-8 BOM would otherwise render as a stray glyph in the first block.
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(source, options);
    let mut builder = Builder::new();
    for event in parser {
        builder.event(event);
    }
    ParseOutput {
        blocks: builder.finish(),
    }
}

struct Builder {
    root: Vec<Block>,
    stack: Vec<Frame>,
}

impl Builder {
    fn new() -> Self {
        Self {
            root: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<Block> {
        // Malformed input can leave unterminated frames; close them all.
        while let Some(frame) = self.stack.pop() {
            self.close_frame(frame);
        }
        self.root
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => self.push_inline(Inline::Text(t.into_string())),
            Event::Code(t) => self.push_inline(Inline::Code(t.into_string())),
            Event::Html(t) => self.push_html(t.into_string()),
            Event::InlineHtml(t) => self.push_inline(Inline::Html(t.into_string())),
            Event::SoftBreak => self.push_inline(Inline::SoftBreak),
            Event::HardBreak => self.push_inline(Inline::HardBreak),
            Event::Rule => self.push_block(Block::Rule),
            Event::TaskListMarker(checked) => {
                if let Some(Frame::Item { task, .. }) = self.stack.last_mut() {
                    *task = Some(checked);
                }
            }
            Event::FootnoteReference(label) => {
                self.push_inline(Inline::FootnoteRef(label.into_string()))
            }
            Event::InlineMath(t) | Event::DisplayMath(t) => {
                self.push_inline(Inline::Code(t.into_string()))
            }
        }
    }

    /// True when the container budget for nesting is exhausted; new
    /// containers then become transparent (children flatten into the parent).
    fn depth_exceeded(&self) -> bool {
        self.stack
            .iter()
            .filter(|f| {
                matches!(
                    f,
                    Frame::Blocks { .. } | Frame::Item { .. } | Frame::List { .. }
                )
            })
            .count()
            >= MAX_MODEL_DEPTH
    }

    fn push_container(&mut self, kind: BlockKind) {
        let kind = if self.depth_exceeded() {
            BlockKind::Transparent
        } else {
            kind
        };
        self.stack.push(Frame::Blocks {
            kind,
            children: Vec::new(),
        });
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.push_inline_frame(InlineKind::Paragraph),
            Tag::Heading { level, .. } => {
                self.push_inline_frame(InlineKind::Heading(heading_level(level)))
            }
            Tag::BlockQuote(_) => self.push_container(BlockKind::Quote),
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => {
                        // The info string may carry attributes ("rust,ignore"); keep the first token.
                        let token = info
                            .split([',', ' '])
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if token.is_empty() { None } else { Some(token) }
                    }
                    CodeBlockKind::Indented => None,
                };
                self.stack.push(Frame::CodeBlock {
                    lang,
                    code: String::new(),
                });
            }
            Tag::List(start) => {
                if self.depth_exceeded() {
                    self.push_container(BlockKind::Transparent);
                } else {
                    self.stack.push(Frame::List {
                        start,
                        items: Vec::new(),
                    });
                }
            }
            Tag::Item => {
                if self.depth_exceeded() {
                    self.push_container(BlockKind::Transparent);
                } else {
                    self.stack.push(Frame::Item {
                        task: None,
                        blocks: Vec::new(),
                    });
                }
            }
            Tag::Table(alignments) => self.stack.push(Frame::Table {
                alignments: alignments.iter().map(convert_alignment).collect(),
                head: Vec::new(),
                rows: Vec::new(),
                current_row: Vec::new(),
                in_head: false,
            }),
            Tag::TableHead => {
                if let Some(Frame::Table { in_head, .. }) = self.stack.last_mut() {
                    *in_head = true;
                }
            }
            Tag::TableRow => {}
            Tag::TableCell => self.push_inline_frame(InlineKind::TableCell),
            Tag::Emphasis => self.push_inline_frame(InlineKind::Emph),
            Tag::Strong => self.push_inline_frame(InlineKind::Strong),
            Tag::Strikethrough => self.push_inline_frame(InlineKind::Strikethrough),
            Tag::Link { dest_url, .. } => {
                self.push_inline_frame(InlineKind::Link(dest_url.into_string()))
            }
            Tag::Image { dest_url, .. } => {
                self.push_inline_frame(InlineKind::Image(dest_url.into_string()))
            }
            Tag::FootnoteDefinition(label) => {
                self.push_container(BlockKind::FootnoteDef(label.into_string()))
            }
            Tag::HtmlBlock => self.stack.push(Frame::HtmlBlock {
                html: String::new(),
            }),
            // Unsupported containers (definition lists, metadata, etc.): let
            // their inline content flow through to the parent.
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Link
            | TagEnd::Image
            | TagEnd::TableCell
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::List(_)
            | TagEnd::Item
            | TagEnd::FootnoteDefinition => {
                if let Some(frame) = self.stack.pop() {
                    self.close_frame(frame);
                }
            }
            TagEnd::TableHead => {
                if let Some(Frame::Table {
                    head,
                    current_row,
                    in_head,
                    ..
                }) = self.stack.last_mut()
                {
                    *head = std::mem::take(current_row);
                    *in_head = false;
                }
            }
            TagEnd::TableRow => {
                if let Some(Frame::Table {
                    rows, current_row, ..
                }) = self.stack.last_mut()
                {
                    rows.push(std::mem::take(current_row));
                }
            }
            TagEnd::Table => {
                if let Some(frame) = self.stack.pop() {
                    self.close_frame(frame);
                }
            }
            TagEnd::HtmlBlock => {
                if let Some(frame) = self.stack.pop() {
                    self.close_frame(frame);
                }
            }
            _ => {}
        }
    }

    fn close_frame(&mut self, frame: Frame) {
        match frame {
            Frame::Inlines { kind, children } => match kind {
                InlineKind::Paragraph => self.push_block(Block::Paragraph(children)),
                InlineKind::Heading(level) => self.push_block(Block::Heading {
                    level,
                    content: children,
                }),
                InlineKind::Strong => self.push_inline(Inline::Strong(children)),
                InlineKind::Emph => self.push_inline(Inline::Emph(children)),
                InlineKind::Strikethrough => self.push_inline(Inline::Strikethrough(children)),
                InlineKind::Link(dest) => self.push_inline(Inline::Link { dest, children }),
                InlineKind::Image(dest) => {
                    let alt = super::model::inlines_to_string(&children);
                    self.push_inline(Inline::Image { dest, alt });
                }
                InlineKind::TableCell => {
                    if let Some(Frame::Table { current_row, .. }) = self.stack.last_mut() {
                        current_row.push(children);
                    }
                }
            },
            Frame::Blocks { kind, children } => match kind {
                BlockKind::Quote => self.push_block(Block::Quote(children)),
                BlockKind::FootnoteDef(label) => self.push_block(Block::FootnoteDef {
                    label,
                    blocks: children,
                }),
                BlockKind::Transparent => {
                    for child in children {
                        self.push_block(child);
                    }
                }
            },
            Frame::List { start, items } => self.push_block(Block::List { start, items }),
            Frame::Item { task, blocks } => {
                if let Some(Frame::List { items, .. }) = self.stack.last_mut() {
                    items.push(ListItem { task, blocks });
                }
            }
            Frame::Table {
                alignments,
                head,
                rows,
                ..
            } => self.push_block(Block::Table {
                alignments,
                head,
                rows,
            }),
            Frame::CodeBlock { lang, code } => {
                let code = code.strip_suffix('\n').unwrap_or(&code).to_string();
                self.push_block(Block::CodeBlock { lang, code });
            }
            Frame::HtmlBlock { html } => {
                let trimmed = html.trim_end().to_string();
                if !trimmed.is_empty() {
                    self.push_block(Block::Html(trimmed));
                }
            }
        }
    }

    /// Add a finished block to the innermost block container (or the root).
    fn push_block(&mut self, block: Block) {
        for frame in self.stack.iter_mut().rev() {
            match frame {
                Frame::Blocks { children, .. } => {
                    children.push(block);
                    return;
                }
                Frame::Item { blocks, .. } => {
                    blocks.push(block);
                    return;
                }
                _ => continue,
            }
        }
        self.root.push(block);
    }

    /// Add a finished inline to the innermost inline container.
    fn push_inline(&mut self, inline: Inline) {
        for frame in self.stack.iter_mut().rev() {
            match frame {
                Frame::Inlines { children, .. } => {
                    children.push(inline);
                    return;
                }
                Frame::CodeBlock { code, .. } => {
                    // Indented code arrives as Text events.
                    if let Inline::Text(t) = inline {
                        code.push_str(&t);
                    }
                    return;
                }
                // "Loose" text directly inside a list item (tight lists).
                Frame::Item { blocks, .. } => {
                    if let Some(Block::Paragraph(children)) = blocks.last_mut() {
                        children.push(inline);
                    } else {
                        blocks.push(Block::Paragraph(vec![inline]));
                    }
                    return;
                }
                _ => continue,
            }
        }
        // Inline at root level (rare; e.g. stray HTML): wrap in a paragraph.
        if let Some(Block::Paragraph(children)) = self.root.last_mut() {
            children.push(inline);
        } else {
            self.root.push(Block::Paragraph(vec![inline]));
        }
    }

    fn push_html(&mut self, html: String) {
        // Code block frames may also receive Html events.
        match self.stack.last_mut() {
            Some(Frame::CodeBlock { code, .. }) => code.push_str(&html),
            Some(Frame::HtmlBlock { html: acc }) => acc.push_str(&html),
            // Html event without an HtmlBlock container (defensive).
            _ => {
                let trimmed = html.trim_end().to_string();
                if !trimmed.is_empty() {
                    self.push_block(Block::Html(trimmed));
                }
            }
        }
    }

    fn push_inline_frame(&mut self, kind: InlineKind) {
        self.stack.push(Frame::Inlines {
            kind,
            children: Vec::new(),
        });
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn convert_alignment(a: &CmAlignment) -> Alignment {
    match a {
        CmAlignment::None => Alignment::None,
        CmAlignment::Left => Alignment::Left,
        CmAlignment::Center => Alignment::Center,
        CmAlignment::Right => Alignment::Right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading_and_paragraph() {
        let out = parse("# Title\n\nHello *world*.");
        assert_eq!(out.blocks.len(), 2);
        assert!(matches!(out.blocks[0], Block::Heading { level: 1, .. }));
        assert!(matches!(out.blocks[1], Block::Paragraph(_)));
    }

    #[test]
    fn parses_table_shape() {
        let src = "| A | B |\n|---|---:|\n| 1 | 2 |\n| 3 | 4 |\n";
        let out = parse(src);
        match &out.blocks[0] {
            Block::Table {
                alignments,
                head,
                rows,
            } => {
                assert_eq!(alignments.len(), 2);
                assert_eq!(alignments[1], Alignment::Right);
                assert_eq!(head.len(), 2);
                assert_eq!(rows.len(), 2);
            }
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn parses_nested_mixed_list_with_tasks() {
        let src = "- top\n  1. inner one\n  2. inner two\n- [x] done task\n";
        let out = parse(src);
        match &out.blocks[0] {
            Block::List { start: None, items } => {
                assert_eq!(items.len(), 2);
                assert!(matches!(
                    items[0].blocks.last(),
                    Some(Block::List { start: Some(1), .. })
                ));
                assert_eq!(items[1].task, Some(true));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn code_block_keeps_language() {
        let out = parse("```rust\nfn main() {}\n```\n");
        match &out.blocks[0] {
            Block::CodeBlock { lang, code } => {
                assert_eq!(lang.as_deref(), Some("rust"));
                assert_eq!(code, "fn main() {}");
            }
            other => panic!("expected code block, got {other:?}"),
        }
    }

    #[test]
    fn survives_malformed_input() {
        // Unclosed emphasis, stray table pipes, lone fence: must not panic.
        let out = parse("**unclosed\n\n| a |\n\n```\nno close");
        assert!(!out.blocks.is_empty());
    }

    #[test]
    fn html_block_stays_contiguous() {
        // pulldown emits one Html event per line; they must merge into a
        // single block or rendering double-spaces the markup.
        let out = parse("<div class=\"x\">\n  <span>hi</span>\n</div>\n");
        let htmls: Vec<&Block> = out
            .blocks
            .iter()
            .filter(|b| matches!(b, Block::Html(_)))
            .collect();
        assert_eq!(htmls.len(), 1, "expected one merged html block");
        if let Block::Html(text) = htmls[0] {
            assert!(text.contains("  <span>"), "indentation lost: {text:?}");
        }
    }

    #[test]
    fn model_depth_is_bounded() {
        // 20k quote levels must produce a model whose nesting depth is
        // capped, or the recursive renderer / Drop would blow the stack.
        let src = format!("{}deep", "> ".repeat(20_000));
        let out = parse(&src);
        let mut depth = 0usize;
        let mut block = out.blocks.first();
        while let Some(Block::Quote(children)) = block {
            depth += 1;
            block = children.first();
            assert!(depth <= MAX_MODEL_DEPTH + 1, "quote nesting not capped");
        }
        // Content must survive the flattening.
        fn contains_text(blocks: &[Block], needle: &str) -> bool {
            blocks.iter().any(|b| match b {
                Block::Paragraph(inlines) => {
                    crate::markdown::model::inlines_to_string(inlines).contains(needle)
                }
                Block::Quote(c) => contains_text(c, needle),
                _ => false,
            })
        }
        assert!(contains_text(&out.blocks, "deep"));
    }
}
