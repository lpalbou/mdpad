//! Visual identity: colors, glyphs, spacing rules.
//!
//! All colors use the 256-color indexed palette for broad terminal support;
//! only syntax highlighting produces RGB (quantized elsewhere when the
//! terminal lacks truecolor).

use ratatui::style::{Color, Modifier, Style};

/// Glyph set; `ascii()` exists so the tool degrades gracefully on terminals
/// or fonts without box-drawing/unicode symbols (`--ascii`).
#[derive(Debug, Clone)]
pub struct CharSet {
    pub bullets: [&'static str; 3],
    pub quote_bar: &'static str,
    pub task_unchecked: &'static str,
    pub task_checked: &'static str,
    pub rule: &'static str,
    pub h1_underline: &'static str,
    pub h2_underline: &'static str,
    pub image_marker: &'static str,
    // Table borders
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub h: &'static str,
    pub v: &'static str,
    pub cross: &'static str,
    pub t_down: &'static str,
    pub t_up: &'static str,
    pub t_left: &'static str,
    pub t_right: &'static str,
}

impl CharSet {
    pub fn unicode() -> Self {
        Self {
            bullets: ["•", "◦", "▪"],
            quote_bar: "▎",
            task_unchecked: "☐",
            // U+2714: width 1 everywhere. Emoji-block glyphs (🗹) lie about
            // their width in many fonts and shift table/list alignment.
            task_checked: "✔",
            rule: "─",
            h1_underline: "━",
            h2_underline: "─",
            image_marker: "▨",
            tl: "╭",
            tr: "╮",
            bl: "╰",
            br: "╯",
            h: "─",
            v: "│",
            cross: "┼",
            t_down: "┬",
            t_up: "┴",
            t_left: "┤",
            t_right: "├",
        }
    }

    pub fn ascii() -> Self {
        Self {
            bullets: ["*", "-", "+"],
            quote_bar: "|",
            task_unchecked: "[ ]",
            task_checked: "[x]",
            rule: "-",
            h1_underline: "=",
            h2_underline: "-",
            image_marker: "img:",
            tl: "+",
            tr: "+",
            bl: "+",
            br: "+",
            h: "-",
            v: "|",
            cross: "+",
            t_down: "+",
            t_up: "+",
            t_left: "+",
            t_right: "+",
        }
    }
}

/// Complete style sheet for rendering.
#[derive(Debug, Clone)]
pub struct Theme {
    pub headings: [Style; 6],
    pub heading_underline: Style,
    pub text: Style,
    pub strong: Style,
    pub emph: Style,
    pub strike: Style,
    pub inline_code: Style,
    pub code_text: Style,
    pub code_bg: Color,
    pub code_lang: Style,
    pub quote_bar: Style,
    pub bullet: Style,
    pub ordered: Style,
    pub task_done: Style,
    pub task_open: Style,
    pub link: Style,
    pub link_url: Style,
    pub image: Style,
    pub table_border: Style,
    pub table_header: Style,
    pub rule: Style,
    pub html: Style,
    pub footnote: Style,
    /// syntect theme name to pair with this UI theme.
    pub syntax_theme: &'static str,
    /// Re-emit backticks around inline code. Colorless output has no other
    /// way to distinguish code from prose.
    pub code_delims: bool,
    pub chars: CharSet,
}

fn idx(i: u8) -> Color {
    Color::Indexed(i)
}

impl Theme {
    pub fn dark(chars: CharSet) -> Self {
        let accent = idx(75); // soft blue
        Self {
            headings: [
                Style::default().fg(idx(75)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(111)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(147)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(152)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(250)).add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(idx(245))
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            ],
            heading_underline: Style::default().fg(idx(60)),
            text: Style::default(),
            strong: Style::default().add_modifier(Modifier::BOLD),
            emph: Style::default().add_modifier(Modifier::ITALIC),
            strike: Style::default()
                .fg(idx(245))
                .add_modifier(Modifier::CROSSED_OUT),
            inline_code: Style::default().fg(idx(209)).bg(idx(236)),
            code_text: Style::default().fg(idx(252)),
            code_bg: idx(235),
            code_lang: Style::default().fg(idx(245)).add_modifier(Modifier::ITALIC),
            quote_bar: Style::default().fg(idx(71)),
            bullet: Style::default().fg(accent),
            ordered: Style::default().fg(accent),
            task_done: Style::default().fg(idx(71)),
            task_open: Style::default().fg(idx(245)),
            link: Style::default()
                .fg(idx(110))
                .add_modifier(Modifier::UNDERLINED),
            link_url: Style::default().fg(idx(245)),
            image: Style::default().fg(idx(245)),
            table_border: Style::default().fg(idx(240)),
            table_header: Style::default().fg(idx(111)).add_modifier(Modifier::BOLD),
            rule: Style::default().fg(idx(240)),
            html: Style::default().fg(idx(245)),
            footnote: Style::default().fg(idx(141)),
            syntax_theme: "base16-ocean.dark",
            code_delims: false,
            chars,
        }
    }

    pub fn light(chars: CharSet) -> Self {
        let accent = idx(26); // deep blue
        Self {
            headings: [
                Style::default().fg(idx(26)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(25)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(61)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(60)).add_modifier(Modifier::BOLD),
                Style::default().fg(idx(238)).add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(idx(243))
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            ],
            heading_underline: Style::default().fg(idx(146)),
            text: Style::default(),
            strong: Style::default().add_modifier(Modifier::BOLD),
            emph: Style::default().add_modifier(Modifier::ITALIC),
            strike: Style::default()
                .fg(idx(243))
                .add_modifier(Modifier::CROSSED_OUT),
            inline_code: Style::default().fg(idx(124)).bg(idx(254)),
            code_text: Style::default().fg(idx(235)),
            code_bg: idx(255),
            code_lang: Style::default().fg(idx(243)).add_modifier(Modifier::ITALIC),
            quote_bar: Style::default().fg(idx(28)),
            bullet: Style::default().fg(accent),
            ordered: Style::default().fg(accent),
            task_done: Style::default().fg(idx(28)),
            task_open: Style::default().fg(idx(243)),
            link: Style::default()
                .fg(idx(25))
                .add_modifier(Modifier::UNDERLINED),
            link_url: Style::default().fg(idx(243)),
            image: Style::default().fg(idx(243)),
            table_border: Style::default().fg(idx(249)),
            table_header: Style::default().fg(idx(25)).add_modifier(Modifier::BOLD),
            rule: Style::default().fg(idx(249)),
            html: Style::default().fg(idx(243)),
            footnote: Style::default().fg(idx(97)),
            syntax_theme: "InspiredGitHub",
            code_delims: false,
            chars,
        }
    }

    /// Colorless theme: layout and typography only. Used for NO_COLOR and
    /// `--no-color`; bold/italic survive because they are not colors.
    pub fn plain(chars: CharSet) -> Self {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let italic = Style::default().add_modifier(Modifier::ITALIC);
        let none = Style::default();
        Self {
            headings: [
                bold,
                bold,
                bold,
                bold,
                bold.add_modifier(Modifier::ITALIC),
                italic,
            ],
            heading_underline: none,
            text: none,
            strong: bold,
            emph: italic,
            strike: Style::default().add_modifier(Modifier::CROSSED_OUT),
            inline_code: none,
            code_text: none,
            code_bg: Color::Reset,
            code_lang: italic,
            quote_bar: none,
            bullet: none,
            ordered: none,
            task_done: none,
            task_open: none,
            link: Style::default().add_modifier(Modifier::UNDERLINED),
            link_url: none,
            image: none,
            table_border: none,
            table_header: bold,
            rule: none,
            html: none,
            footnote: none,
            syntax_theme: "",
            code_delims: true,
            chars,
        }
    }
}
