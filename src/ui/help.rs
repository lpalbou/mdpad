//! Help overlay: the complete key reference.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};

use crate::ui::toc::centered;

const KEYS: &[(&str, &str)] = &[
    ("j / k, ↓ / ↑", "scroll line"),
    ("Space / b, PgDn / PgUp", "scroll page"),
    ("d / u", "scroll half page"),
    ("g / G", "go to top / bottom"),
    ("/", "search  (n / N: next / previous match)"),
    ("Esc", "clear search + selection / close overlay"),
    ("t", "table of contents"),
    ("L", "toggle link URLs"),
    ("mouse drag", "select + copy to clipboard"),
    ("Ctrl+C", "copy selection again (without one: quit)"),
    ("m", "toggle mouse (off = terminal-native selection)"),
    ("e", "edit in built-in editor"),
    ("E", "edit in $EDITOR"),
    ("r", "reload file from disk"),
    ("?", "this help"),
    ("q", "quit"),
    ("", ""),
    ("— editor —", ""),
    ("Ctrl+S", "save"),
    ("Esc", "back to viewer (asks if unsaved)"),
    ("Ctrl+Z / Ctrl+Y", "undo / redo"),
];

pub fn draw(frame: &mut Frame, area: Rect) {
    let key_style = Style::default()
        .fg(Color::Indexed(75))
        .add_modifier(Modifier::BOLD);

    let lines: Vec<Line> = KEYS
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("  {key:<24}"), key_style),
                Span::raw(desc.to_string()),
            ])
        })
        .collect();

    let height = (lines.len() as u16 + 4).min(area.height.saturating_sub(2));
    let popup = centered(area, 64.min(area.width), height);

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" mdpad — keys ")
                .title_style(Style::default().add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Indexed(240)))
                .padding(Padding::vertical(1)),
        ),
        popup,
    );
}
