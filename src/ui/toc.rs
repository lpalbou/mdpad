//! Table-of-contents overlay: headings extracted from rendered lines.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding};

use crate::render::RenderedLine;

pub struct TocEntry {
    pub level: u8,
    pub text: String,
    /// Index into the rendered line vector.
    pub line: usize,
}

pub struct TocState {
    pub entries: Vec<TocEntry>,
    pub selected: usize,
}

impl TocState {
    pub fn build(lines: &[RenderedLine]) -> Self {
        let entries = lines
            .iter()
            .enumerate()
            .filter_map(|(i, rl)| {
                rl.heading.as_ref().map(|h| TocEntry {
                    level: h.level,
                    text: h.text.clone(),
                    line: i,
                })
            })
            .collect();
        Self {
            entries,
            selected: 0,
        }
    }

    /// Select the heading the viewport currently sits in.
    pub fn sync_to_scroll(&mut self, scroll: usize) {
        let mut best = 0;
        for (i, e) in self.entries.iter().enumerate() {
            if e.line <= scroll {
                best = i;
            } else {
                break;
            }
        }
        self.selected = best;
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let max = self.entries.len() as isize - 1;
        self.selected = (self.selected as isize + delta).clamp(0, max) as usize;
    }

    pub fn selected_line(&self) -> Option<usize> {
        self.entries.get(self.selected).map(|e| e.line)
    }
}

pub fn draw(frame: &mut Frame, area: Rect, toc: &TocState) {
    let width = (area.width as usize * 2 / 3)
        .clamp(30, 70)
        .min(area.width as usize) as u16;
    // Clamp in usize BEFORE casting: 65k+ headings would overflow u16.
    let height = (toc.entries.len().saturating_add(4))
        .min(area.height.saturating_sub(4) as usize)
        .max(5) as u16;
    let popup = centered(area, width, height);

    let items: Vec<ListItem> = toc
        .entries
        .iter()
        .map(|e| {
            let indent = "  ".repeat((e.level.saturating_sub(1)) as usize);
            let style = if e.level <= 2 {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::styled(e.text.clone(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(toc.selected));
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Contents  (Enter: jump, Esc: close) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Indexed(240)))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Indexed(238))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_widget(Clear, popup);
    frame.render_stateful_widget(list, popup, &mut state);
}

pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
