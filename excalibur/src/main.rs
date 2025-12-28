use crate::app::App;
use std::fs::OpenOptions;

pub mod app;
pub mod event;
pub mod modules;
pub mod ui;
pub mod view;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Open /dev/tty for terminal I/O instead of stdout
    // This allows stdout to be used for command output while TUI uses /dev/tty
    let tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?;

    let backend = ratatui::backend::CrosstermBackend::new(tty);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    // Run app
    let result = App::new().run(&mut terminal);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    result
}
