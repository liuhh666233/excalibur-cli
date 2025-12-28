use crate::app::App;

pub mod app;
pub mod event;
pub mod modules;
pub mod ui;
pub mod view;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
