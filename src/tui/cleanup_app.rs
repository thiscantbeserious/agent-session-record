//! Cleanup command TUI application
//!
//! Interactive file explorer for selecting and deleting session recordings.
//! Features: multi-select, search, agent filter, glob select, storage preview.

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::app::{handle_shared_key, App, KeyResult, SharedMode, SharedState, TuiApp};
use super::widgets::preview::prefetch_adjacent_previews;
use super::widgets::FileItem;
use crate::theme::current_theme;
use crate::StorageManager;

/// UI mode for the cleanup application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Normal browsing mode
    #[default]
    Normal,
    /// Search mode - typing filters by filename
    Search,
    /// Agent filter mode - selecting agent to filter by
    AgentFilter,
    /// Glob select mode - enter pattern to select matching files
    GlobSelect,
    /// Help mode - showing keyboard shortcuts
    Help,
    /// Confirm delete mode
    ConfirmDelete,
}

impl Mode {
    /// Map this mode to a `SharedMode`, if it is a shared mode.
    fn to_shared_mode(self) -> Option<SharedMode> {
        match self {
            Mode::Normal => Some(SharedMode::Normal),
            Mode::Search => Some(SharedMode::Search),
            Mode::AgentFilter => Some(SharedMode::AgentFilter),
            Mode::Help => Some(SharedMode::Help),
            Mode::ConfirmDelete => Some(SharedMode::ConfirmDelete),
            Mode::GlobSelect => None,
        }
    }

    /// Convert a `SharedMode` into the corresponding `Mode`.
    fn from_shared_mode(shared: SharedMode) -> Self {
        match shared {
            SharedMode::Normal => Mode::Normal,
            SharedMode::Search => Mode::Search,
            SharedMode::AgentFilter => Mode::AgentFilter,
            SharedMode::Help => Mode::Help,
            SharedMode::ConfirmDelete => Mode::ConfirmDelete,
        }
    }
}

/// Cleanup application state
pub struct CleanupApp {
    /// Base app for terminal handling
    app: App,
    /// Shared state (explorer, search, agent filter, preview cache, etc.)
    shared_state: SharedState,
    /// Current UI mode
    mode: Mode,
    /// Glob pattern input buffer
    glob_input: String,
    /// Storage manager for deletion (kept for future use by bulk-delete)
    #[allow(dead_code)]
    storage: StorageManager,
    /// Whether files were deleted (for success message)
    files_deleted: bool,
}

impl CleanupApp {
    /// Create a new cleanup application with the given sessions.
    pub fn new(items: Vec<FileItem>, storage: StorageManager) -> Result<Self> {
        let app = App::new(Duration::from_millis(250))?;
        let shared_state = SharedState::new(items);

        Ok(Self {
            app,
            shared_state,
            mode: Mode::Normal,
            glob_input: String::new(),
            storage,
            files_deleted: false,
        })
    }

    /// Check if any files were deleted during this session
    pub fn files_were_deleted(&self) -> bool {
        self.files_deleted
    }
}

// --- TuiApp trait implementation ---

impl TuiApp for CleanupApp {
    fn app(&mut self) -> &mut App {
        &mut self.app
    }

    fn shared_state(&mut self) -> &mut SharedState {
        &mut self.shared_state
    }

    fn is_normal_mode(&self) -> bool {
        matches!(self.mode, Mode::Normal)
    }

    fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Try shared key handling first for shared modes
        if let Some(shared_mode) = self.mode.to_shared_mode() {
            match handle_shared_key(&shared_mode, key, &mut self.shared_state) {
                KeyResult::Consumed => return Ok(()),
                KeyResult::EnterMode(mode) => {
                    self.mode = Mode::from_shared_mode(mode);
                    return Ok(());
                }
                KeyResult::NotConsumed => {}
            }
        }

        // App-specific key handling
        match self.mode {
            Mode::Normal => self.handle_normal_key(key)?,
            Mode::GlobSelect => self.handle_glob_key(key)?,
            Mode::ConfirmDelete => self.handle_confirm_delete_key(key)?,
            // Search, AgentFilter, Help are fully handled by shared logic above
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        // Get terminal size for page calculations
        let (_, height) = self.app.size()?;
        self.shared_state
            .explorer
            .set_page_size((height.saturating_sub(6)) as usize);

        // Poll cache for completed loads and request prefetch
        self.shared_state.preview_cache.poll();
        prefetch_adjacent_previews(
            &self.shared_state.explorer,
            &mut self.shared_state.preview_cache,
        );

        let mode = self.mode;
        let glob_input = &self.glob_input;

        // Compute status and footer text
        let status_text = compute_status_text(mode, glob_input, &self.shared_state);
        let footer_text = compute_footer_text(mode, self.shared_state.explorer.selected_count());

        // Calculate selected size for confirm delete modal
        let selected_size: u64 = self
            .shared_state
            .explorer
            .selected_items()
            .iter()
            .map(|i| i.size)
            .sum();
        let selected_count = self.shared_state.explorer.selected_count();

        // Get preview for current selection from cache
        let current_path = self
            .shared_state
            .explorer
            .selected_item()
            .map(|i| i.path.clone());
        let preview = current_path
            .as_ref()
            .and_then(|p| self.shared_state.preview_cache.get(p));

        // Extract &mut explorer before the closure (avoids borrow conflict with self.app)
        let explorer = &mut self.shared_state.explorer;

        self.app.draw(|frame| {
            let area = frame.area();
            let chunks = super::app::layout::build_explorer_layout(area);

            // Render file explorer list (with checkboxes for multi-select)
            super::app::list_view::render_explorer_list(
                frame, chunks[0], explorer, preview, true, // show checkboxes in cleanup view
                false,
            );

            // Render status line and footer
            super::app::status_footer::render_status_line(frame, chunks[1], &status_text);
            super::app::status_footer::render_footer_text(frame, chunks[2], footer_text);

            // Render modal overlays
            match mode {
                Mode::Help => render_help_modal(frame, area),
                Mode::ConfirmDelete => {
                    render_confirm_delete_modal(frame, area, selected_count, selected_size);
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}

// --- App-specific key handlers ---

impl CleanupApp {
    /// Handle app-specific keys in normal mode.
    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Selection
            KeyCode::Char(' ') => {
                self.shared_state.explorer.toggle_select();
            }
            KeyCode::Char('a') => {
                self.shared_state.explorer.toggle_all();
            }
            KeyCode::Char('g') => {
                self.mode = Mode::GlobSelect;
                self.glob_input.clear();
            }

            // Actions
            KeyCode::Enter => {
                if self.shared_state.explorer.selected_count() > 0 {
                    self.mode = Mode::ConfirmDelete;
                }
            }

            // Clear/Cancel
            KeyCode::Esc => {
                if self.shared_state.explorer.selected_count() > 0 {
                    // First Esc clears selection
                    self.shared_state.explorer.select_none();
                } else {
                    // Second Esc clears filters
                    self.shared_state.explorer.clear_filters();
                    self.shared_state.search_input.clear();
                    self.shared_state.agent_filter_idx = 0;
                }
            }

            // Quit
            KeyCode::Char('q') => self.app.quit(),

            _ => {}
        }
        Ok(())
    }

    /// Handle keys in glob select mode.
    fn handle_glob_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                if !self.glob_input.is_empty() {
                    let pattern = self.glob_input.clone();
                    let matched = self.select_by_glob(&pattern);
                    self.shared_state.status_message =
                        Some(format!("Selected {} matching files", matched));
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.glob_input.pop();
            }
            KeyCode::Char(c) => {
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                    self.glob_input.push(c);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Select items matching a glob-like pattern.
    /// Supports: * (any chars), ? (single char), agent/pattern syntax
    fn select_by_glob(&mut self, pattern: &str) -> usize {
        // Parse agent/pattern syntax (e.g., "claude/*.cast" or "*2024*")
        let (agent_filter, file_pattern) = if let Some(slash_pos) = pattern.find('/') {
            let agent = &pattern[..slash_pos];
            let pat = &pattern[slash_pos + 1..];
            (Some(agent), pat)
        } else {
            (None, pattern)
        };

        // Collect matching items that aren't already selected
        let items_to_select: Vec<(usize, String, String, bool)> = self
            .shared_state
            .explorer
            .visible_items()
            .map(|(vis_idx, item, is_selected)| {
                (vis_idx, item.agent.clone(), item.name.clone(), is_selected)
            })
            .collect();

        // Track original position
        let original_selected = self.shared_state.explorer.selected();
        let mut actual_count = 0;

        // Select matching items
        for (vis_idx, agent, name, is_selected) in items_to_select {
            let matches = if let Some(agent_pat) = agent_filter {
                glob_match(&agent, agent_pat) && glob_match(&name, file_pattern)
            } else {
                glob_match(&name, file_pattern)
            };
            if matches && !is_selected {
                self.shared_state.explorer.home();
                for _ in 0..vis_idx {
                    self.shared_state.explorer.down();
                }
                self.shared_state.explorer.toggle_select();
                actual_count += 1;
            }
        }

        // Restore original position
        self.shared_state.explorer.home();
        let max_pos = self.shared_state.explorer.len().saturating_sub(1);
        for _ in 0..original_selected.min(max_pos) {
            self.shared_state.explorer.down();
        }

        actual_count
    }

    /// Handle keys in confirm delete mode.
    fn handle_confirm_delete_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.delete_selected()?;
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    /// Delete all selected sessions.
    fn delete_selected(&mut self) -> Result<()> {
        let selected_items = self.shared_state.explorer.selected_items();
        if selected_items.is_empty() {
            return Ok(());
        }

        let paths: Vec<String> = selected_items.iter().map(|i| i.path.clone()).collect();
        let count = paths.len();

        let mut deleted = 0;
        let mut total_freed: u64 = 0;
        for path in &paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                total_freed += metadata.len();
            }
            if std::fs::remove_file(path).is_ok() {
                deleted += 1;
            }
        }

        for path in &paths {
            self.shared_state.explorer.remove_item(path);
        }

        self.update_delete_status(deleted, count, total_freed);

        Ok(())
    }

    /// Update the status message after a bulk delete operation.
    fn update_delete_status(&mut self, deleted: usize, count: usize, total_freed: u64) {
        if deleted == count {
            self.shared_state.status_message = Some(format!(
                "Deleted {} sessions (freed {})",
                deleted,
                format_size(total_freed)
            ));
            self.files_deleted = true;
        } else {
            self.shared_state.status_message = Some(format!(
                "Deleted {}/{} sessions (some files could not be removed)",
                deleted, count
            ));
            if deleted > 0 {
                self.files_deleted = true;
            }
        }
    }
}

// --- Status and footer helpers ---

/// Compute the status text for the given mode and shared state.
fn compute_status_text(mode: Mode, glob_input: &str, state: &SharedState) -> String {
    if let Some(msg) = &state.status_message {
        return msg.clone();
    }
    match mode {
        Mode::Search => format!("Search: {}_", state.search_input),
        Mode::GlobSelect => format!("Glob pattern: {}_", glob_input),
        Mode::AgentFilter => {
            let agent = &state.available_agents[state.agent_filter_idx];
            format!(
                "Filter by agent: {} (left/right to change, Enter to apply)",
                agent
            )
        }
        Mode::ConfirmDelete | Mode::Help => String::new(),
        Mode::Normal => format_normal_status(&state.explorer),
    }
}

/// Format the status line for normal mode (shows selection or filter info).
fn format_normal_status(explorer: &super::widgets::FileExplorer) -> String {
    let selected_count = explorer.selected_count();
    if selected_count > 0 {
        let selected_size: u64 = explorer.selected_items().iter().map(|i| i.size).sum();
        return format!(
            "{} selected ({}) | {} total sessions",
            selected_count,
            format_size(selected_size),
            explorer.len()
        );
    }

    let mut parts = vec![];
    if let Some(search) = explorer.search_filter() {
        parts.push(format!("search: \"{}\"", search));
    }
    if let Some(agent) = explorer.agent_filter() {
        parts.push(format!("agent: {}", agent));
    }
    if parts.is_empty() {
        format!("{} sessions | Space to select", explorer.len())
    } else {
        format!(
            "{} sessions ({}) | Space to select",
            explorer.len(),
            parts.join(", ")
        )
    }
}

/// Get the footer text for the given mode.
fn compute_footer_text(mode: Mode, selected_count: usize) -> &'static str {
    match mode {
        Mode::Search => "Esc: cancel | Enter: apply | Backspace: delete",
        Mode::GlobSelect => "Esc: cancel | Enter: select matching | Backspace: delete",
        Mode::AgentFilter => "left/right: change | Enter: apply | Esc: cancel",
        Mode::ConfirmDelete => "y: confirm | n/Esc: cancel",
        Mode::Help => "Press any key to close",
        Mode::Normal => {
            if selected_count > 0 {
                "Space: toggle | a: toggle all | Enter: delete selected | Esc: clear | ?: help"
            } else {
                "Space: select | a: all | g: glob | /: search | f: filter | ?: help | q: quit"
            }
        }
    }
}

// --- Modal rendering ---

/// Render the help modal overlay for the cleanup app.
fn render_help_modal(frame: &mut Frame, area: Rect) {
    let theme = current_theme();

    let modal_width = 65.min(area.width.saturating_sub(4));
    let modal_height = 20.min(area.height.saturating_sub(4));
    let x = (area.width - modal_width) / 2;
    let y = (area.height - modal_height) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let help_text = build_help_text(&theme);
    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .title(" Help "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, modal_area);
}

/// Build the help text lines for the cleanup help modal.
fn build_help_text(theme: &crate::theme::Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            "Cleanup Keyboard Shortcuts",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  up/down, j/k", Style::default().fg(theme.accent)),
            Span::raw("   Move cursor"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn", Style::default().fg(theme.accent)),
            Span::raw("      Page up/down"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End", Style::default().fg(theme.accent)),
            Span::raw("       Go to first/last"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Selection",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Space", Style::default().fg(theme.accent)),
            Span::raw("          Toggle select current item"),
        ]),
        Line::from(vec![
            Span::styled("  a", Style::default().fg(theme.accent)),
            Span::raw("              Select all / Deselect all"),
        ]),
        Line::from(vec![
            Span::styled("  g", Style::default().fg(theme.accent)),
            Span::raw("              Glob select (e.g., *2024*, claude/*.cast)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Filtering",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  /", Style::default().fg(theme.accent)),
            Span::raw("              Search by filename"),
        ]),
        Line::from(vec![
            Span::styled("  f", Style::default().fg(theme.accent)),
            Span::raw("              Filter by agent"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(theme.error)),
            Span::raw("          Delete selected (with confirmation)"),
        ]),
        Line::from(vec![
            Span::styled("  Esc", Style::default().fg(theme.accent)),
            Span::raw("            Clear selection / Clear filters"),
        ]),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(theme.accent)),
            Span::raw("              Quit without deleting"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(theme.text_secondary),
        )),
    ]
}

/// Render the confirm delete modal overlay for bulk deletion.
fn render_confirm_delete_modal(frame: &mut Frame, area: Rect, count: usize, size: u64) {
    let theme = current_theme();

    let modal_width = 50.min(area.width.saturating_sub(4));
    let modal_height = 8.min(area.height.saturating_sub(4));
    let x = (area.width - modal_width) / 2;
    let y = (area.height - modal_height) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let text = vec![
        Line::from(Span::styled(
            "Delete Sessions?",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Sessions to delete: {}", count)),
        Line::from(format!("Storage to free: {}", format_size(size))),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", Style::default().fg(theme.error)),
            Span::raw(": Yes, delete  |  "),
            Span::styled("n", Style::default().fg(theme.accent)),
            Span::raw(": No, cancel"),
        ]),
    ];

    let confirm = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.error))
                .title(" Confirm Delete "),
        )
        .alignment(Alignment::Center);

    frame.render_widget(confirm, modal_area);
}

// --- Utility functions ---

/// Simple glob pattern matching.
/// Supports * (match any) and ? (match single char).
fn glob_match(text: &str, pattern: &str) -> bool {
    let text = text.to_lowercase();
    let pattern = pattern.to_lowercase();

    glob_match_recursive(&text, &pattern)
}

fn glob_match_recursive(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                let rest_pattern: String = pattern_chars.collect();
                if rest_pattern.is_empty() {
                    return true;
                }
                let rest_text: String = text_chars.collect();
                for i in 0..=rest_text.len() {
                    if glob_match_recursive(&rest_text[i..], &rest_pattern) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => match text_chars.next() {
                Some(t) if t == c => {}
                _ => return false,
            },
        }
    }

    text_chars.next().is_none()
}

/// Format a byte size as human-readable string.
fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_default_is_normal() {
        assert_eq!(Mode::default(), Mode::Normal);
    }

    #[test]
    fn mode_equality() {
        assert_eq!(Mode::Search, Mode::Search);
        assert_ne!(Mode::Search, Mode::Normal);
        assert_ne!(Mode::GlobSelect, Mode::Search);
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn mode_clone_and_copy() {
        let mode = Mode::Help;
        let cloned = mode.clone();
        let copied = mode;
        assert_eq!(cloned, copied);
    }

    #[test]
    fn mode_debug_format() {
        let mode = Mode::ConfirmDelete;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("ConfirmDelete"));
    }

    #[test]
    fn glob_mode_exists() {
        let mode = Mode::GlobSelect;
        let debug = format!("{:?}", mode);
        assert!(debug.contains("GlobSelect"));
    }

    // Glob matching tests

    #[test]
    fn glob_match_exact() {
        assert!(glob_match("test.cast", "test.cast"));
        assert!(!glob_match("test.cast", "other.cast"));
    }

    #[test]
    fn glob_match_star_any() {
        assert!(glob_match("test.cast", "*"));
        assert!(glob_match("test.cast", "*.cast"));
        assert!(glob_match("test.cast", "test.*"));
        assert!(glob_match("test.cast", "*test*"));
        assert!(glob_match("session_2024_01.cast", "*2024*"));
    }

    #[test]
    fn glob_match_question_single() {
        assert!(glob_match("test.cast", "tes?.cast"));
        assert!(glob_match("test.cast", "????.cast"));
        assert!(!glob_match("test.cast", "???.cast"));
    }

    #[test]
    fn glob_match_case_insensitive() {
        assert!(glob_match("TEST.CAST", "test.cast"));
        assert!(glob_match("Test.Cast", "TEST.CAST"));
        assert!(glob_match("MyFile.cast", "*myfile*"));
    }

    #[test]
    fn glob_match_complex_patterns() {
        assert!(glob_match(
            "session_2024_01_15.cast",
            "session_????_??_??.cast"
        ));
        assert!(glob_match("claude_session.cast", "*_session.cast"));
        assert!(!glob_match("test.txt", "*.cast"));
    }
}
