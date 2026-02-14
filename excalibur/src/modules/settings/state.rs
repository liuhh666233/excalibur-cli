use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    SelectProfile,
    ConfirmSwap,
    BackupRename,
}

#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_active: bool,
}

#[derive(Debug)]
pub struct SettingsState {
    pub profiles: Vec<ProfileEntry>,
    pub selected_index: usize,
    pub preview_content: String,
    pub rename_input: String,
    pub input_mode: InputMode,
    pub notification: Option<(String, Instant)>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            selected_index: 0,
            preview_content: String::new(),
            rename_input: String::new(),
            input_mode: InputMode::SelectProfile,
            notification: None,
        }
    }

    pub fn load_profiles(&mut self) {
        self.profiles.clear();
        let claude_dir = match dirs::home_dir() {
            Some(home) => home.join(".claude"),
            None => return,
        };

        let entries = match std::fs::read_dir(&claude_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if !name.starts_with("settings") || !name.ends_with(".json") {
                continue;
            }
            let is_active = name == "settings.json";
            let display = if is_active {
                "[active] settings.json".to_string()
            } else {
                name.clone()
            };
            self.profiles.push(ProfileEntry {
                name: display,
                path,
                is_active,
            });
        }

        // Sort: active first, then alphabetical
        self.profiles
            .sort_by(|a, b| b.is_active.cmp(&a.is_active).then(a.name.cmp(&b.name)));

        self.selected_index = 0;
        self.update_preview();
    }

    pub fn update_preview(&mut self) {
        self.preview_content = match self.get_selected_profile() {
            Some(p) => match std::fs::read_to_string(&p.path) {
                Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
                    Ok(val) => serde_json::to_string_pretty(&val).unwrap_or(raw),
                    Err(_) => raw,
                },
                Err(e) => format!("Error reading file: {}", e),
            },
            None => String::new(),
        };
    }

    pub fn select_next(&mut self) {
        if !self.profiles.is_empty() && self.selected_index < self.profiles.len() - 1 {
            self.selected_index += 1;
            self.update_preview();
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_preview();
        }
    }

    pub fn get_selected_profile(&self) -> Option<&ProfileEntry> {
        self.profiles.get(self.selected_index)
    }

    pub fn set_notification(&mut self, message: String) {
        self.notification = Some((message, Instant::now()));
    }

    pub fn clear_expired_notifications(&mut self) {
        if let Some((_, timestamp)) = &self.notification {
            if timestamp.elapsed().as_secs() > 3 {
                self.notification = None;
            }
        }
    }
}
