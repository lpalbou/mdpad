//! Document viewport, scrollbar and status bar drawing.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::render::RenderedLine;
use crate::ui::search::{SearchState, highlight_line};

pub struct Viewer {
    pub scroll: usize,
}

impl Viewer {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    pub fn max_scroll(&self, total: usize, viewport: usize) -> usize {
        total.saturating_sub(viewport)
    }

    pub fn clamp(&mut self, total: usize, viewport: usize) {
        self.scroll = self.scroll.min(self.max_scroll(total, viewport));
    }

    pub fn scroll_by(&mut self, delta: isize, total: usize, viewport: usize) {
        let max = self.max_scroll(total, viewport) as isize;
        self.scroll = (self.scroll as isize + delta).clamp(0, max) as usize;
    }

    /// Center a target line in the viewport (search jumps, TOC jumps).
    pub fn center_on(&mut self, line: usize, total: usize, viewport: usize) {
        let half = viewport / 2;
        self.scroll = line
            .saturating_sub(half)
            .min(self.max_scroll(total, viewport));
    }

    /// Draw the document slice + scrollbar into `area`.
    pub fn draw_document(
        &self,
        frame: &mut Frame,
        area: Rect,
        lines: &[RenderedLine],
        search: &SearchState,
    ) {
        let height = area.height as usize;
        let end = (self.scroll + height).min(lines.len());
        let current = search.current_match();

        // Matches are sorted by line: binary-search the visible window once
        // instead of scanning every match for every visible row.
        let window = if search.is_active() {
            let lo = search.matches.partition_point(|m| m.line < self.scroll);
            let hi = search.matches.partition_point(|m| m.line < end);
            &search.matches[lo..hi]
        } else {
            &[]
        };

        let mut visible: Vec<Line<'static>> = Vec::with_capacity(height);
        for (idx, rl) in lines[self.scroll..end].iter().enumerate() {
            let abs = self.scroll + idx;
            let ranges: Vec<(usize, usize, bool)> = window
                .iter()
                .filter(|m| m.line == abs)
                .map(|m| (m.start, m.end, current.is_some_and(|c| c == *m)))
                .collect();
            if ranges.is_empty() {
                visible.push(rl.line.clone());
            } else {
                visible.push(highlight_line(&rl.line, &ranges));
            }
        }

        frame.render_widget(Paragraph::new(visible), area);

        if lines.len() > height {
            let mut sb_state =
                ScrollbarState::new(lines.len().saturating_sub(height)).position(self.scroll);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .thumb_style(Style::default().fg(Color::Indexed(240)))
                    .track_style(Style::default().fg(Color::Indexed(236))),
                area,
                &mut sb_state,
            );
        }
    }
}

/// Bottom status bar. `left` describes state (file, dirty, mode), `right`
/// shows position; the middle hosts transient messages.
pub fn draw_status(
    frame: &mut Frame,
    area: Rect,
    left: Vec<Span<'static>>,
    message: Option<&str>,
    right: String,
) {
    let bar_style = Style::default()
        .bg(Color::Indexed(236))
        .fg(Color::Indexed(250));
    let [left_area, msg_area, right_area] = Layout::horizontal([
        Constraint::Fill(2),
        Constraint::Fill(3),
        Constraint::Length(right.len() as u16 + 2),
    ])
    .areas(area);

    let mut left_spans = vec![Span::raw(" ")];
    left_spans.extend(left);
    frame.render_widget(
        Paragraph::new(Line::from(left_spans)).style(bar_style),
        left_area,
    );

    let msg = message.unwrap_or("");
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            msg.to_string(),
            Style::default()
                .fg(Color::Indexed(214))
                .add_modifier(Modifier::BOLD),
        )))
        .style(bar_style)
        .centered(),
        msg_area,
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::raw(format!("{right} "))))
            .style(bar_style)
            .right_aligned(),
        right_area,
    );
}
