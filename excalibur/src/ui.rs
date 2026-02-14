use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};

use crate::{app::App, view::View};

impl Widget for &App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.current_view {
            View::MainMenu => self.render_main_menu(area, buf),
            View::Module(_) => self.render_module(area, buf),
        }
    }
}

impl App {
    /// Render the main menu
    fn render_main_menu(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Help
            ])
            .split(area);

        // Title
        let title_block = Block::bordered()
            .title("Excalibur CLI (xcl)")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan));
        title_block.render(chunks[0], buf);

        // Module list
        let modules = self.module_manager.list_modules();
        let items: Vec<ListItem> = modules
            .iter()
            .enumerate()
            .map(|(i, module)| {
                let shortcut_text = if let Some(ch) = module.shortcut {
                    format!("[{}] ", ch)
                } else {
                    "    ".to_string()
                };

                let content = vec![Line::from(vec![
                    Span::styled(shortcut_text, Style::default().fg(Color::Yellow)),
                    Span::raw(&module.name),
                ])];

                let style = if i == self.selected_menu_item {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title("Select a module")
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::White));

        list.render(chunks[1], buf);

        // Help text
        let help_text = Paragraph::new(
            "q: Quit | ↑/↓ or j/k: Navigate | Enter: Select | Shortcut key: Direct access",
        )
        .block(Block::bordered().border_type(BorderType::Rounded))
        .fg(Color::DarkGray)
        .centered();
        help_text.render(chunks[2], buf);
    }

    /// Render the active module
    fn render_module(&self, area: Rect, buf: &mut Buffer) {
        // Delegate rendering to the module manager
        self.module_manager.render(area, buf);
    }
}
