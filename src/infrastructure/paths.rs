// Determines where the config file lives on disk. Follows the XDG
// spec so it works on any Linux desktop without hardcoding paths.

use std::path::PathBuf;

fn base_config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME directory not set");
            PathBuf::from(home).join(".config")
        })
        .join("leaf")
}

pub fn config_file_path() -> PathBuf {
    base_config_dir().join("config.toml")
}

pub fn data_file_path(name: &str) -> PathBuf {
    base_config_dir().join(name)
}
