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
    state: HistoryState,  // 直接存储，不用 Option
    clipboard: ClipboardManager,
}

impl HistoryModule {
    pub fn new() -> Self {
        // 预加载：在创建时就解析历史文件
        let parser = FishHistoryParser::new().ok();

        let (commands, stats) = if let Some(parser) = parser {
            // 尝试解析文件
            match (parser.parse(), parser.get_stats()) {
                (Ok(commands), Ok(stats)) => (commands, stats),
                _ => (Vec::new(), std::collections::HashMap::new()),
            }
        } else {
            // 解析器创建失败，使用空数据
            (Vec::new(), std::collections::HashMap::new())
        };

        Self {
            state: HistoryState::new(commands, stats),
            clipboard: ClipboardManager::new(),
        }
    }

    /// Handle key events in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        use ratatui::crossterm::event::KeyModifiers;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                return Ok(ModuleAction::Exit);
            }
            KeyCode::Enter => {
                // Output selected command to stdout for Fish integration
                if let Some(cmd) = self.state.get_selected_command() {
                    return Ok(ModuleAction::Output(cmd.cmd.clone()));
                }
            }
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+O: Output and execute immediately
                if let Some(cmd) = self.state.get_selected_command() {
                    return Ok(ModuleAction::OutputAndExecute(cmd.cmd.clone()));
                }
            }
            KeyCode::Char('/') => {
                self.state.input_mode = InputMode::Search;
                self.state.search_query.clear();
            }
            KeyCode::Char('s') => {
                self.state.cycle_sort_mode();
            }
            KeyCode::Char('y') => {
                if let Some(cmd) = self.state.get_selected_command() {
                    match self.clipboard.copy(&cmd.cmd) {
                        Ok(_) => {
                            self.state.set_notification(format!("Copied: {}", cmd.cmd));
                        }
                        Err(e) => {
                            self.state.set_notification(format!("Failed to copy: {}", e));
                        }
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_next();
            }
            KeyCode::PageUp => {
                self.state.page_up();
            }
            KeyCode::PageDown => {
                self.state.page_down();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.select_first();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.state.select_last();
            }
            _ => {}
        }
        Ok(ModuleAction::None)
    }

    /// Handle key events in search mode
    fn handle_search_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::Normal;
                self.state.search_query.clear();
                self.state.apply_filters();
            }
            KeyCode::Enter => {
                self.state.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
                self.state.apply_filters();
            }
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
                self.state.apply_filters();
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
        // 数据已在 new() 中预加载，这里只重置 UI 状态
        self.state.selected_index = 0;
        self.state.search_query.clear();
        self.state.input_mode = InputMode::Normal;
        self.state.notification = None;

        // 重新应用过滤和排序（以防数据已更新）
        self.state.apply_filters();

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModuleAction> {
        match self.state.input_mode {
            InputMode::Normal => self.handle_normal_mode(key_event),
            InputMode::Search => self.handle_search_mode(key_event),
        }
    }

    fn update(&mut self) -> Result<()> {
        // 清理过期通知
        self.state.clear_expired_notifications();
        Ok(())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        ui::render(&self.state, area, buf);
    }

    fn cleanup(&mut self) -> Result<()> {
        // 数据保留在内存中，只重置 UI 状态
        self.state.selected_index = 0;
        self.state.search_query.clear();
        self.state.input_mode = InputMode::Normal;
        self.state.notification = None;
        Ok(())
    }
}
