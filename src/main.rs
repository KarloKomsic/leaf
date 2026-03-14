mod application;
mod domain;
mod infrastructure;

// Application layer import
use application::app_state::AppState;

// Domain layer import
use domain::library_collection::Library;

// Infrastructure layer import
use infrastructure::config::{Config, ConfigError};
use infrastructure::paths::config_file_path;
use infrastructure::scanner::scan_library;

// Imports from std
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn main() {
    let config = Config::load().unwrap_or_else(|_| {
        let config = Config::new();
        config.save().expect("Failed to save default config");
        config
    });

    let mut app_state = AppState::new(config);
    app_state.load_library();

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

    if let Some(library) = &app_state.library {
        println!("Found {} documents:", library.document_count());

        for doc in &library.documents {
            println!(" - {}", doc.title);
        }

        // Query search
        println!("\nEnter a search query:");
        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();
        let query = query.trim(); // Trims the \n sign

        // Results
        let results = library.search(query);
        println!("\nFound {} results:", results.len());
        for (i, doc) in results.iter().enumerate() {
            println!(" {}. {}", i + 1, doc.title);
        }

        // Select result user wants
        println!("\nEnter number to open book (or press Enter to skip):");
        let mut selection = String::new();
        std::io::stdin().read_line(&mut selection).unwrap();
        let selection = selection.trim();

        // Parse input
        if let Ok(index) = selection.parse::<usize>() {
            if index > 0 && index <= results.len() {
                let doc = results[index - 1];
                println!("Opening: {}", doc.title);

                // Open the file
                Command::new("xdg-open")
                    .arg(&doc.path)
                    .spawn()
                    .expect("Failed to open file");
            }
        } else {
            println!("Invalid selection.");
        }
    }
}
