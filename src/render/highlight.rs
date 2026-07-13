//! Fenced code block syntax highlighting via syntect.
//!
//! Syntect emits RGB colors; on terminals without truecolor we quantize to
//! the xterm-256 palette so colors stay sane instead of garbled.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

pub struct Highlighter {
    /// Loaded lazily: syntax/theme dumps cost tens of milliseconds and
    /// several MB, wasted entirely in `--no-highlight` or colorless runs.
    assets: std::cell::OnceCell<(SyntaxSet, ThemeSet)>,
    pub enabled: bool,
    pub truecolor: bool,
    /// Highlighting is width-independent, so results survive terminal
    /// resizes; this cache makes re-renders (resize, toggles) nearly free
    /// for code-heavy documents.
    cache: RefCell<HashMap<u64, Vec<Vec<Span<'static>>>>>,
}

impl Highlighter {
    pub fn new(enabled: bool, truecolor: bool) -> Self {
        Self {
            assets: std::cell::OnceCell::new(),
            enabled,
            truecolor,
            cache: RefCell::new(HashMap::new()),
        }
    }

    fn assets(&self) -> &(SyntaxSet, ThemeSet) {
        self.assets.get_or_init(|| {
            (
                SyntaxSet::load_defaults_newlines(),
                ThemeSet::load_defaults(),
            )
        })
    }

    /// Highlight `code`, returning one span vector per source line.
    /// Falls back to `fallback` styling when the language is unknown,
    /// highlighting is disabled, or the syntect theme is missing.
    pub fn highlight(
        &self,
        code: &str,
        lang: Option<&str>,
        theme_name: &str,
        fallback: Style,
    ) -> Vec<Vec<Span<'static>>> {
        let plain = || {
            code.split('\n')
                .map(|l| vec![Span::styled(expand_tabs(l), fallback)])
                .collect()
        };

        if !self.enabled || theme_name.is_empty() {
            return plain();
        }
        let Some(lang) = lang else { return plain() };
        let (syntaxes, themes) = self.assets();
        let Some(syntax) = syntaxes
            .find_syntax_by_token(lang)
            .or_else(|| syntaxes.find_syntax_by_extension(lang))
        else {
            return plain();
        };
        let Some(theme) = themes.themes.get(theme_name) else {
            return plain();
        };

        let key = cache_key(code, lang, theme_name);
        if let Some(hit) = self.cache.borrow().get(&key) {
            return hit.clone();
        }

        let mut hl = HighlightLines::new(syntax, theme);
        let mut out = Vec::new();
        for line in code.split('\n') {
            // Expand tabs to real tab stops BEFORE highlighting so columns
            // survive; syntect regions then never contain tabs.
            let line = expand_tabs(line);
            // syntect wants the trailing newline for correct state tracking.
            let with_nl = format!("{line}\n");
            match hl.highlight_line(&with_nl, syntaxes) {
                Ok(regions) => {
                    let mut spans = Vec::new();
                    for (style, text) in regions {
                        let text = text.trim_end_matches('\n');
                        if text.is_empty() {
                            continue;
                        }
                        spans.push(Span::styled(
                            text.to_string(),
                            convert_style(style, fallback, self.truecolor),
                        ));
                    }
                    out.push(spans);
                }
                Err(_) => out.push(vec![Span::styled(line.clone(), fallback)]),
            }
        }
        self.cache.borrow_mut().insert(key, out.clone());
        out
    }
}

fn cache_key(code: &str, lang: &str, theme_name: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut h);
    lang.hash(&mut h);
    theme_name.hash(&mut h);
    h.finish()
}

/// Expand tabs to 4-column tab *stops* (column-aware, not a blind 4-space
/// substitution), so tab-aligned code keeps its alignment.
fn expand_tabs(s: &str) -> String {
    if !s.contains('\t') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 8);
    let mut col = 0usize;
    for ch in s.chars() {
        if ch == '\t' {
            let pad = 4 - (col % 4);
            out.extend(std::iter::repeat_n(' ', pad));
            col += pad;
        } else {
            out.push(ch);
            col += crate::render::wrap::display_width(&ch.to_string());
        }
    }
    out
}

fn convert_style(s: syntect::highlighting::Style, base: Style, truecolor: bool) -> Style {
    let fg = s.foreground;
    let color = if truecolor {
        Color::Rgb(fg.r, fg.g, fg.b)
    } else {
        Color::Indexed(quantize_256(fg.r, fg.g, fg.b))
    };
    let mut style = base.fg(color);
    if s.font_style.contains(FontStyle::BOLD) {
        style = style.add_modifier(Modifier::BOLD);
    }
    if s.font_style.contains(FontStyle::ITALIC) {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if s.font_style.contains(FontStyle::UNDERLINE) {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

/// Map RGB to the closest xterm-256 palette index (6x6x6 cube + grayscale ramp).
pub fn quantize_256(r: u8, g: u8, b: u8) -> u8 {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    // Candidate from the grayscale ramp (232..=255).
    let gray_avg = (r + g + b) / 3;
    let gray_idx = ((gray_avg - 8).max(0) / 10).min(23);
    let gray_val = 8 + gray_idx * 10;
    // Candidate from the 6x6x6 color cube (16..=231).
    let to_cube = |v: i32| -> i32 {
        if v < 48 {
            0
        } else if v < 114 {
            1
        } else {
            (v - 35) / 40
        }
    };
    let (cr, cg, cb) = (to_cube(r), to_cube(g), to_cube(b));
    let cube_val = |c: i32| if c == 0 { 0 } else { c * 40 + 55 };
    let (vr, vg, vb) = (cube_val(cr), cube_val(cg), cube_val(cb));

    let dist_cube = (r - vr).pow(2) + (g - vg).pow(2) + (b - vb).pow(2);
    let dist_gray = (r - gray_val).pow(2) + (g - gray_val).pow(2) + (b - gray_val).pow(2);

    if dist_gray < dist_cube {
        (232 + gray_idx) as u8
    } else {
        (16 + 36 * cr + 6 * cg + cb) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_edges() {
        assert_eq!(quantize_256(0, 0, 0), 16); // cube black
        assert_eq!(quantize_256(255, 255, 255), 231); // cube white
        assert_eq!(quantize_256(128, 128, 128), 244); // mid gray -> ramp
    }

    #[test]
    fn unknown_language_falls_back_to_plain() {
        let hl = Highlighter::new(true, true);
        let lines = hl.highlight(
            "x = 1",
            Some("nosuchlang"),
            "base16-ocean.dark",
            Style::default(),
        );
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0][0].content.as_ref(), "x = 1");
    }

    #[test]
    fn rust_gets_colored() {
        let hl = Highlighter::new(true, true);
        let lines = hl.highlight(
            "fn main() {}",
            Some("rs"),
            "base16-ocean.dark",
            Style::default(),
        );
        assert!(lines[0].len() > 1, "expected multiple colored regions");
    }
}
