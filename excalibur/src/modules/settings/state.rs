use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    SelectProfile,
    ConfirmSwap,
    BackupRename,
    InputCopyName,
    InputRenameName,
    ConfirmDelete,
    EditKeys,
    EditValue,
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
    pub rename_cursor: usize,
    pub input_mode: InputMode,
    pub notification: Option<(String, Instant)>,
    // JSON key-value editor
    pub edit_entries: Vec<(String, String)>,
    pub edit_index: usize,
    pub edit_value_buf: String,
    pub edit_cursor: usize,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            selected_index: 0,
            preview_content: String::new(),
            rename_input: String::new(),
            rename_cursor: 0,
            input_mode: InputMode::SelectProfile,
            notification: None,
            edit_entries: Vec::new(),
            edit_index: 0,
            edit_value_buf: String::new(),
            edit_cursor: 0,
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

    /// Pre-fill rename_input with current filename and position cursor before ".json"
    pub fn init_rename_input(&mut self) {
        if let Some(profile) = self.profiles.get(self.selected_index) {
            let name = profile
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("settings.json")
                .to_string();
            self.rename_input = name;
            // Place cursor before ".json"
            let cursor = self.rename_input.chars().count().saturating_sub(5); // len(".json") = 5
            self.rename_cursor = cursor;
        }
    }

    /// Insert a char at rename_cursor position
    pub fn rename_insert_char(&mut self, c: char) {
        let byte_idx = self
            .rename_input
            .char_indices()
            .nth(self.rename_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.rename_input.len());
        self.rename_input.insert(byte_idx, c);
        self.rename_cursor += 1;
    }

    /// Delete char before rename_cursor
    pub fn rename_backspace(&mut self) {
        if self.rename_cursor > 0 {
            let byte_idx = self
                .rename_input
                .char_indices()
                .nth(self.rename_cursor - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next_byte = self
                .rename_input
                .char_indices()
                .nth(self.rename_cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.rename_input.len());
            self.rename_input.replace_range(byte_idx..next_byte, "");
            self.rename_cursor -= 1;
        }
    }

    pub fn rename_cursor_left(&mut self) {
        if self.rename_cursor > 0 {
            self.rename_cursor -= 1;
        }
    }

    pub fn rename_cursor_right(&mut self) {
        let char_count = self.rename_input.chars().count();
        if self.rename_cursor < char_count {
            self.rename_cursor += 1;
        }
    }

    pub fn parse_json_entries(&mut self) {
        self.edit_entries.clear();
        let Some(profile) = self.profiles.get(self.selected_index) else {
            return;
        };
        let raw = match std::fs::read_to_string(&profile.path) {
            Ok(r) => r,
            Err(_) => return,
        };
        let obj: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(&raw) {
            Ok(serde_json::Value::Object(m)) => m,
            _ => return,
        };
        Self::flatten_json("", &serde_json::Value::Object(obj), &mut self.edit_entries);
        self.edit_index = 0;
    }

    fn flatten_json(prefix: &str, value: &serde_json::Value, entries: &mut Vec<(String, String)>) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    Self::flatten_json(&key, v, entries);
                }
            }
            serde_json::Value::String(s) => {
                entries.push((prefix.to_string(), s.clone()));
            }
            other => {
                entries.push((prefix.to_string(), other.to_string()));
            }
        }
    }

    pub fn save_edit(&mut self) -> Result<(), String> {
        let profile = match self.profiles.get(self.selected_index) {
            Some(p) => p,
            None => return Err("No profile selected".to_string()),
        };
        let raw = std::fs::read_to_string(&profile.path)
            .map_err(|e| format!("Read failed: {}", e))?;
        let mut root: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|_| "Invalid JSON".to_string())?;

        for (dotted_key, v) in &self.edit_entries {
            let parts: Vec<&str> = dotted_key.split('.').collect();
            // Parse value: try bool/number/null first, then fall back to string
            let val = if v == "true" {
                serde_json::Value::Bool(true)
            } else if v == "false" {
                serde_json::Value::Bool(false)
            } else if v == "null" {
                serde_json::Value::Null
            } else if let Ok(n) = v.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else if let Ok(n) = v.parse::<f64>() {
                serde_json::json!(n)
            } else {
                // Plain string — no quotes needed, no escaping
                serde_json::Value::String(v.clone())
            };

            // Navigate to the nested location and set value
            let mut target = &mut root;
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    target[*part] = val.clone();
                } else {
                    if target.get(*part).is_none() {
                        target[*part] = serde_json::Value::Object(serde_json::Map::new());
                    }
                    target = &mut target[*part];
                }
            }
        }

        let json = serde_json::to_string_pretty(&root)
            .map_err(|e| format!("Serialize failed: {}", e))?;
        std::fs::write(&profile.path, json)
            .map_err(|e| format!("Write failed: {}", e))?;
        self.update_preview();
        Ok(())
    }

    pub fn edit_select_next(&mut self) {
        if !self.edit_entries.is_empty() && self.edit_index < self.edit_entries.len() - 1 {
            self.edit_index += 1;
        }
    }

    pub fn edit_select_previous(&mut self) {
        if self.edit_index > 0 {
            self.edit_index -= 1;
        }
    }
}
