use super::state::{HistoryState, InputMode};
use chrono::{DateTime, Utc};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Row, Table, Widget},
};

/// Main render function for history module
pub fn render(state: &HistoryState, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Search bar
            Constraint::Min(10),   // Table
            Constraint::Length(6), // Details panel
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    render_header(state, chunks[0], buf);
    render_search_bar(state, chunks[1], buf);
    render_table(state, chunks[2], buf);
    render_details(state, chunks[3], buf);
    render_status_bar(state, chunks[4], buf);

    // Render notification if present
    if let Some((ref msg, _)) = state.notification {
        render_notification(msg, area, buf);
    }
}

/// Render the header
fn render_header(state: &HistoryState, area: Rect, buf: &mut Buffer) {
    let title = format!(
        " Command History │ Sort: {} │ Commands: {}/{} ",
        state.sort_mode.display(),
        state.filtered_count(),
        state.total_count()
    );

    let header = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Cyan));

    header.render(area, buf);
}

/// Render the search bar
fn render_search_bar(state: &HistoryState, area: Rect, buf: &mut Buffer) {
    let (text, style) = match state.input_mode {
        InputMode::Search => (
            format!(" Search: {}█", state.search_query),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Normal if !state.search_query.is_empty() => (
            format!(" Filter: {} ", state.search_query),
            Style::default().fg(Color::Green),
        ),
        InputMode::Normal => (
            " Press / to search ".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    };

    let search = Paragraph::new(text)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .style(style);

    search.render(area, buf);
}

/// Render the command table
fn render_table(state: &HistoryState, area: Rect, buf: &mut Buffer) {
    if state.filtered_indices.is_empty() {
        let empty = Paragraph::new("No commands found")
            .block(
                Block::bordered()
                    .title(" Commands ")
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        empty.render(area, buf);
        return;
    }

    // 虚拟滚动：只渲染可见行
    let table_height = area.height.saturating_sub(4) as usize; // 减去边框和表头
    let visible_rows = table_height.min(30); // 最多渲染 30 行

    // 计算可见范围
    let selected = state.selected_index;
    let total = state.filtered_indices.len();

    // 计算滚动窗口
    let start = if selected < visible_rows / 2 {
        0
    } else if selected + visible_rows / 2 >= total {
        total.saturating_sub(visible_rows)
    } else {
        selected.saturating_sub(visible_rows / 2)
    };
    let end = (start + visible_rows).min(total);

    // 只为可见行创建 Row
    let rows: Vec<Row> = state
        .filtered_indices
        .iter()
        .skip(start)
        .take(end - start)
        .map(|&idx| {
            let cmd = &state.commands[idx];

            // 使用字节长度预检查（更快）
            let cmd_display = if cmd.cmd.len() > 180 {
                // 约 60 个中文字符
                // 只在需要时才调用昂贵的 chars 操作
                let char_count = cmd.cmd.chars().count();
                if char_count > 60 {
                    let truncated: String = cmd.cmd.chars().take(57).collect();
                    format!("{}...", truncated)
                } else {
                    cmd.cmd.clone()
                }
            } else {
                cmd.cmd.clone()
            };

            let path_count = if cmd.paths.is_empty() {
                "-".to_string()
            } else {
                cmd.paths.len().to_string()
            };

            Row::new(vec![
                format!(" {} ", cmd.count),
                format!(" {} ", cmd_display),
                format!(" {} ", path_count),
                format!(" {} ", cmd.format_timestamp()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Min(40),
        Constraint::Length(8),
        Constraint::Length(15),
    ];

    let header = Row::new(vec![" Count ", " Command ", " Paths ", " Last Used "])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    // 调整 table_state 的选中索引为相对位置
    let mut adjusted_state = state.table_state.clone();
    if let Some(selected) = adjusted_state.selected() {
        if selected >= start {
            adjusted_state.select(Some(selected - start));
        }
    }

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(format!(" Commands ({}-{}/{}) ", start + 1, end, total))
                .border_type(BorderType::Rounded),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    ratatui::widgets::StatefulWidget::render(table, area, buf, &mut adjusted_state);
}

/// Render the details panel
fn render_details(state: &HistoryState, area: Rect, buf: &mut Buffer) {
    let content = if let Some(cmd) = state.get_selected_command() {
        let stats = state.stats.get(&cmd.cmd);

        let first_used = stats
            .map(|s| format_timestamp_full(s.first_used))
            .unwrap_or_else(|| "unknown".to_string());

        let last_used = format_timestamp_full(cmd.timestamp);

        vec![
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Cyan)),
                Span::raw(&cmd.cmd),
            ]),
            Line::from(vec![
                Span::styled("First used: ", Style::default().fg(Color::Cyan)),
                Span::raw(first_used),
            ]),
            Line::from(vec![
                Span::styled("Last used: ", Style::default().fg(Color::Cyan)),
                Span::raw(last_used),
            ]),
            Line::from(vec![
                Span::styled("Total uses: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{} times", cmd.count)),
            ]),
        ]
    } else {
        vec![Line::from("No command selected")]
    };

    let details = Paragraph::new(content)
        .block(
            Block::bordered()
                .title(" Command Details ")
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::White));

    details.render(area, buf);
}

/// Render the status bar
fn render_status_bar(state: &HistoryState, area: Rect, buf: &mut Buffer) {
    let help_text = match state.input_mode {
        InputMode::Search => "Esc: Exit search │ Enter: Apply filter │ Type to search",
        InputMode::Normal => {
            "Enter: Select │ Ctrl+O: Execute │ Esc/q: Exit │ /: Search │ s: Sort │ ↑↓/jk: Navigate"
        }
    };

    let status = Paragraph::new(help_text)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::DarkGray))
        .centered();

    status.render(area, buf);
}

/// Render notification popup
fn render_notification(message: &str, area: Rect, buf: &mut Buffer) {
    let notification_width = (message.len() + 4).min(area.width as usize - 2) as u16;
    let notification_height = 3;

    let x = (area.width.saturating_sub(notification_width)) / 2;
    let y = area.height.saturating_sub(notification_height + 2);

    let notification_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: notification_width,
        height: notification_height,
    };

    let notification = Paragraph::new(message)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Green).bg(Color::Black)),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .centered();

    notification.render(notification_area, buf);
}

/// Format timestamp as full date/time string
fn format_timestamp_full(timestamp: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        let utc: DateTime<Utc> = dt.into();
        utc.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "unknown".to_string()
    }
}
