use super::collector::Supervisor;
use super::state::{InputMode, ProcessTracerState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Row, StatefulWidget, Table, Widget},
};

/// Render the process tracer UI
pub fn render(state: &ProcessTracerState, area: Rect, buf: &mut Buffer) {
    // Split screen into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Search bar
            Constraint::Min(10),    // Process table
            Constraint::Length(7),  // Details panel
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    render_header(state, chunks[0], buf);
    render_search_bar(state, chunks[1], buf);
    render_process_table(state, chunks[2], buf);
    render_details_panel(state, chunks[3], buf);
    render_status_bar(state, chunks[4], buf);

    // Render notification if present
    if let Some((ref msg, _)) = state.notification {
        render_notification(msg, area, buf);
    }
}

/// Render header with title and process count
fn render_header(state: &ProcessTracerState, area: Rect, buf: &mut Buffer) {
    let (filtered_count, total_count) = state.get_counts();
    let count_text = if filtered_count == total_count {
        format!("{} processes", total_count)
    } else {
        format!("{}/{} processes", filtered_count, total_count)
    };

    let title = format!(" Process Tracer │ {} ", count_text);

    let header = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Cyan));

    header.render(area, buf);
}

/// Render search bar
fn render_search_bar(state: &ProcessTracerState, area: Rect, buf: &mut Buffer) {
    let (text, style) = match state.input_mode {
        InputMode::Search => (
            format!(" Search: {}█", state.search_query),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Normal => (
            " Press / to search".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    };

    let search = Paragraph::new(text)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(style),
        )
        .style(style);

    search.render(area, buf);
}

/// Render process table
fn render_process_table(state: &ProcessTracerState, area: Rect, buf: &mut Buffer) {
    if state.filtered_indices.is_empty() {
        // Show "no processes" message
        let msg = Paragraph::new("No processes found")
            .block(
                Block::bordered()
                    .title(" Processes ")
                    .border_type(BorderType::Rounded),
            )
            .centered()
            .style(Style::default().fg(Color::DarkGray));
        msg.render(area, buf);
        return;
    }

    // Create table rows
    let rows: Vec<Row> = state
        .filtered_indices
        .iter()
        .map(|&idx| {
            let proc = &state.processes[idx];

            // Format warnings
            let warning_symbols: Vec<&str> = proc.warnings.iter()
                .map(|w| w.symbol())
                .collect();
            let warnings_str = warning_symbols.join(" ");

            Row::new(vec![
                proc.pid.to_string(),
                proc.name.clone(),
                proc.user.clone(),
                format!("{:.1}%", proc.cpu_percent),
                proc.memory_str(),
                warnings_str,
            ])
        })
        .collect();

    // Create table
    let header = Row::new(vec!["PID", "Name", "User", "CPU%", "Memory", "Warnings"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let sort_indicator = format!(" Processes (Sort: {}) ", state.sort_mode.as_str());

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(15),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .title(sort_indicator)
            .border_type(BorderType::Rounded),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    // Render with state
    let mut table_state = state.table_state.clone();
    StatefulWidget::render(table, area, buf, &mut table_state);
}

/// Render details panel for selected process
fn render_details_panel(state: &ProcessTracerState, area: Rect, buf: &mut Buffer) {
    if let Some(proc) = state.get_selected_process() {
        let cmdline = if proc.cmdline.is_empty() {
            format!("[{}]", proc.name)
        } else {
            proc.cmdline.join(" ")
        };

        // Truncate if too long
        let cmdline_display = if cmdline.len() > 100 {
            format!("{}...", &cmdline[..97])
        } else {
            cmdline
        };

        let supervisor_str = match &proc.supervisor {
            Supervisor::Systemd { unit } => format!("systemd ({})", unit),
            Supervisor::Docker { container_id } => format!("docker ({})", container_id),
            Supervisor::Shell => "shell".to_string(),
            Supervisor::Unknown => "unknown".to_string(),
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Cyan)),
                Span::raw(&cmdline_display),
            ]),
            Line::from(vec![
                Span::styled("Parent:  ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("PID {}", proc.ppid)),
            ]),
            Line::from(vec![
                Span::styled("User:    ", Style::default().fg(Color::Cyan)),
                Span::raw(&proc.user),
            ]),
            Line::from(vec![
                Span::styled("Uptime:  ", Style::default().fg(Color::Cyan)),
                Span::raw(proc.uptime_str()),
            ]),
            Line::from(vec![
                Span::styled("Supervisor: ", Style::default().fg(Color::Cyan)),
                Span::raw(supervisor_str),
            ]),
        ];

        // Add warnings if any
        if !proc.warnings.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Warnings: ", Style::default().fg(Color::Red)),
            ]));

            for warning in &proc.warnings {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        warning.symbol(),
                        Style::default().fg(warning.color()),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        warning.description(),
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            }
        }

        let details = Paragraph::new(lines).block(
            Block::bordered()
                .title(format!(" Details - {} (PID {}) ", proc.name, proc.pid))
                .border_type(BorderType::Rounded),
        );

        details.render(area, buf);
    } else {
        let msg = Paragraph::new("No process selected")
            .block(
                Block::bordered()
                    .title(" Details ")
                    .border_type(BorderType::Rounded),
            )
            .centered()
            .style(Style::default().fg(Color::DarkGray));
        msg.render(area, buf);
    }
}

/// Render status bar with keybindings
fn render_status_bar(_state: &ProcessTracerState, area: Rect, buf: &mut Buffer) {
    let help_text = "[j/k] Navigate  [s] Sort  [r] Refresh  [/] Search  [Esc/q] Exit";

    let status = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        )
        .centered()
        .style(Style::default().fg(Color::Gray));

    status.render(area, buf);
}

/// Render notification popup
fn render_notification(message: &str, area: Rect, buf: &mut Buffer) {
    let notification_width = (message.len() + 4).min(area.width as usize) as u16;
    let notification_height = 3;

    let x = area.width.saturating_sub(notification_width) / 2;
    let y = area.height.saturating_sub(notification_height + 2);

    let notification_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: notification_width,
        height: notification_height,
    };

    // Clear background
    for py in notification_area.y..notification_area.y + notification_area.height {
        for px in notification_area.x..notification_area.x + notification_area.width {
            if let Some(cell) = buf.cell_mut((px, py)) {
                cell.set_char(' ');
                cell.set_bg(Color::Black);
            }
        }
    }

    let notification = Paragraph::new(message)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Green).bg(Color::Black)),
        )
        .centered()
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    notification.render(notification_area, buf);
}
