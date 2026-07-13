//! Rendered lines -> ANSI escape text for print mode (pipes, testing, CI).

use ratatui::style::{Color, Modifier, Style};

use crate::render::RenderedLine;

/// Serialize rendered lines to a string with ANSI escapes.
/// With `styled == false` the output is pure text (NO_COLOR / --no-color).
pub fn to_ansi(lines: &[RenderedLine], styled: bool) -> String {
    let mut out = String::new();
    for rl in lines {
        if styled {
            let mut current = String::new();
            for span in &rl.line.spans {
                let code = style_code(span.style);
                if code != current {
                    out.push_str("\x1b[0m");
                    out.push_str(&code);
                    current = code;
                }
                out.push_str(&span.content);
            }
            if !current.is_empty() {
                out.push_str("\x1b[0m");
            }
            // Trailing whitespace is invisible but pollutes copy-paste.
            while out.ends_with(' ') {
                out.pop();
            }
        } else {
            let text = rl.plain_text();
            out.push_str(text.trim_end());
        }
        out.push('\n');
    }
    out
}

/// Build the SGR sequence for a style (empty string = default style).
fn style_code(style: Style) -> String {
    let mut params: Vec<String> = Vec::new();
    let m = style.add_modifier;
    if m.contains(Modifier::BOLD) {
        params.push("1".into());
    }
    if m.contains(Modifier::DIM) {
        params.push("2".into());
    }
    if m.contains(Modifier::ITALIC) {
        params.push("3".into());
    }
    if m.contains(Modifier::UNDERLINED) {
        params.push("4".into());
    }
    if m.contains(Modifier::REVERSED) {
        params.push("7".into());
    }
    if m.contains(Modifier::CROSSED_OUT) {
        params.push("9".into());
    }
    if let Some(fg) = style.fg {
        push_color(&mut params, fg, false);
    }
    if let Some(bg) = style.bg {
        push_color(&mut params, bg, true);
    }
    if params.is_empty() {
        String::new()
    } else {
        format!("\x1b[{}m", params.join(";"))
    }
}

fn push_color(params: &mut Vec<String>, color: Color, background: bool) {
    let base = if background { 40 } else { 30 };
    let ext = if background { 48 } else { 38 };
    match color {
        Color::Reset => {}
        Color::Black => params.push(format!("{base}")),
        Color::Red => params.push(format!("{}", base + 1)),
        Color::Green => params.push(format!("{}", base + 2)),
        Color::Yellow => params.push(format!("{}", base + 3)),
        Color::Blue => params.push(format!("{}", base + 4)),
        Color::Magenta => params.push(format!("{}", base + 5)),
        Color::Cyan => params.push(format!("{}", base + 6)),
        Color::Gray => params.push(format!("{}", base + 7)),
        Color::DarkGray => params.push(format!("{}", base + 60)),
        Color::LightRed => params.push(format!("{}", base + 61)),
        Color::LightGreen => params.push(format!("{}", base + 62)),
        Color::LightYellow => params.push(format!("{}", base + 63)),
        Color::LightBlue => params.push(format!("{}", base + 64)),
        Color::LightMagenta => params.push(format!("{}", base + 65)),
        Color::LightCyan => params.push(format!("{}", base + 66)),
        Color::White => params.push(format!("{}", base + 67)),
        Color::Indexed(i) => params.push(format!("{ext};5;{i}")),
        Color::Rgb(r, g, b) => params.push(format!("{ext};2;{r};{g};{b}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::{Line, Span};

    #[test]
    fn plain_mode_has_no_escapes() {
        let lines = vec![RenderedLine::plain(Line::from(Span::styled(
            "hello",
            Style::default().fg(Color::Indexed(75)),
        )))];
        let out = to_ansi(&lines, false);
        assert_eq!(out, "hello\n");
    }

    #[test]
    fn styled_mode_emits_and_resets() {
        let lines = vec![RenderedLine::plain(Line::from(Span::styled(
            "hi",
            Style::default().fg(Color::Indexed(75)),
        )))];
        let out = to_ansi(&lines, true);
        assert!(out.contains("\x1b[38;5;75m"));
        assert!(out.ends_with("\x1b[0m\n"));
    }
}
