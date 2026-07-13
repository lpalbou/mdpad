//! Semantic document model.
//!
//! The parser converts raw markdown into this tree; the renderer consumes it.
//! Keeping a semantic model (instead of rendering during parsing) is what
//! allows exact re-layout on terminal resize and precise TOC/search targets.

/// Column alignment for tables, mirroring GFM semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

/// Inline (span-level) content.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    /// Inline code span: `code`
    Code(String),
    Strong(Vec<Inline>),
    Emph(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Link {
        dest: String,
        children: Vec<Inline>,
    },
    Image {
        dest: String,
        alt: String,
    },
    /// Raw inline HTML, rendered verbatim and dimmed.
    Html(String),
    /// Footnote reference like `[^1]`.
    FootnoteRef(String),
    SoftBreak,
    HardBreak,
}

/// One list item; `task` is `Some(checked)` for GFM task-list items.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub task: Option<bool>,
    pub blocks: Vec<Block>,
}

/// A table cell is a sequence of inlines.
pub type Cell = Vec<Inline>;

/// Block (structural) content.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)] // `CodeBlock` is the standard CommonMark term
pub enum Block {
    Heading {
        /// 1..=6
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    List {
        /// `Some(n)` for ordered lists starting at n, `None` for bullet lists.
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    CodeBlock {
        /// Language token from the fence, if any (e.g. "rust").
        lang: Option<String>,
        code: String,
    },
    Quote(Vec<Block>),
    Table {
        alignments: Vec<Alignment>,
        head: Vec<Cell>,
        rows: Vec<Vec<Cell>>,
    },
    /// Thematic break (---).
    Rule,
    /// Raw HTML block, rendered verbatim and dimmed.
    Html(String),
    FootnoteDef {
        label: String,
        blocks: Vec<Block>,
    },
}

/// Flatten inlines to plain text (used for image alt text, TOC labels, search).
pub fn inlines_to_string(inlines: &[Inline]) -> String {
    let mut out = String::new();
    push_plain(inlines, &mut out);
    out
}

fn push_plain(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(t) | Inline::Code(t) | Inline::Html(t) => out.push_str(t),
            Inline::Strong(c) | Inline::Emph(c) | Inline::Strikethrough(c) => push_plain(c, out),
            Inline::Link { children, dest } => {
                if children.is_empty() {
                    out.push_str(dest);
                } else {
                    push_plain(children, out);
                }
            }
            Inline::Image { alt, .. } => out.push_str(alt),
            Inline::FootnoteRef(label) => {
                out.push_str("[^");
                out.push_str(label);
                out.push(']');
            }
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
        }
    }
}
