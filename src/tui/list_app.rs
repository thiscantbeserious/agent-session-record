//! List command TUI application
//!
//! Interactive file explorer for browsing and managing session recordings.
//! Features: search, agent filter, play, delete, add marker.

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::app::modals::render_confirm_delete_modal;
use super::app::{handle_shared_key, App, KeyResult, SharedMode, SharedState, TuiApp};
use super::widgets::preview::prefetch_adjacent_previews;
use super::widgets::FileItem;
use crate::asciicast::{apply_transforms, TransformResult};
use crate::files::backup::{backup_path_for, create_backup, has_backup, restore_from_backup};
use crate::theme::current_theme;

/// UI mode for the list application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Normal browsing mode
    #[default]
    Normal,
    /// Search mode - typing filters by filename
    Search,
    /// Agent filter mode - selecting agent to filter by
    AgentFilter,
    /// Help mode - showing keyboard shortcuts
    Help,
    /// Confirm delete mode
    ConfirmDelete,
    /// Context menu mode - showing actions for selected file
    ContextMenu,
    /// Optimize result mode - showing optimization results or error
    OptimizeResult,
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
            Mode::ContextMenu | Mode::OptimizeResult => None,
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

/// Context menu item definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuItem {
    Play,
    Copy,
    Optimize,
    Analyze,
    Restore,
    Delete,
    AddMarker,
}

impl ContextMenuItem {
    /// All menu items in display order
    pub const ALL: [ContextMenuItem; 7] = [
        ContextMenuItem::Play,
        ContextMenuItem::Copy,
        ContextMenuItem::Optimize,
        ContextMenuItem::Analyze,
        ContextMenuItem::Restore,
        ContextMenuItem::Delete,
        ContextMenuItem::AddMarker,
    ];

    /// Get the display label for this menu item
    pub fn label(&self) -> &'static str {
        match self {
            ContextMenuItem::Play => "Play",
            ContextMenuItem::Copy => "Copy to clipboard",
            ContextMenuItem::Optimize => "Optimize",
            ContextMenuItem::Analyze => "Analyze",
            ContextMenuItem::Restore => "Restore from backup",
            ContextMenuItem::Delete => "Delete",
            ContextMenuItem::AddMarker => "Add marker",
        }
    }

    /// Get the shortcut key hint for this menu item
    pub fn shortcut(&self) -> &'static str {
        match self {
            ContextMenuItem::Play => "p",
            ContextMenuItem::Copy => "c",
            ContextMenuItem::Optimize => "t",
            ContextMenuItem::Analyze => "a",
            ContextMenuItem::Restore => "r",
            ContextMenuItem::Delete => "d",
            ContextMenuItem::AddMarker => "m",
        }
    }
}

/// Holds the result of an optimize operation for display in modal.
#[derive(Debug, Clone)]
pub struct OptimizeResultState {
    /// The filename that was optimized
    pub filename: String,
    /// The result (Ok with data or Err with message)
    pub result: Result<TransformResult, String>,
}

/// List application state
pub struct ListApp {
    /// Base app for terminal handling
    app: App,
    /// Shared state (explorer, search, agent filter, preview cache, etc.)
    shared_state: SharedState,
    /// Current UI mode
    mode: Mode,
    /// Context menu selected index
    context_menu_idx: usize,
    /// Optimize result for modal display
    optimize_result: Option<OptimizeResultState>,
}

impl ListApp {
    /// Create a new list application with the given sessions.
    pub fn new(items: Vec<FileItem>) -> Result<Self> {
        let app = App::new(Duration::from_millis(250))?;
        let shared_state = SharedState::new(items);

        Ok(Self {
            app,
            shared_state,
            mode: Mode::Normal,
            context_menu_idx: 0,
            optimize_result: None,
        })
    }

    /// Set initial agent filter (for CLI argument support)
    pub fn set_agent_filter(&mut self, agent: &str) {
        if let Some(idx) = self
            .shared_state
            .available_agents
            .iter()
            .position(|a| a == agent)
        {
            self.shared_state.agent_filter_idx = idx;
            self.shared_state.apply_agent_filter();
        }
    }

    /// Render the help modal overlay.
    /// Public for snapshot testing.
    pub fn render_help_modal(frame: &mut Frame, area: Rect) {
        let theme = current_theme();

        // Center the modal
        let modal_width = 60.min(area.width.saturating_sub(4));
        let modal_height = 28.min(area.height.saturating_sub(4));
        let x = (area.width - modal_width) / 2;
        let y = (area.height - modal_height) / 2;
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear the area behind the modal
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

    /// Render the context menu modal overlay.
    ///
    /// This function is public to allow snapshot testing.
    pub fn render_context_menu_modal(
        frame: &mut Frame,
        area: Rect,
        selected_idx: usize,
        backup_exists: bool,
    ) {
        let theme = current_theme();

        // Center the modal
        let modal_width = 40.min(area.width.saturating_sub(4));
        let modal_height = (ContextMenuItem::ALL.len() + 5) as u16;
        let modal_height = modal_height.min(area.height.saturating_sub(4));
        let x = (area.width - modal_width) / 2;
        let y = (area.height - modal_height) / 2;
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear the area behind the modal
        frame.render_widget(Clear, modal_area);

        let lines = build_context_menu_lines(&theme, selected_idx, backup_exists);
        let menu = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent))
                    .title(" Menu "),
            )
            .alignment(Alignment::Left);

        frame.render_widget(menu, modal_area);
    }

    /// Render the optimize result modal overlay.
    ///
    /// This function is public to allow snapshot testing.
    pub fn render_optimize_result_modal(
        frame: &mut Frame,
        area: Rect,
        result_state: &OptimizeResultState,
    ) {
        let theme = current_theme();

        // Determine modal size based on success or error
        let is_success = result_state.result.is_ok();
        let modal_width = 55.min(area.width.saturating_sub(4));
        let modal_height = if is_success { 10 } else { 8 };
        let modal_height = modal_height.min(area.height.saturating_sub(4));

        // Center the modal
        let x = (area.width - modal_width) / 2;
        let y = (area.height - modal_height) / 2;
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear the area behind the modal
        frame.render_widget(Clear, modal_area);

        let (title, border_color, lines) = build_optimize_result_content(&theme, result_state);

        let modal = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(modal, modal_area);
    }
}

// --- TuiApp trait implementation ---

impl TuiApp for ListApp {
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
            Mode::ConfirmDelete => self.handle_confirm_delete_key(key)?,
            Mode::ContextMenu => self.handle_context_menu_key(key)?,
            Mode::OptimizeResult => self.handle_optimize_result_key(key)?,
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
        let context_menu_idx = self.context_menu_idx;
        let optimize_result = self.optimize_result.clone();

        // Check if backup exists for selected file (for context menu)
        let backup_exists = self
            .shared_state
            .explorer
            .selected_item()
            .map(|i| has_backup(std::path::Path::new(&i.path)))
            .unwrap_or(false);

        // Compute status and footer text before extracting preview
        // (extract_preview borrows preview_cache mutably, so compute these first)
        let status_text = compute_status_text(mode, &self.shared_state);
        let footer_text = compute_footer_text(mode);
        let selected_name = self
            .shared_state
            .explorer
            .selected_item()
            .map(|i| i.name.clone());

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

            // Render file explorer list
            super::app::list_view::render_explorer_list(
                frame,
                chunks[0],
                explorer,
                preview,
                false, // no checkboxes in list view
                backup_exists,
            );

            // Render status line and footer
            super::app::status_footer::render_status_line(frame, chunks[1], &status_text);
            super::app::status_footer::render_footer_text(frame, chunks[2], footer_text);

            // Render modal overlays
            match mode {
                Mode::Help => Self::render_help_modal(frame, area),
                Mode::ConfirmDelete => {
                    if let Some(ref name) = selected_name {
                        render_confirm_delete_modal(frame, area, name);
                    }
                }
                Mode::ContextMenu => {
                    Self::render_context_menu_modal(frame, area, context_menu_idx, backup_exists);
                }
                Mode::OptimizeResult => {
                    if let Some(ref result_state) = optimize_result {
                        Self::render_optimize_result_modal(frame, area, result_state);
                    }
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}

// --- App-specific key handlers ---

impl ListApp {
    /// Handle app-specific keys in normal mode.
    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if self.shared_state.explorer.selected_item().is_some() {
                    self.context_menu_idx = 0;
                    self.mode = Mode::ContextMenu;
                }
            }
            // Direct shortcuts (bypass context menu)
            KeyCode::Char('p') => self.play_session()?,
            KeyCode::Char('c') => self.copy_to_clipboard()?,
            KeyCode::Char('t') => self.optimize_session()?,
            KeyCode::Char('a') => self.analyze_session()?,
            KeyCode::Char('d') => {
                if self.shared_state.explorer.selected_item().is_some() {
                    self.mode = Mode::ConfirmDelete;
                }
            }
            KeyCode::Char('m') => self.add_marker()?,

            // Clear filters
            KeyCode::Esc => {
                self.shared_state.explorer.clear_filters();
                self.shared_state.search_input.clear();
                self.shared_state.agent_filter_idx = 0;
            }

            _ => {}
        }
        Ok(())
    }

    /// Handle keys in confirm delete mode.
    fn handle_confirm_delete_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.delete_session()?;
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in context menu mode.
    fn handle_context_menu_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.context_menu_idx > 0 {
                    self.context_menu_idx -= 1;
                } else {
                    self.context_menu_idx = ContextMenuItem::ALL.len() - 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.context_menu_idx = (self.context_menu_idx + 1) % ContextMenuItem::ALL.len();
            }
            KeyCode::Enter => self.execute_context_menu_action()?,
            KeyCode::Char(c) => {
                if let Some(idx) = shortcut_to_menu_idx(c) {
                    self.context_menu_idx = idx;
                    self.execute_context_menu_action()?;
                }
            }
            KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in optimize result mode.
    fn handle_optimize_result_key(&mut self, key: KeyEvent) -> Result<()> {
        if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
            self.mode = Mode::Normal;
            self.optimize_result = None;
        }
        Ok(())
    }

    /// Execute the currently selected context menu action.
    fn execute_context_menu_action(&mut self) -> Result<()> {
        let action = ContextMenuItem::ALL[self.context_menu_idx];

        // Guard: check if Restore is disabled (no backup)
        if matches!(action, ContextMenuItem::Restore) {
            if let Some(item) = self.shared_state.explorer.selected_item() {
                let path = std::path::Path::new(&item.path);
                if !has_backup(path) {
                    self.mode = Mode::Normal;
                    self.shared_state.status_message =
                        Some(format!("No backup exists for: {}", item.name.clone()));
                    return Ok(());
                }
            }
        }

        self.mode = Mode::Normal; // Close menu first

        match action {
            ContextMenuItem::Play => self.play_session()?,
            ContextMenuItem::Copy => self.copy_to_clipboard()?,
            ContextMenuItem::Optimize => self.optimize_session()?,
            ContextMenuItem::Analyze => self.analyze_session()?,
            ContextMenuItem::Restore => self.restore_session()?,
            ContextMenuItem::Delete => {
                if self.shared_state.explorer.selected_item().is_some() {
                    self.mode = Mode::ConfirmDelete;
                }
            }
            ContextMenuItem::AddMarker => self.add_marker()?,
        }
        Ok(())
    }
}

// --- Session actions ---

impl ListApp {
    /// Play the selected session with asciinema.
    fn play_session(&mut self) -> Result<()> {
        use crate::player;

        if let Some(item) = self.shared_state.explorer.selected_item() {
            let path = Path::new(&item.path);
            self.app.suspend()?;
            let result = player::play_session(path)?;
            self.app.resume()?;
            self.shared_state.status_message = Some(result.message());
        }
        Ok(())
    }

    /// Copy the selected session to the clipboard.
    fn copy_to_clipboard(&mut self) -> Result<()> {
        use crate::clipboard::copy_file_to_clipboard;

        if let Some(item) = self.shared_state.explorer.selected_item() {
            let path = Path::new(&item.path);
            let filename = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("recording");
            match copy_file_to_clipboard(path) {
                Ok(result) => {
                    self.shared_state.status_message = Some(result.message(filename));
                }
                Err(e) => {
                    self.shared_state.status_message = Some(format!("Copy failed: {}", e));
                }
            }
        }
        Ok(())
    }

    /// Delete the selected session.
    fn delete_session(&mut self) -> Result<()> {
        if let Some(item) = self.shared_state.explorer.selected_item() {
            let path = item.path.clone();
            let name = item.name.clone();

            if let Err(e) = std::fs::remove_file(&path) {
                self.shared_state.status_message = Some(format!("Failed to delete: {}", e));
            } else {
                let backup = backup_path_for(std::path::Path::new(&path));
                let backup_deleted = std::fs::remove_file(&backup).is_ok();
                self.shared_state.explorer.remove_item(&path);
                self.shared_state.status_message = Some(if backup_deleted {
                    format!("Deleted: {} (and backup)", name)
                } else {
                    format!("Deleted: {}", name)
                });
            }
        }
        Ok(())
    }

    /// Restore the selected session from its backup.
    fn restore_session(&mut self) -> Result<()> {
        if let Some(item) = self.shared_state.explorer.selected_item() {
            let path = std::path::Path::new(&item.path);
            let name = item.name.clone();
            let path_str = item.path.clone();

            match restore_from_backup(path) {
                Ok(()) => {
                    self.shared_state.preview_cache.invalidate(&path_str);
                    self.shared_state.explorer.update_item_metadata(&path_str);
                    self.shared_state.status_message =
                        Some(format!("Restored from backup: {}", name));
                }
                Err(e) => {
                    self.shared_state.status_message = Some(format!("Failed to restore: {}", e));
                }
            }
        }
        Ok(())
    }

    /// Optimize the selected session (apply silence removal).
    fn optimize_session(&mut self) -> Result<()> {
        if let Some(item) = self.shared_state.explorer.selected_item() {
            let path = std::path::Path::new(&item.path);
            let name = item.name.clone();
            let path_str = item.path.clone();

            let result = match apply_transforms(path) {
                Ok(result) => {
                    self.shared_state.preview_cache.invalidate(&path_str);
                    self.shared_state.explorer.update_item_metadata(&path_str);
                    Ok(result)
                }
                Err(e) => Err(e.to_string()),
            };

            self.optimize_result = Some(OptimizeResultState {
                filename: name,
                result,
            });
            self.mode = Mode::OptimizeResult;
        }
        Ok(())
    }

    /// Analyze the selected session using the analyze subcommand.
    fn analyze_session(&mut self) -> Result<()> {
        if let Some(item) = self.shared_state.explorer.selected_item() {
            let path = item.path.clone();
            let file_path = std::path::Path::new(&path);
            if let Err(e) = create_backup(file_path) {
                self.shared_state.status_message =
                    Some(format!("ERROR: Backup failed for {}: {}", path, e));
                return Ok(());
            }

            self.app.suspend()?;
            let status = std::process::Command::new(std::env::current_exe()?)
                .args(["analyze", &path, "--wait"])
                .status();
            self.app.resume()?;

            self.handle_analyze_result(status, file_path, &path)?;
        }
        Ok(())
    }

    /// Process the result of an analyze subprocess.
    fn handle_analyze_result(
        &mut self,
        status: std::io::Result<std::process::ExitStatus>,
        file_path: &std::path::Path,
        path: &str,
    ) -> Result<()> {
        match status {
            Ok(s) if s.success() => {
                if !file_path.exists() {
                    self.handle_renamed_file(file_path, path);
                } else {
                    let path_string = path.to_string();
                    self.shared_state.preview_cache.invalidate(&path_string);
                    self.shared_state.explorer.update_item_metadata(path);
                    self.shared_state.status_message = Some("Analysis complete".to_string());
                }
            }
            Ok(s) => {
                self.shared_state.status_message = Some(format!(
                    "Analyze exited with code {}",
                    s.code().unwrap_or(-1)
                ));
            }
            Err(e) => {
                self.shared_state.status_message = Some(format!("Failed to run analyze: {}", e));
            }
        }
        Ok(())
    }

    /// Handle the case where analyze renamed the file.
    fn handle_renamed_file(&mut self, file_path: &std::path::Path, path: &str) {
        let new_file = find_newest_cast_file(file_path);
        if let Some(new_path) = new_file {
            let new_path_str = new_path.to_string_lossy().to_string();
            self.shared_state.preview_cache.invalidate(&new_path_str);
            self.shared_state
                .explorer
                .update_item_path(path, &new_path_str);
            self.shared_state.status_message = Some(format!(
                "Analysis complete (renamed to {})",
                new_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
            ));
        } else {
            self.shared_state.explorer.remove_item(path);
            self.shared_state.status_message =
                Some("Analysis complete (file was renamed)".to_string());
        }
    }

    /// Add a marker to the selected session (placeholder).
    fn add_marker(&mut self) -> Result<()> {
        self.shared_state.status_message = Some("Marker feature coming soon!".to_string());
        Ok(())
    }
}

// --- Helper functions ---

/// Find the newest .cast file in the parent directory of `file_path`.
fn find_newest_cast_file(file_path: &std::path::Path) -> Option<std::path::PathBuf> {
    file_path.parent().and_then(|parent| {
        std::fs::read_dir(parent).ok().and_then(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("cast"))
                .max_by_key(|e| {
                    e.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                })
                .map(|e| e.path())
        })
    })
}

/// Map a shortcut character to its context menu index.
fn shortcut_to_menu_idx(c: char) -> Option<usize> {
    let target = match c {
        'p' => ContextMenuItem::Play,
        'c' => ContextMenuItem::Copy,
        't' => ContextMenuItem::Optimize,
        'a' => ContextMenuItem::Analyze,
        'r' => ContextMenuItem::Restore,
        'd' => ContextMenuItem::Delete,
        'm' => ContextMenuItem::AddMarker,
        _ => return None,
    };
    ContextMenuItem::ALL.iter().position(|i| *i == target)
}

/// Compute the status text for the given mode and shared state.
fn compute_status_text(mode: Mode, state: &SharedState) -> String {
    if let Some(msg) = &state.status_message {
        return msg.clone();
    }
    match mode {
        Mode::Search => format!("Search: {}_", state.search_input),
        Mode::AgentFilter => {
            let agent = &state.available_agents[state.agent_filter_idx];
            format!(
                "Filter by agent: {} (\u{2190}/\u{2192} to change, Enter to apply)",
                agent
            )
        }
        Mode::ConfirmDelete => "Delete this session? (y/n)".to_string(),
        Mode::Help | Mode::ContextMenu | Mode::OptimizeResult => String::new(),
        Mode::Normal => format_normal_status(&state.explorer),
    }
}

/// Format the status line for normal mode (shows active filters).
fn format_normal_status(explorer: &super::widgets::FileExplorer) -> String {
    let mut parts = vec![];
    if let Some(search) = explorer.search_filter() {
        parts.push(format!("search: \"{}\"", search));
    }
    if let Some(agent) = explorer.agent_filter() {
        parts.push(format!("agent: {}", agent));
    }
    if parts.is_empty() {
        format!("{} sessions", explorer.len())
    } else {
        format!("{} sessions ({})", explorer.len(), parts.join(", "))
    }
}

/// Get the footer text for the given mode.
fn compute_footer_text(mode: Mode) -> &'static str {
    match mode {
        Mode::Search => "Esc: cancel | Enter: apply search | Backspace: delete char",
        Mode::AgentFilter => "\u{2190}/\u{2192}: change agent | Enter: apply | Esc: cancel",
        Mode::ConfirmDelete => "y: confirm delete | n/Esc: cancel",
        Mode::Help => "Press any key to close help",
        Mode::ContextMenu => "\u{2191}\u{2193}: navigate | Enter: select | Esc: cancel",
        Mode::OptimizeResult => "Enter/Esc: dismiss",
        Mode::Normal => {
            "\u{2191}\u{2193}: navigate | Enter: menu | p: play | c: copy | t: optimize | a: analyze | d: delete | ?: help | q: quit"
        }
    }
}

/// Build the help text lines for the help modal.
fn build_help_text(theme: &crate::theme::Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(theme.text_secondary),
        )),
        Line::from(vec![
            Span::styled("  \u{2191}/\u{2193} j/k", Style::default().fg(theme.accent)),
            Span::raw("    Navigate"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/Dn", Style::default().fg(theme.accent)),
            Span::raw("    Page up/down"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End", Style::default().fg(theme.accent)),
            Span::raw("   First/last"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default().fg(theme.text_secondary),
        )),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(theme.accent)),
            Span::raw("       Context menu"),
        ]),
        Line::from(vec![
            Span::styled("  p", Style::default().fg(theme.accent)),
            Span::raw("           Play session"),
        ]),
        Line::from(vec![
            Span::styled("  c", Style::default().fg(theme.accent)),
            Span::raw("           Copy to clipboard"),
        ]),
        Line::from(vec![
            Span::styled("  t", Style::default().fg(theme.accent)),
            Span::raw("           Optimize (removes silence)"),
        ]),
        Line::from(vec![
            Span::styled("  a", Style::default().fg(theme.accent)),
            Span::raw("           Analyze session"),
        ]),
        Line::from(vec![
            Span::styled("  d", Style::default().fg(theme.accent)),
            Span::raw("           Delete session"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Filtering",
            Style::default().fg(theme.text_secondary),
        )),
        Line::from(vec![
            Span::styled("  /", Style::default().fg(theme.accent)),
            Span::raw("           Search by filename"),
        ]),
        Line::from(vec![
            Span::styled("  f", Style::default().fg(theme.accent)),
            Span::raw("           Filter by agent"),
        ]),
        Line::from(vec![
            Span::styled("  Esc", Style::default().fg(theme.accent)),
            Span::raw("         Clear filters"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(theme.accent)),
            Span::raw("           This help"),
        ]),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(theme.accent)),
            Span::raw("           Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(theme.text_secondary),
        )),
    ]
}

/// Build the context menu lines for the context menu modal.
fn build_context_menu_lines<'a>(
    theme: &crate::theme::Theme,
    selected_idx: usize,
    backup_exists: bool,
) -> Vec<Line<'a>> {
    let mut lines = vec![
        Line::from(Span::styled(
            "Actions",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (idx, item) in ContextMenuItem::ALL.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let is_restore = matches!(item, ContextMenuItem::Restore);
        let is_disabled = is_restore && !backup_exists;

        let label = if is_restore && !backup_exists {
            format!("  {} ({}) - no backup", item.label(), item.shortcut())
        } else {
            format!("  {} ({})", item.label(), item.shortcut())
        };

        let style = if is_selected {
            theme.highlight_style()
        } else if is_disabled {
            Style::default().fg(theme.text_secondary)
        } else {
            Style::default().fg(theme.text_primary)
        };

        let prefix = if is_selected { "> " } else { "  " };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, label),
            style,
        )));

        if matches!(item, ContextMenuItem::Optimize) {
            lines.push(Line::from(Span::styled(
                "       Removes silence from recording",
                Style::default().fg(theme.text_secondary),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "\u{2191}\u{2193}: navigate | Enter: select | Esc: cancel",
        Style::default().fg(theme.text_secondary),
    )));

    lines
}

/// Build the content for the optimize result modal.
fn build_optimize_result_content<'a>(
    theme: &crate::theme::Theme,
    result_state: &OptimizeResultState,
) -> (&'a str, ratatui::style::Color, Vec<Line<'a>>) {
    match &result_state.result {
        Ok(result) => {
            let lines = vec![
                Line::from(Span::styled(
                    format!("File: {}", result_state.filename),
                    Style::default().fg(theme.text_primary),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Original: ", Style::default().fg(theme.text_secondary)),
                    Span::styled(
                        format_duration(result.original_duration),
                        Style::default().fg(theme.text_primary),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("New:      ", Style::default().fg(theme.text_secondary)),
                    Span::styled(
                        format_duration(result.new_duration),
                        Style::default().fg(theme.text_primary),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Saved:    ", Style::default().fg(theme.text_secondary)),
                    Span::styled(
                        format!(
                            "{} ({:.0}%)",
                            format_duration(result.time_saved()),
                            result.percent_saved()
                        ),
                        Style::default()
                            .fg(theme.success)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Backup: ", Style::default().fg(theme.text_secondary)),
                    Span::styled(
                        if result.backup_created {
                            "Created"
                        } else {
                            "Using existing"
                        },
                        Style::default().fg(theme.text_primary),
                    ),
                ]),
            ];
            (" Optimization Complete ", theme.success, lines)
        }
        Err(error) => {
            let lines = vec![
                Line::from(Span::styled(
                    format!("File: {}", result_state.filename),
                    Style::default().fg(theme.text_primary),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Error:",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    error.to_string(),
                    Style::default().fg(theme.error),
                )),
            ];
            (" Optimization Failed ", theme.error, lines)
        }
    }
}

/// Format a duration in seconds as human-readable string.
///
/// Examples:
/// - 65.5 -> "1m 5s"
/// - 3661.0 -> "1h 1m 1s"
/// - 30.0 -> "30s"
fn format_duration(seconds: f64) -> String {
    let total_secs = seconds.round() as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
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
    fn context_menu_has_seven_items() {
        assert_eq!(ContextMenuItem::ALL.len(), 7);
    }

    #[test]
    fn context_menu_items_have_labels() {
        for item in ContextMenuItem::ALL {
            assert!(!item.label().is_empty());
        }
    }

    #[test]
    fn context_menu_items_have_shortcuts() {
        for item in ContextMenuItem::ALL {
            assert!(!item.shortcut().is_empty());
        }
    }

    #[test]
    fn context_menu_copy_label_and_shortcut() {
        assert_eq!(ContextMenuItem::Copy.label(), "Copy to clipboard");
        assert_eq!(ContextMenuItem::Copy.shortcut(), "c");
    }

    #[test]
    fn context_menu_item_order() {
        // Verify expected order: Play, Copy, Optimize, Analyze, Restore, Delete, AddMarker
        assert_eq!(ContextMenuItem::ALL[0], ContextMenuItem::Play);
        assert_eq!(ContextMenuItem::ALL[1], ContextMenuItem::Copy);
        assert_eq!(ContextMenuItem::ALL[2], ContextMenuItem::Optimize);
        assert_eq!(ContextMenuItem::ALL[3], ContextMenuItem::Analyze);
        assert_eq!(ContextMenuItem::ALL[4], ContextMenuItem::Restore);
        assert_eq!(ContextMenuItem::ALL[5], ContextMenuItem::Delete);
        assert_eq!(ContextMenuItem::ALL[6], ContextMenuItem::AddMarker);
    }

    #[test]
    fn context_menu_mode_is_context_menu() {
        assert_eq!(Mode::ContextMenu, Mode::ContextMenu);
        assert_ne!(Mode::ContextMenu, Mode::Normal);
    }

    #[test]
    fn format_duration_seconds_only() {
        assert_eq!(format_duration(30.0), "30s");
        assert_eq!(format_duration(0.0), "0s");
        assert_eq!(format_duration(59.4), "59s"); // rounds down
    }

    #[test]
    fn format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(60.0), "1m 0s");
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(3599.0), "59m 59s");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(3600.0), "1h 0m 0s");
        assert_eq!(format_duration(3661.0), "1h 1m 1s");
        assert_eq!(format_duration(7322.0), "2h 2m 2s");
    }

    #[test]
    fn optimize_result_mode_exists() {
        assert_eq!(Mode::OptimizeResult, Mode::OptimizeResult);
        assert_ne!(Mode::OptimizeResult, Mode::Normal);
    }
}
