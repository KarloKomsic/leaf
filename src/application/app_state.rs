use crate::domain::library::{LibraryPath, LibraryState};
use crate::infrastructure::config::Config;

#[derive(Debug)]
pub struct AppState {
    pub library_state: LibraryState,
    config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let library_state = match &config.library_directory {
            Some(path) => {
                // For now, we assume every path is valid
                // Validation comes later
                let lib_path = LibraryPath::new(path.clone());
                LibraryState::Ready(lib_path)
            }
            None => LibraryState::NotConfigured,
        };

        Self {
            library_state,
            config,
        }
    }
}
