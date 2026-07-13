//! Status bar composition and the unsaved-changes confirm dialog.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};

use crate::ui::search::SearchState;
use crate::ui::toc::centered;
use crate::ui::viewer::draw_status;

/// Everything the status bar needs to describe the current app state.
pub struct StatusContext<'a> {
    pub editing: bool,
    pub search_input: bool,
    pub file_name: String,
    pub dirty: bool,
    pub search: &'a SearchState,
    pub status_message: Option<&'a str>,
    /// (bottom visible line, total lines) for the percent indicator.
    pub position: (usize, usize),
}

pub fn draw_statusbar(frame: &mut Frame, area: Rect, ctx: StatusContext<'_>) {
    let mut left: Vec<Span<'static>> = Vec::new();
    let mode_tag = if ctx.editing { " EDIT " } else { " READ " };
    left.push(Span::styled(
        mode_tag,
        Style::default()
            .bg(Color::Indexed(if ctx.editing { 130 } else { 24 }))
            .fg(Color::Indexed(255))
            .add_modifier(Modifier::BOLD),
    ));
    left.push(Span::raw(" "));
    left.push(Span::styled(
        ctx.file_name,
        Style::default().add_modifier(Modifier::BOLD),
    ));
    if ctx.dirty {
        left.push(Span::styled(
            " [+]",
            Style::default().fg(Color::Indexed(214)),
        ));
    }

    // Search input takes over the middle slot while typing.
    let message = if ctx.search_input {
        Some(format!("/{}▏", ctx.search.query))
    } else if ctx.search.is_active() {
        let total = ctx.search.matches.len();
        if total > 0 {
            Some(format!(
                "/{}  {}/{}",
                ctx.search.query,
                ctx.search.current + 1,
                total
            ))
        } else {
            Some(format!("/{}  no matches", ctx.search.query))
        }
    } else {
        ctx.status_message.map(str::to_string)
    };

    let right = if ctx.editing {
        "^S save  Esc back".to_string()
    } else {
        let (bottom, total) = ctx.position;
        let pct = (bottom * 100).checked_div(total).unwrap_or(100);
        format!("{pct:>3}%  ? help")
    };

    draw_status(frame, area, left, message.as_deref(), right);
}

pub fn draw_confirm(frame: &mut Frame, area: Rect) {
    let popup = centered(area, 44.min(area.width), 5);
    let lines = vec![
        Line::from(Span::styled(
            "Unsaved changes",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled(
                "[s]",
                Style::default()
                    .fg(Color::Indexed(75))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("ave   "),
            Span::styled(
                "[d]",
                Style::default()
                    .fg(Color::Indexed(203))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("iscard   "),
            Span::styled(
                "[c]",
                Style::default()
                    .fg(Color::Indexed(245))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("ancel"),
        ]),
    ];
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines).centered().block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Indexed(214)))
                .padding(Padding::horizontal(1)),
        ),
        popup,
    );
}
