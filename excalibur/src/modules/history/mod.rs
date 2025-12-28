mod clipboard;
mod parser;
mod state;
mod ui;

use super::{Module, ModuleAction, ModuleId, ModuleMetadata};
use clipboard::ClipboardManager;
use color_eyre::Result;
use parser::FishHistoryParser;
use ratatui::{buffer::Buffer, crossterm::event::{KeyCode, KeyEvent}, layout::Rect};
use state::{HistoryState, InputMode};

#[derive(Debug)]
pub struct HistoryModule {
    state: Option<HistoryState>,
    parser: FishHistoryParser,
    clipboard: ClipboardManager,
}

impl HistoryModule {
    pub fn new() -> Self {
        Self {
            state: None,
            parser: FishHistoryParser::new().unwrap_or_else(|_| {
                // Fallback if parser creation fails
                FishHistoryParser::new().unwrap()
            }),
            clipboard: ClipboardManager::new(),
        }
    }

    /// Handle key events in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        let Some(ref mut state) = self.state else {
            return Ok(ModuleAction::None);
        };

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                return Ok(ModuleAction::Exit);
            }
            KeyCode::Char('/') => {
                state.input_mode = InputMode::Search;
                state.search_query.clear();
            }
            KeyCode::Char('s') => {
                state.cycle_sort_mode();
            }
            KeyCode::Char('y') => {
                if let Some(cmd) = state.get_selected_command() {
                    match self.clipboard.copy(&cmd.cmd) {
                        Ok(_) => {
                            state.set_notification(format!("Copied: {}", cmd.cmd));
                        }
                        Err(e) => {
                            state.set_notification(format!("Failed to copy: {}", e));
                        }
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.select_next();
            }
            KeyCode::PageUp => {
                state.page_up();
            }
            KeyCode::PageDown => {
                state.page_down();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                state.select_first();
            }
            KeyCode::End | KeyCode::Char('G') => {
                state.select_last();
            }
            _ => {}
        }
        Ok(ModuleAction::None)
    }

    /// Handle key events in search mode
    fn handle_search_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        let Some(ref mut state) = self.state else {
            return Ok(ModuleAction::None);
        };

        match key.code {
            KeyCode::Esc => {
                state.input_mode = InputMode::Normal;
                state.search_query.clear();
                state.apply_filters();
            }
            KeyCode::Enter => {
                state.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                state.search_query.pop();
                state.apply_filters();
            }
            KeyCode::Char(c) => {
                state.search_query.push(c);
                state.apply_filters();
            }
            _ => {}
        }
        Ok(ModuleAction::None)
    }
}

impl Module for HistoryModule {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: ModuleId::History,
            name: "Command History".to_string(),
            description: "Browse and search shell command history".to_string(),
            shortcut: Some('h'),
        }
    }

    fn init(&mut self) -> Result<()> {
        // Parse history file
        let commands = self.parser.parse()?;
        let stats = self.parser.get_stats()?;

        // Initialize state
        self.state = Some(HistoryState::new(commands, stats));

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModuleAction> {
        if let Some(ref state) = self.state {
            match state.input_mode {
                InputMode::Normal => self.handle_normal_mode(key_event),
                InputMode::Search => self.handle_search_mode(key_event),
            }
        } else {
            Ok(ModuleAction::None)
        }
    }

    fn update(&mut self) -> Result<()> {
        // Clear expired notifications
        if let Some(ref mut state) = self.state {
            state.clear_expired_notifications();
        }
        Ok(())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref state) = self.state {
            ui::render(state, area, buf);
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        self.state = None;
        Ok(())
    }
}
