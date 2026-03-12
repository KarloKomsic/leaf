use crate::domain::library::{LibraryPath, LibraryState};
use crate::infrastructure::config::Config;
use std::path::Path;

#[derive(Debug)]
pub struct AppState {
    pub library_state: LibraryState,
    config: Config,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let library_state = match &config.library_directory {
            Some(path) => {
                let path_ref = Path::new(path);

                if path_ref.exists() && path_ref.is_dir() {
                    let library_path = LibraryPath::new(path.clone());
                    LibraryState::Ready(library_path)
                } else {
                    LibraryState::Invalid
                }
            }
            None => LibraryState::NotConfigured,
        };

        Self {
            library_state,
            config,
        }
    }

    // This lets the library ask if library needs to be configured/set
    pub fn is_library_configured(&self) -> bool {
        !matches!(self.library_state, LibraryState::NotConfigured)
    }

    pub fn set_library_directory(&mut self, path: String) -> Result<(), String> {
        use std::path::Path;

        let path_ref = Path::new(&path);

        if path_ref.exists() && path_ref.is_dir() {
            let lib_path = LibraryPath::new(path.clone());

            self.library_state = LibraryState::Ready(lib_path);
            self.config.library_directory = Some(path);

            self.config
                .save()
                .map_err(|e| format!("Failed to save config: {}", e))?;

            Ok(())
        } else {
            Err("Directory does not exist or is not a valid directory".to_string())
        }
    }
}
