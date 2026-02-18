#[derive(Debug)]
pub struct Config {
    pub config_version: String,
    pub library_directory: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            config_version: "0.1.0".to_string(),
            library_directory: None,
        }
    }
}
