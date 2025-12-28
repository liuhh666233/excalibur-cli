use crate::modules::ModuleId;

/// Represents the current view of the application
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    /// Main menu showing available modules
    MainMenu,
    /// Active module view
    Module(ModuleId),
}

impl Default for View {
    fn default() -> Self {
        Self::MainMenu
    }
}
