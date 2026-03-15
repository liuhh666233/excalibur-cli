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
            KeyCode::Char('c') => {
                if self.state.get_selected_profile().is_some() {
                    self.state.init_rename_input();
                    self.state.input_mode = InputMode::InputCopyName;
                }
                Ok(ModuleAction::None)
            }
            KeyCode::Char('r') => {
                if let Some(profile) = self.state.get_selected_profile() {
                    if profile.is_active {
                        self.state
                            .set_notification("Cannot rename active profile".to_string());
                    } else {
                        self.state.init_rename_input();
                        self.state.input_mode = InputMode::InputRenameName;
                    }
                }
                Ok(ModuleAction::None)
            }
            KeyCode::Char('d') => {
                if let Some(profile) = self.state.get_selected_profile() {
                    if profile.is_active {
                        self.state
                            .set_notification("Cannot delete active profile".to_string());
                    } else {
                        self.state.input_mode = InputMode::ConfirmDelete;
                    }
                }
                Ok(ModuleAction::None)
            }
            KeyCode::Char('e') => {
                if self.state.get_selected_profile().is_some() {
                    self.state.parse_json_entries();
                    if self.state.edit_entries.is_empty() {
                        self.state
                            .set_notification("No editable keys found".to_string());
                    } else {
                        self.state.input_mode = InputMode::EditKeys;
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
                self.state.init_rename_input();
                self.state.input_mode = InputMode::BackupRename;
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
                self.state.rename_backspace();
                Ok(ModuleAction::None)
            }
            KeyCode::Left => {
                self.state.rename_cursor_left();
                Ok(ModuleAction::None)
            }
            KeyCode::Right => {
                self.state.rename_cursor_right();
                Ok(ModuleAction::None)
            }
            KeyCode::Char(c) => {
                self.state.rename_insert_char(c);
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_copy_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::SelectProfile;
                self.state.rename_input.clear();
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                self.execute_copy()?;
                Ok(ModuleAction::None)
            }
            KeyCode::Backspace => {
                self.state.rename_backspace();
                Ok(ModuleAction::None)
            }
            KeyCode::Left => {
                self.state.rename_cursor_left();
                Ok(ModuleAction::None)
            }
            KeyCode::Right => {
                self.state.rename_cursor_right();
                Ok(ModuleAction::None)
            }
            KeyCode::Char(c) => {
                self.state.rename_insert_char(c);
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_rename_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::SelectProfile;
                self.state.rename_input.clear();
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                self.execute_rename()?;
                Ok(ModuleAction::None)
            }
            KeyCode::Backspace => {
                self.state.rename_backspace();
                Ok(ModuleAction::None)
            }
            KeyCode::Left => {
                self.state.rename_cursor_left();
                Ok(ModuleAction::None)
            }
            KeyCode::Right => {
                self.state.rename_cursor_right();
                Ok(ModuleAction::None)
            }
            KeyCode::Char(c) => {
                self.state.rename_insert_char(c);
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_confirm_delete(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::SelectProfile;
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                self.execute_delete()?;
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_edit_keys(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::SelectProfile;
                self.state.edit_entries.clear();
                Ok(ModuleAction::None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.edit_select_previous();
                Ok(ModuleAction::None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.edit_select_next();
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                if let Some((_, v)) = self.state.edit_entries.get(self.state.edit_index) {
                    self.state.edit_value_buf = v.clone();
                    self.state.edit_cursor = self.state.edit_value_buf.len();
                    self.state.input_mode = InputMode::EditValue;
                }
                Ok(ModuleAction::None)
            }
            _ => Ok(ModuleAction::None),
        }
    }

    fn handle_edit_value(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        use ratatui::crossterm::event::KeyModifiers;

        match key.code {
            KeyCode::Esc => {
                self.state.input_mode = InputMode::EditKeys;
                self.state.edit_value_buf.clear();
                Ok(ModuleAction::None)
            }
            KeyCode::Enter => {
                // Save the edited value back
                let idx = self.state.edit_index;
                if idx < self.state.edit_entries.len() {
                    self.state.edit_entries[idx].1 = self.state.edit_value_buf.clone();
                    match self.state.save_edit() {
                        Ok(()) => self.state.set_notification("Saved".to_string()),
                        Err(e) => self.state.set_notification(e),
                    }
                }
                self.state.input_mode = InputMode::EditKeys;
                self.state.edit_value_buf.clear();
                Ok(ModuleAction::None)
            }
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Backspace: clear entire value
                self.state.edit_value_buf.clear();
                self.state.edit_cursor = 0;
                Ok(ModuleAction::None)
            }
            KeyCode::Backspace => {
                if self.state.edit_cursor > 0 {
                    let byte_idx = self
                        .state
                        .edit_value_buf
                        .char_indices()
                        .nth(self.state.edit_cursor - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let next_byte = self
                        .state
                        .edit_value_buf
                        .char_indices()
                        .nth(self.state.edit_cursor)
                        .map(|(i, _)| i)
                        .unwrap_or(self.state.edit_value_buf.len());
                    self.state.edit_value_buf.replace_range(byte_idx..next_byte, "");
                    self.state.edit_cursor -= 1;
                }
                Ok(ModuleAction::None)
            }
            KeyCode::Left => {
                if self.state.edit_cursor > 0 {
                    self.state.edit_cursor -= 1;
                }
                Ok(ModuleAction::None)
            }
            KeyCode::Right => {
                let char_count = self.state.edit_value_buf.chars().count();
                if self.state.edit_cursor < char_count {
                    self.state.edit_cursor += 1;
                }
                Ok(ModuleAction::None)
            }
            KeyCode::Char(c) => {
                let byte_idx = self
                    .state
                    .edit_value_buf
                    .char_indices()
                    .nth(self.state.edit_cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(self.state.edit_value_buf.len());
                self.state.edit_value_buf.insert(byte_idx, c);
                self.state.edit_cursor += 1;
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
            let filename = self.state.rename_input.trim().to_string();
            if filename.is_empty() {
                self.state
                    .set_notification("Name cannot be empty".to_string());
                return Ok(());
            }
            let backup_path = claude_dir.join(&filename);
            if backup_path.exists() {
                self.state
                    .set_notification(format!("{} already exists", filename));
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

    fn execute_copy(&mut self) -> Result<()> {
        let claude_dir = match dirs::home_dir() {
            Some(home) => home.join(".claude"),
            None => {
                self.state
                    .set_notification("Cannot find home directory".to_string());
                return Ok(());
            }
        };

        let filename = self.state.rename_input.trim().to_string();
        if filename.is_empty() {
            self.state
                .set_notification("Name cannot be empty".to_string());
            return Ok(());
        }

        let target_path = claude_dir.join(&filename);
        if target_path.exists() {
            self.state
                .set_notification(format!("{} already exists", filename));
            return Ok(());
        }

        let selected_path = match self.state.get_selected_profile() {
            Some(p) => p.path.clone(),
            None => return Ok(()),
        };

        if let Err(e) = std::fs::copy(&selected_path, &target_path) {
            self.state.set_notification(format!("Copy failed: {}", e));
            return Ok(());
        }

        self.state
            .set_notification(format!("Copied to {}", filename));
        self.state.input_mode = InputMode::SelectProfile;
        self.state.rename_input.clear();
        self.state.load_profiles();
        Ok(())
    }

    fn execute_rename(&mut self) -> Result<()> {
        let claude_dir = match dirs::home_dir() {
            Some(home) => home.join(".claude"),
            None => {
                self.state
                    .set_notification("Cannot find home directory".to_string());
                return Ok(());
            }
        };

        let filename = self.state.rename_input.trim().to_string();
        if filename.is_empty() {
            self.state
                .set_notification("Name cannot be empty".to_string());
            return Ok(());
        }

        let target_path = claude_dir.join(&filename);
        if target_path.exists() {
            self.state
                .set_notification(format!("{} already exists", filename));
            return Ok(());
        }

        let selected_path = match self.state.get_selected_profile() {
            Some(p) => p.path.clone(),
            None => return Ok(()),
        };

        if let Err(e) = std::fs::rename(&selected_path, &target_path) {
            self.state
                .set_notification(format!("Rename failed: {}", e));
            return Ok(());
        }

        self.state
            .set_notification(format!("Renamed to {}", filename));
        self.state.input_mode = InputMode::SelectProfile;
        self.state.rename_input.clear();
        self.state.load_profiles();
        Ok(())
    }

    fn execute_delete(&mut self) -> Result<()> {
        let selected_path = match self.state.get_selected_profile() {
            Some(p) => p.path.clone(),
            None => return Ok(()),
        };

        let name = selected_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        if let Err(e) = std::fs::remove_file(&selected_path) {
            self.state
                .set_notification(format!("Delete failed: {}", e));
            return Ok(());
        }

        self.state
            .set_notification(format!("Deleted {}", name));
        self.state.input_mode = InputMode::SelectProfile;
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
            InputMode::InputCopyName => self.handle_copy_mode(key_event),
            InputMode::InputRenameName => self.handle_rename_mode(key_event),
            InputMode::ConfirmDelete => self.handle_confirm_delete(key_event),
            InputMode::EditKeys => self.handle_edit_keys(key_event),
            InputMode::EditValue => self.handle_edit_value(key_event),
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
