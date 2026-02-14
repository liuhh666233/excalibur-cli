mod state;
mod ui;

use super::{Module, ModuleAction, ModuleId, ModuleMetadata};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};
use state::{InputMode, SettingsState};

#[derive(Debug)]
pub struct SettingsModule {
    state: SettingsState,
}

impl SettingsModule {
    pub fn new() -> Self {
        Self {
            state: SettingsState::new(),
        }
    }

    fn handle_select_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(ModuleAction::Exit),
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous();
                Ok(ModuleAction::None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_next();
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                if let Some(profile) = self.state.get_selected_profile() {
                    if profile.is_active {
                        self.state
                            .set_notification("Already the active profile".to_string());
                    } else {
                        self.state.input_mode = InputMode::ConfirmSwap;
                    }
                }
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_confirm_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::SelectProfile;
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                self.execute_swap(false)?;
                Ok(ModuleAction::None)
            }
            KeyCode::Char('b') => {
                self.state.input_mode = InputMode::BackupRename;
                self.state.rename_input.clear();
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_backup_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::ConfirmSwap;
                self.state.rename_input.clear();
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                self.execute_swap(true)?;
                Ok(ModuleAction::None)
            }
            KeyCode::Backspace => {
                self.state.rename_input.pop();
                Ok(ModuleAction::None)
            }
            KeyCode::Char(c) if c.is_alphanumeric() || c == '-' || c == '_' => {
                self.state.rename_input.push(c);
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn execute_swap(&mut self, backup: bool) -> Result<()> {
        let claude_dir = match dirs::home_dir() {
            Some(home) => home.join(".claude"),
            None => {
                self.state
                    .set_notification("Cannot find home directory".to_string());
                return Ok(());
            }
        };

        let active_path = claude_dir.join("settings.json");
        let selected_path = match self.state.get_selected_profile() {
            Some(p) => p.path.clone(),
            None => return Ok(()),
        };

        if backup {
            let suffix = self.state.rename_input.trim().to_string();
            if suffix.is_empty() {
                self.state
                    .set_notification("Name cannot be empty".to_string());
                return Ok(());
            }
            let backup_path = claude_dir.join(format!("settings_{}.json", suffix));
            if backup_path.exists() {
                self.state
                    .set_notification(format!("settings_{}.json already exists", suffix));
                return Ok(());
            }
            if active_path.exists() {
                if let Err(e) = std::fs::rename(&active_path, &backup_path) {
                    self.state.set_notification(format!("Backup failed: {}", e));
                    return Ok(());
                }
            }
        } else if active_path.exists() {
            if let Err(e) = std::fs::remove_file(&active_path) {
                self.state.set_notification(format!("Remove failed: {}", e));
                return Ok(());
            }
        }

        if let Err(e) = std::fs::copy(&selected_path, &active_path) {
            self.state.set_notification(format!("Copy failed: {}", e));
            return Ok(());
        }

        let profile_name = selected_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");
        self.state
            .set_notification(format!("Switched to {}", profile_name));

        self.state.input_mode = InputMode::SelectProfile;
        self.state.rename_input.clear();
        self.state.load_profiles();
        Ok(())
    }
}

impl Module for SettingsModule {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: ModuleId::Settings,
            name: "Claude Settings".to_string(),
            description: "Switch Claude Code settings profiles".to_string(),
            shortcut: Some('s'),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.state.input_mode = InputMode::SelectProfile;
        self.state.rename_input.clear();
        self.state.notification = None;
        self.state.load_profiles();
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ModuleAction> {
        match self.state.input_mode {
            InputMode::SelectProfile => self.handle_select_mode(key_event),
            InputMode::ConfirmSwap => self.handle_confirm_mode(key_event),
            InputMode::BackupRename => self.handle_backup_mode(key_event),
        }
    }

    fn update(&mut self) -> Result<()> {
        self.state.clear_expired_notifications();
        Ok(())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        ui::render(&self.state, area, buf);
    }

    fn cleanup(&mut self) -> Result<()> {
        self.state.input_mode = InputMode::SelectProfile;
        self.state.rename_input.clear();
        self.state.notification = None;
        self.state.profiles.clear();
        self.state.preview_content.clear();
        Ok(())
    }
}
