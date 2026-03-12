mod application;
mod domain;
mod infrastructure;

use application::app_state::AppState;
use infrastructure::config::{Config, ConfigError};
use infrastructure::paths::config_file_path;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let config = Config::load().unwrap_or_else(|_| {
        let config = Config::new();
        config.save().expect("Failed to save default config");
        config
    });

    let mut app_state = AppState::new(config);

    if !app_state.is_library_configured() {
        loop {
            println!("Please enter your library directory:");

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let input = input.trim();

            match app_state.set_library_directory(input.to_string()) {
                Ok(_) => {
                    println!("Library directory saved.");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    println!("Please try again.");
                }
            }
        }
    }

    println!("Appstate: {:#?}", app_state);
}
