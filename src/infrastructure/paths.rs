use std::path::PathBuf;

pub fn config_file_path() -> PathBuf {
    let base_config = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME directory not set");
            PathBuf::from(home).join(".config")
        });

    base_config.join("leaf").join("config.toml")
}
