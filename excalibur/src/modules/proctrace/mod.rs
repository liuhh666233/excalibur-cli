mod collector;
mod state;
mod ui;

use crate::modules::{Module, ModuleAction, ModuleId, ModuleMetadata};
use color_eyre::Result;
use collector::ProcessCollector;
use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};
use state::{InputMode, ProcessTracerState};
use std::time::Duration;

/// Process Tracer module
#[derive(Debug)]
pub struct ProcessTracerModule {
    state: ProcessTracerState,
    collector: ProcessCollector,
}

impl ProcessTracerModule {
    pub fn new() -> Self {
        let mut module = Self {
            state: ProcessTracerState::new(),
            collector: ProcessCollector::new(),
        };

        // Pre-load process data
        if let Ok(processes) = module.collector.collect() {
            module.state.update_processes(processes);
        }

        module
    }

    /// Force refresh process list
    fn refresh_processes(&mut self) -> Result<()> {
        let processes = self.collector.collect()?;
        self.state.update_processes(processes);
        self.state
            .set_notification("Refreshed process list".to_string());
        Ok(())
    }

    /// Handle key events in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        use ratatui::crossterm::event::KeyCode;

        match key.code {
            // Exit
            KeyCode::Esc | KeyCode::Char('q') => Ok(ModuleAction::Exit),

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous();
                Ok(ModuleAction::None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_next();
                Ok(ModuleAction::None)
            }
            KeyCode::PageUp => {
                self.state.page_up();
                Ok(ModuleAction::None)
            }
            KeyCode::PageDown => {
                self.state.page_down();
                Ok(ModuleAction::None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.select_first();
                Ok(ModuleAction::None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.state.select_last();
                Ok(ModuleAction::None)
            }

            // Sort
            KeyCode::Char('s') => {
                self.state.cycle_sort_mode();
                Ok(ModuleAction::None)
            }

            // Refresh
            KeyCode::Char('r') => {
                self.refresh_processes()?;
                Ok(ModuleAction::None)
            }

            // Enter search mode
            KeyCode::Char('/') => {
                self.state.input_mode = InputMode::Search;
                self.state.search_query.clear();
                Ok(ModuleAction::None)
            }

            _ => Ok(ModuleAction::None),
        }
    }

    /// Handle key events in search mode
    fn handle_search_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        use ratatui::crossterm::event::KeyCode;

        match key.code {
            // Exit search mode and clear search
            KeyCode::Esc => {
                self.state.input_mode = InputMode::Normal;
                self.state.search_query.clear();
                self.state.apply_filters();
                Ok(ModuleAction::None)
            }

            // Apply search
            KeyCode::Enter => {
                self.state.input_mode = InputMode::Normal;
                Ok(ModuleAction::None)
            }

            // Input character
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
                self.state.apply_filters();
                Ok(ModuleAction::None)
            }

            // Backspace
            KeyCode::Backspace => {
                self.state.search_query.pop();
                self.state.apply_filters();
                Ok(ModuleAction::None)
            }

            _ => Ok(ModuleAction::None),
        }
    }
}

impl Module for ProcessTracerModule {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: ModuleId::ProcessTracer,
            name: "Process Tracer".to_string(),
            description: "Inspect running processes and their supervisors".to_string(),
            shortcut: Some('p'),
        }
    }

    fn init(&mut self) -> Result<()> {
        // Reset UI state
        self.state.selected_index = 0;
        self.state.search_query.clear();
        self.state.input_mode = InputMode::Normal;
        self.state.notification = None;

        // Refresh process list
        self.refresh_processes()?;

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModuleAction> {
        match self.state.input_mode {
            InputMode::Normal => self.handle_normal_mode(key_event),
            InputMode::Search => self.handle_search_mode(key_event),
        }
    }

    fn update(&mut self) -> Result<()> {
        // Clear expired notifications
        self.state.clear_expired_notifications();

        // Auto-refresh every 1 second
        if self.state.last_update.elapsed() >= Duration::from_secs(1) {
            if let Ok(processes) = self.collector.collect() {
                self.state.update_processes(processes);
                self.state.last_update = std::time::Instant::now();
            }
        }

        Ok(())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        ui::render(&self.state, area, buf);
    }

    fn cleanup(&mut self) -> Result<()> {
        // Reset UI state (keep data in memory for fast re-entry)
        self.state.selected_index = 0;
        self.state.search_query.clear();
        self.state.input_mode = InputMode::Normal;
        self.state.notification = None;

        Ok(())
    }
}
