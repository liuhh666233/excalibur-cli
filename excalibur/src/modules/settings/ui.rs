use super::state::{InputMode, SettingsState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget, Wrap},
};

pub fn render(state: &SettingsState, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let title = format!(" Claude Settings | {} profiles ", state.profiles.len());
    Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Cyan))
        .render(chunks[0], buf);

    // Main content: list + preview
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_profile_list(state, main_chunks[0], buf);
    match state.input_mode {
        InputMode::EditKeys | InputMode::EditValue => {
            render_edit_panel(state, main_chunks[1], buf);
        }
        _ => {
            render_preview(state, main_chunks[1], buf);
        }
    }

    // Action bar
    let action_bar_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let (action_spans, style) = match state.input_mode {
        InputMode::ConfirmSwap => (
            vec![Span::styled(
                " Switch to this profile? [Enter] Switch  [b] Backup & Switch  [Esc] Cancel",
                action_bar_style,
            )],
            action_bar_style,
        ),
        InputMode::BackupRename => {
            let (before, after) = split_at_cursor(&state.rename_input, state.rename_cursor);
            (
                vec![
                    Span::styled(" Backup as: ", action_bar_style),
                    Span::styled(before, action_bar_style),
                    Span::styled("█", Style::default().fg(Color::White).bg(Color::Yellow)),
                    Span::styled(after, action_bar_style),
                ],
                action_bar_style,
            )
        }
        InputMode::InputCopyName => {
            let (before, after) = split_at_cursor(&state.rename_input, state.rename_cursor);
            (
                vec![
                    Span::styled(" Copy as: ", action_bar_style),
                    Span::styled(before, action_bar_style),
                    Span::styled("█", Style::default().fg(Color::White).bg(Color::Yellow)),
                    Span::styled(after, action_bar_style),
                ],
                action_bar_style,
            )
        }
        InputMode::InputRenameName => {
            let (before, after) = split_at_cursor(&state.rename_input, state.rename_cursor);
            (
                vec![
                    Span::styled(" Rename to: ", action_bar_style),
                    Span::styled(before, action_bar_style),
                    Span::styled("█", Style::default().fg(Color::White).bg(Color::Yellow)),
                    Span::styled(after, action_bar_style),
                ],
                action_bar_style,
            )
        }
        InputMode::ConfirmDelete => {
            let name = state
                .get_selected_profile()
                .map(|p| p.name.as_str())
                .unwrap_or("?");
            let delete_style = Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD);
            (
                vec![Span::styled(
                    format!(" Delete {}? [Enter] Confirm  [Esc] Cancel", name),
                    delete_style,
                )],
                delete_style,
            )
        }
        InputMode::SelectProfile | InputMode::EditKeys | InputMode::EditValue => {
            (vec![], Style::default().fg(Color::DarkGray))
        }
    };
    Paragraph::new(Line::from(action_spans))
        .block(Block::bordered().border_type(BorderType::Rounded))
        .style(style)
        .render(chunks[2], buf);

    // Status bar
    let help = match state.input_mode {
        InputMode::SelectProfile => {
            "[Enter] Switch  [c] Copy  [r] Rename  [d] Delete  [e] Edit  [j/k] Navigate  [Esc] Exit"
        }
        InputMode::ConfirmSwap => {
            "[Enter] Switch (delete old)  [b] Backup first  [Esc] Cancel"
        }
        InputMode::BackupRename | InputMode::InputCopyName | InputMode::InputRenameName => {
            "[Enter] Confirm  [Esc] Cancel"
        }
        InputMode::ConfirmDelete => "[Enter] Delete  [Esc] Cancel",
        InputMode::EditKeys => "[Enter] Edit value  [j/k] Navigate  [Esc] Back",
        InputMode::EditValue => "[Enter] Save  [←/→] Move cursor  [Ctrl+⌫] Clear  [Esc] Cancel",
    };
    Paragraph::new(help)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        )
        .centered()
        .style(Style::default().fg(Color::Gray))
        .render(chunks[3], buf);

    // Notification overlay
    if let Some((ref msg, _)) = state.notification {
        render_notification(msg, area, buf);
    }
}

fn render_profile_list(state: &SettingsState, area: Rect, buf: &mut Buffer) {
    if state.profiles.is_empty() {
        Paragraph::new("No profiles found in ~/.claude/")
            .block(
                Block::bordered()
                    .title(" Profiles ")
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::DarkGray))
            .render(area, buf);
        return;
    }

    let items: Vec<ListItem> = state
        .profiles
        .iter()
        .enumerate()
        .map(|(i, profile)| {
            let selected = i == state.selected_index;
            let symbol = if selected { "▶ " } else { "  " };
            let name_style = if profile.is_active {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let line = Line::from(vec![
                Span::raw(symbol),
                Span::styled(&profile.name, name_style),
            ]);
            ListItem::new(line).style(if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    Widget::render(
        List::new(items).block(
            Block::bordered()
                .title(" Profiles ")
                .border_type(BorderType::Rounded),
        ),
        area,
        buf,
    );
}

fn render_preview(state: &SettingsState, area: Rect, buf: &mut Buffer) {
    let title = match state.get_selected_profile() {
        Some(p) => format!(
            " Preview: {} ",
            p.path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ),
        None => " Preview ".to_string(),
    };

    Paragraph::new(state.preview_content.as_str())
        .block(
            Block::bordered()
                .title(title)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false })
        .render(area, buf);
}

fn render_edit_panel(state: &SettingsState, area: Rect, buf: &mut Buffer) {
    let title = match state.get_selected_profile() {
        Some(p) => format!(
            " Edit: {} ",
            p.path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ),
        None => " Edit ".to_string(),
    };

    if state.edit_entries.is_empty() {
        Paragraph::new("No keys to edit")
            .block(
                Block::bordered()
                    .title(title)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::DarkGray))
            .render(area, buf);
        return;
    }

    let items: Vec<ListItem> = state
        .edit_entries
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            let selected = i == state.edit_index;
            let is_editing = selected && state.input_mode == InputMode::EditValue;

            let symbol = if selected { "▶ " } else { "  " };
            let key_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);

            if is_editing {
                // Show editable value with cursor
                let buf_str = &state.edit_value_buf;
                let cursor = state.edit_cursor;
                let (before, after): (String, String) = {
                    let chars: Vec<char> = buf_str.chars().collect();
                    let b: String = chars[..cursor].iter().collect();
                    let a: String = chars[cursor..].iter().collect();
                    (b, a)
                };
                let line = Line::from(vec![
                    Span::raw(symbol),
                    Span::styled(key, key_style),
                    Span::styled(": ", Style::default().fg(Color::DarkGray)),
                    Span::styled(before, Style::default().fg(Color::Yellow)),
                    Span::styled("█", Style::default().fg(Color::White).bg(Color::Yellow)),
                    Span::styled(after, Style::default().fg(Color::Yellow)),
                ]);
                ListItem::new(line).style(Style::default().bg(Color::DarkGray))
            } else {
                // Truncate long values for display
                let display_val = if value.len() > 60 {
                    let truncated: String = value.chars().take(57).collect();
                    format!("{}...", truncated)
                } else {
                    value.clone()
                };
                let val_style = if selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };
                let line = Line::from(vec![
                    Span::raw(symbol),
                    Span::styled(key, key_style),
                    Span::styled(": ", Style::default().fg(Color::DarkGray)),
                    Span::styled(display_val, val_style),
                ]);
                ListItem::new(line).style(if selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                })
            }
        })
        .collect();

    Widget::render(
        List::new(items).block(
            Block::bordered()
                .title(title)
                .border_type(BorderType::Rounded),
        ),
        area,
        buf,
    );
}

fn render_notification(message: &str, area: Rect, buf: &mut Buffer) {
    let w = (message.len() + 4).min(area.width as usize) as u16;
    let h = 3u16;
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h + 2);
    let rect = Rect::new(x, y, w, h);

    for py in rect.y..rect.y + rect.height {
        for px in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((px, py)) {
                cell.set_char(' ');
                cell.set_bg(Color::Black);
            }
        }
    }

    Paragraph::new(message)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Green).bg(Color::Black)),
        )
        .centered()
        .style(Style::default().fg(Color::Green).bg(Color::Black))
        .render(rect, buf);
}

fn split_at_cursor(s: &str, cursor: usize) -> (String, String) {
    let chars: Vec<char> = s.chars().collect();
    let before: String = chars[..cursor.min(chars.len())].iter().collect();
    let after: String = chars[cursor.min(chars.len())..].iter().collect();
    (before, after)
}
