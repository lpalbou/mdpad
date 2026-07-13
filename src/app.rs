//! Application state machine and event loop.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use ratatui::layout::{Constraint, Layout, Rect};

use crate::cli::Args;
use crate::markdown::model::Block as MdBlock;
use crate::markdown::parser::parse;
use crate::render::inline::LinkMode;
use crate::render::{RenderedLine, Renderer};
use crate::ui::editor::{Editor, spawn_external};
use crate::ui::search::SearchState;
use crate::ui::statusbar::{StatusContext, draw_confirm, draw_statusbar};
use crate::ui::term::TermGuard;
use crate::ui::toc::TocState;
use crate::ui::viewer::Viewer;
use crate::ui::{help, toc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    View,
    SearchInput,
    Toc,
    Help,
    Edit,
    /// Editor Esc with unsaved changes: save / discard / cancel.
    ConfirmDiscard,
}

const STATUS_TTL: Duration = Duration::from_secs(4);

pub fn run(
    source: String,
    path: Option<PathBuf>,
    renderer: Renderer,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut guard, mut terminal) = TermGuard::enter(!args.no_mouse)?;
    let mut app = App::new(source, path, renderer);
    let result = app.event_loop(&mut terminal, &mut guard);
    drop(guard);
    result
}

struct App {
    source: String,
    path: Option<PathBuf>,
    renderer: Renderer,
    blocks: Vec<MdBlock>,
    lines: Vec<RenderedLine>,
    /// (plain text, lowercased) per rendered line; feeds incremental search.
    haystacks: Vec<(String, String)>,
    viewer: Viewer,
    search: SearchState,
    toc: TocState,
    editor: Option<Editor>,
    mode: Mode,
    status: Option<(String, Instant)>,
    width: u16,
    doc_height: u16,
    quit: bool,
}

impl App {
    fn new(source: String, path: Option<PathBuf>, renderer: Renderer) -> Self {
        let blocks = parse(&source).blocks;
        Self {
            source,
            path,
            renderer,
            blocks,
            lines: Vec::new(),
            haystacks: Vec::new(),
            viewer: Viewer::new(),
            search: SearchState::default(),
            toc: TocState {
                entries: Vec::new(),
                selected: 0,
            },
            editor: None,
            mode: Mode::View,
            status: None,
            width: 0,
            doc_height: 0,
            quit: false,
        }
    }

    fn event_loop(
        &mut self,
        terminal: &mut crate::ui::term::Term,
        guard: &mut TermGuard,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut needs_redraw = true;
        while !self.quit {
            let size = terminal.size()?;
            if size.width != self.width {
                self.width = size.width;
                self.refresh();
                needs_redraw = true;
            }
            if needs_redraw {
                terminal.draw(|frame| self.draw(frame))?;
                needs_redraw = false;
            }

            if !event::poll(Duration::from_millis(250))? {
                // Idle: redraw only to expire a transient status message.
                if let Some((_, t)) = &self.status
                    && t.elapsed() > STATUS_TTL
                {
                    self.status = None;
                    needs_redraw = true;
                }
                continue;
            }
            // Drain the entire pending burst (fast typing, paste, resize
            // storms) before the next draw: one frame per burst, not one
            // frame per event.
            loop {
                match event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => {
                        self.on_key(key, terminal, guard)?
                    }
                    Event::Mouse(m) => match m.kind {
                        MouseEventKind::ScrollDown => self.scroll(3),
                        MouseEventKind::ScrollUp => self.scroll(-3),
                        _ => {}
                    },
                    Event::Resize(w, _) if w != self.width => {
                        self.width = w;
                        self.refresh();
                    }
                    _ => {}
                }
                needs_redraw = true;
                if self.quit || !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Re-render the document at the current width and rebuild derived state.
    fn refresh(&mut self) {
        let width = self.width.max(20) as usize;
        self.lines = self.renderer.render(&self.blocks, width);
        self.haystacks = self
            .lines
            .iter()
            .map(|rl| {
                let text = rl.plain_text();
                let lower = text.to_lowercase();
                (text, lower)
            })
            .collect();
        self.toc = TocState::build(&self.lines);
        if self.search.is_active() {
            self.search.run(&self.haystacks);
            // Keep the "current" match near the viewport instead of
            // teleporting to match 0 after a resize.
            self.search.seek_from(self.viewer.scroll);
        }
        self.viewer
            .clamp(self.lines.len(), self.doc_height as usize);
    }

    fn set_source(&mut self, source: String) {
        self.source = source;
        self.blocks = parse(&self.source).blocks;
        self.refresh();
    }

    fn flash(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    // ---------- drawing ----------

    fn draw(&mut self, frame: &mut Frame) {
        let [doc_area, status_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
        self.doc_height = doc_area.height;

        match self.mode {
            Mode::Edit | Mode::ConfirmDiscard => {
                if let Some(editor) = &self.editor {
                    editor.draw(frame, doc_area);
                }
            }
            _ => {
                self.viewer
                    .clamp(self.lines.len(), doc_area.height as usize);
                self.viewer
                    .draw_document(frame, doc_area, &self.lines, &self.search);
            }
        }

        self.draw_statusbar(frame, status_area);

        match self.mode {
            Mode::Toc => toc::draw(frame, doc_area, &self.toc),
            Mode::Help => help::draw(frame, doc_area),
            Mode::ConfirmDiscard => draw_confirm(frame, doc_area),
            _ => {}
        }
    }

    fn draw_statusbar(&mut self, frame: &mut Frame, area: Rect) {
        let editing = matches!(self.mode, Mode::Edit | Mode::ConfirmDiscard);
        let total = self.lines.len();
        let bottom = (self.viewer.scroll + self.doc_height as usize).min(total);
        let ctx = StatusContext {
            editing,
            search_input: self.mode == Mode::SearchInput,
            file_name: self
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(stdin)".into()),
            dirty: self.editor.as_ref().is_some_and(|e| e.dirty),
            search: &self.search,
            // Expiry is handled by the event loop's idle tick.
            status_message: self.status.as_ref().map(|(m, _)| m.as_str()),
            position: (bottom, total),
        };
        draw_statusbar(frame, area, ctx);
    }

    // ---------- input ----------

    fn on_key(
        &mut self,
        key: KeyEvent,
        terminal: &mut crate::ui::term::Term,
        guard: &mut TermGuard,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ctrl+C always exits, except with unsaved edits where it prompts
        // (including in the prompt itself: a reflexive second Ctrl+C must
        // not silently destroy the buffer).
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            let editing = matches!(self.mode, Mode::Edit | Mode::ConfirmDiscard);
            if editing && self.editor.as_ref().is_some_and(|e| e.dirty) {
                self.mode = Mode::ConfirmDiscard;
            } else {
                self.quit = true;
            }
            return Ok(());
        }
        match self.mode {
            Mode::View => self.key_view(key, terminal, guard)?,
            Mode::SearchInput => self.key_search(key),
            Mode::Toc => self.key_toc(key),
            Mode::Help => {
                if matches!(
                    key.code,
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')
                ) {
                    self.mode = Mode::View;
                }
            }
            Mode::Edit => self.key_edit(key),
            Mode::ConfirmDiscard => self.key_confirm(key),
        }
        Ok(())
    }

    fn key_view(
        &mut self,
        key: KeyEvent,
        terminal: &mut crate::ui::term::Term,
        guard: &mut TermGuard,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let page = self.doc_height.max(1) as isize;
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('m') => {
                let on = !guard.mouse;
                guard.set_mouse(on);
                self.flash(if on {
                    "mouse scroll on (m: off for text selection)"
                } else {
                    "mouse released: native text selection works"
                });
            }
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => self.scroll(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll(-1),
            KeyCode::Char(' ') | KeyCode::PageDown | KeyCode::Char('f') => self.scroll(page),
            KeyCode::Char('b') | KeyCode::PageUp => self.scroll(-page),
            KeyCode::Char('d') => self.scroll(page / 2),
            KeyCode::Char('u') => self.scroll(-page / 2),
            KeyCode::Char('g') | KeyCode::Home => self.viewer.scroll = 0,
            KeyCode::Char('G') | KeyCode::End => {
                self.viewer.scroll = self
                    .viewer
                    .max_scroll(self.lines.len(), self.doc_height as usize)
            }
            KeyCode::Char('/') => {
                self.search.query.clear();
                self.search.matches.clear();
                self.mode = Mode::SearchInput;
            }
            KeyCode::Char('n') => self.jump_match(true),
            KeyCode::Char('N') => self.jump_match(false),
            KeyCode::Esc => {
                self.search.query.clear();
                self.search.matches.clear();
            }
            KeyCode::Char('t') => {
                self.toc.sync_to_scroll(self.viewer.scroll);
                if self.toc.entries.is_empty() {
                    self.flash("no headings in this document");
                } else {
                    self.mode = Mode::Toc;
                }
            }
            KeyCode::Char('L') => {
                self.renderer.link_mode = match self.renderer.link_mode {
                    LinkMode::TextOnly => LinkMode::WithUrl,
                    LinkMode::WithUrl => LinkMode::TextOnly,
                };
                self.refresh();
                self.flash(match self.renderer.link_mode {
                    LinkMode::WithUrl => "link URLs shown",
                    LinkMode::TextOnly => "link URLs hidden",
                });
            }
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('e') => self.open_builtin_editor(),
            KeyCode::Char('E') => self.open_external_editor(terminal, guard),
            KeyCode::Char('?') => self.mode = Mode::Help,
            _ => {}
        }
        Ok(())
    }

    fn key_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search.query.clear();
                self.search.matches.clear();
                self.mode = Mode::View;
            }
            KeyCode::Enter => {
                self.mode = Mode::View;
                if self.search.is_active() && self.search.matches.is_empty() {
                    self.flash("no matches");
                }
            }
            KeyCode::Backspace => {
                self.search.query.pop();
                self.incremental_search();
            }
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.search.query.push(c);
                self.incremental_search();
            }
            _ => {}
        }
    }

    fn incremental_search(&mut self) {
        self.search.run(&self.haystacks);
        self.search.seek_from(self.viewer.scroll);
        if let Some(m) = self.search.current_match() {
            self.viewer
                .center_on(m.line, self.lines.len(), self.doc_height as usize);
        }
    }

    fn jump_match(&mut self, forward: bool) {
        if !self.search.is_active() || self.search.matches.is_empty() {
            self.flash("no active search");
            return;
        }
        if forward {
            self.search.next()
        } else {
            self.search.prev()
        }
        if let Some(m) = self.search.current_match() {
            self.viewer
                .center_on(m.line, self.lines.len(), self.doc_height as usize);
        }
    }

    fn key_toc(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('t') | KeyCode::Char('q') => self.mode = Mode::View,
            KeyCode::Char('j') | KeyCode::Down => self.toc.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.toc.move_selection(-1),
            KeyCode::Enter => {
                if let Some(line) = self.toc.selected_line() {
                    self.viewer.scroll = line.min(
                        self.viewer
                            .max_scroll(self.lines.len(), self.doc_height as usize),
                    );
                }
                self.mode = Mode::View;
            }
            _ => {}
        }
    }

    fn key_edit(&mut self, key: KeyEvent) {
        let Some(editor) = &mut self.editor else {
            self.mode = Mode::View;
            return;
        };
        // Intercept editor-level commands before the textarea sees them.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.save_editor();
            return;
        }
        if key.code == KeyCode::Esc {
            if editor.dirty {
                self.mode = Mode::ConfirmDiscard;
            } else {
                self.editor = None;
                self.mode = Mode::View;
            }
            return;
        }
        if editor.textarea.input(key) {
            // Undo/redo can land back on the saved state; recheck instead of
            // latching dirty forever.
            let undo_redo = key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('z') | KeyCode::Char('y'));
            if undo_redo {
                editor.refresh_dirty();
            } else {
                editor.dirty = true;
            }
        }
    }

    fn key_confirm(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.save_editor();
                if self.editor.as_ref().is_some_and(|e| !e.dirty) {
                    self.editor = None;
                    self.mode = Mode::View;
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Discard buffer changes; the document keeps its saved content.
                self.editor = None;
                self.mode = Mode::View;
                self.flash("changes discarded");
            }
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => self.mode = Mode::Edit,
            _ => {}
        }
    }

    // ---------- actions ----------

    fn scroll(&mut self, delta: isize) {
        if matches!(self.mode, Mode::View) {
            self.viewer
                .scroll_by(delta, self.lines.len(), self.doc_height as usize);
        }
    }

    fn reload(&mut self) {
        let Some(path) = &self.path else {
            self.flash("(stdin) cannot be reloaded");
            return;
        };
        match std::fs::read_to_string(path) {
            Ok(source) => {
                self.set_source(source);
                self.flash("reloaded");
            }
            Err(e) => self.flash(format!("reload failed: {e}")),
        }
    }

    fn open_builtin_editor(&mut self) {
        let editor = Editor::new(&self.source);
        let mixed = editor.mixed_endings;
        self.editor = Some(editor);
        self.mode = Mode::Edit;
        if self.path.is_none() {
            self.flash("editing stdin: saving is disabled");
        } else if mixed {
            self.flash("mixed line endings: saving will unify to CRLF");
        }
    }

    fn save_editor(&mut self) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        let Some(path) = &self.path else {
            self.flash("no file to save to (opened from stdin)");
            return;
        };
        match editor.save(path) {
            Ok(()) => {
                let content = editor.content();
                self.set_source(content);
                self.flash("saved");
            }
            Err(e) => self.flash(format!("save failed: {e}")),
        }
    }

    fn open_external_editor(
        &mut self,
        terminal: &mut crate::ui::term::Term,
        guard: &mut TermGuard,
    ) {
        let Some(path) = self.path.clone() else {
            self.flash("$EDITOR needs a file (opened from stdin)");
            return;
        };
        guard.suspend();
        let result = spawn_external(&path);
        let _ = guard.resume();
        let _ = terminal.clear();
        match result {
            Ok(()) => match std::fs::read_to_string(&path) {
                Ok(source) => {
                    self.set_source(source);
                    self.flash("reloaded after $EDITOR");
                }
                Err(e) => self.flash(format!("reload failed: {e}")),
            },
            Err(e) => self.flash(e),
        }
    }
}
