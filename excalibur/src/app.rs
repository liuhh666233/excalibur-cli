use crate::event::{AppEvent, Event, EventHandler};
use crate::modules::{manager::ModuleManager, ModuleAction};
use crate::view::View;
use ratatui::{
    backend::Backend,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    Terminal,
};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Current view
    pub current_view: View,
    /// Module manager
    pub module_manager: ModuleManager,
    /// Selected menu item index (for main menu navigation)
    pub selected_menu_item: usize,
    /// Event handler.
    pub events: EventHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_view: View::MainMenu,
            module_manager: ModuleManager::new(),
            selected_menu_item: 0,
            events: EventHandler::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::EnterModule(module_id) => {
                    self.module_manager.activate(module_id)?;
                    self.current_view = View::Module(module_id);
                }
                AppEvent::ExitModule => {
                    self.module_manager.deactivate()?;
                    self.current_view = View::MainMenu;
                }
                AppEvent::ModuleAction(action) => match action {
                    ModuleAction::Exit => {
                        self.events.send(AppEvent::ExitModule);
                    }
                    ModuleAction::Quit => {
                        self.quit();
                    }
                    ModuleAction::Output(cmd) => {
                        // Output command to stdout for Fish integration
                        // Exit code 0 means insert command into command line
                        // Must restore terminal before exit since exit() bypasses Drop

                        // Open /dev/tty to send cleanup commands
                        if let Ok(mut tty) = std::fs::OpenOptions::new()
                            .write(true)
                            .open("/dev/tty")
                        {
                            // Clear screen and restore terminal
                            let _ = crossterm::execute!(
                                &mut tty,
                                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                                crossterm::cursor::MoveTo(0, 0),
                                crossterm::terminal::LeaveAlternateScreen,
                                crossterm::event::DisableMouseCapture
                            );
                        }
                        let _ = crossterm::terminal::disable_raw_mode();

                        println!("{}", cmd);
                        std::process::exit(0);
                    }
                    ModuleAction::OutputAndExecute(cmd) => {
                        // Output command and signal to execute immediately
                        // Exit code 10 means insert and execute command
                        // Must restore terminal before exit since exit() bypasses Drop

                        // Open /dev/tty to send cleanup commands
                        if let Ok(mut tty) = std::fs::OpenOptions::new()
                            .write(true)
                            .open("/dev/tty")
                        {
                            // Clear screen and restore terminal
                            let _ = crossterm::execute!(
                                &mut tty,
                                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                                crossterm::cursor::MoveTo(0, 0),
                                crossterm::terminal::LeaveAlternateScreen,
                                crossterm::event::DisableMouseCapture
                            );
                        }
                        let _ = crossterm::terminal::disable_raw_mode();

                        println!("{}", cmd);
                        std::process::exit(10);
                    }
                    ModuleAction::None | ModuleAction::Notification(_) => {}
                },
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // Global quit handlers
        if matches!(key_event.code, KeyCode::Char('c' | 'C'))
            && key_event.modifiers == KeyModifiers::CONTROL
        {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }

        match &self.current_view {
            View::MainMenu => self.handle_main_menu_keys(key_event),
            View::Module(_) => {
                // Route to active module
                let action = self.module_manager.handle_key_event(key_event)?;
                self.events.send(AppEvent::ModuleAction(action));
                Ok(())
            }
        }
    }

    /// Handle key events in main menu
    fn handle_main_menu_keys(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        let modules = self.module_manager.list_modules();
        let module_count = modules.len();

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if module_count > 0 {
                    self.selected_menu_item = (self.selected_menu_item + 1) % module_count;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if module_count > 0 {
                    self.selected_menu_item =
                        if self.selected_menu_item == 0 {
                            module_count - 1
                        } else {
                            self.selected_menu_item - 1
                        };
                }
            }
            KeyCode::Enter => {
                if let Some(module) = modules.get(self.selected_menu_item) {
                    self.events.send(AppEvent::EnterModule(module.id));
                }
            }
            KeyCode::Char(c) => {
                // Check for module shortcuts
                for module in &modules {
                    if module.shortcut == Some(c) {
                        self.events.send(AppEvent::EnterModule(module.id));
                        break;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        // Update active module
        if let Err(e) = self.module_manager.update() {
            eprintln!("Error updating module: {}", e);
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
