use crate::domain::library::{LibraryPath, LibraryState};
use crate::domain::library_collection::Library;
use crate::domain::reading_status::ReadingStatus;
use crate::infrastructure::cache::metadata_cache::MetadataCache;
use crate::infrastructure::config::Config;
use crate::infrastructure::reading_status_store::ReadingStatusStore;
use crate::infrastructure::scanner::scan_library;
use std::path::Path;

#[derive(Debug)]
pub struct AppState {
    pub library_state: LibraryState,
    pub library: Option<Library>,
    config: Config,
    pub metadata_cache: MetadataCache,
    pub reading_status: ReadingStatusStore,
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

        let metadata_cache = MetadataCache::load(Path::new("metadata_cache.json"));
        let reading_status = ReadingStatusStore::load(Path::new("reading_status.json"));

        Self {
            library_state,
            library: None,
            config,
            metadata_cache,
            reading_status,
        }
    }

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

    pub fn load_library(&mut self) {
        if let LibraryState::Ready(lib_path) = &self.library_state {
            let docs = scan_library(Path::new(lib_path.as_str()), &mut self.metadata_cache);

            self.library = Some(Library::new(docs));
            self.metadata_cache.save(Path::new("metadata_cache.json"));
            self.reading_status.save(Path::new("reading_status.json"));
        }
    }

    pub fn get_reading_status(&self, path: &Path) -> ReadingStatus {
        self.reading_status.get(path)
    }

    pub fn mark_started(&mut self, path: &Path) {
        self.reading_status.mark_started(path);
        self.reading_status.save(Path::new("reading_status.json"));
    }

    pub fn mark_completed(&mut self, path: &Path) {
        self.reading_status.mark_completed(path);
        self.reading_status.save(Path::new("reading_status.json"));
    }
}
