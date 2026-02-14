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
    render_preview(state, main_chunks[1], buf);

    // Action bar
    let (text, style) = match state.input_mode {
        InputMode::ConfirmSwap => (
            " Switch to this profile? [Enter] Switch  [b] Backup & Switch  [Esc] Cancel"
                .to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::BackupRename => (
            format!(" Backup current as: settings_{}.json█", state.rename_input),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::SelectProfile => (String::new(), Style::default().fg(Color::DarkGray)),
    };
    Paragraph::new(text)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .style(style)
        .render(chunks[2], buf);

    // Status bar
    let help = match state.input_mode {
        InputMode::SelectProfile => "[Enter] Switch  [j/k] Navigate  [Esc] Exit",
        InputMode::ConfirmSwap => "[Enter] Switch (delete old)  [b] Backup first  [Esc] Cancel",
        InputMode::BackupRename => "[Enter] Confirm  [Esc] Back",
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
