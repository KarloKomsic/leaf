use crate::application::app_state::AppState;
use crate::domain::document::Document;
use std::io::{self, Write};
use std::process::Command;

pub struct CliApp {
    pub state: AppState,
}

fn open_file(doc: &Document) {
    Command::new("xdg-open")
        .arg(&doc.path)
        .spawn()
        .expect("Failed to open file");
}

impl CliApp {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn run(&mut self) {
        self.ensure_library_configured();
        self.show_library();
        self.search_loop();
    }

    fn ensure_library_configured(&mut self) {
        if !self.state.is_library_configured() {
            loop {
                println!("Please enter your library directory:");

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                let input = input.trim();

                match self.state.set_library_directory(input.to_string()) {
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
    }

    fn show_library(&self) {
        if let Some(library) = &self.state.library {
            println!("Found {} documents:", library.document_count());

            for doc in &library.documents {
                println!(" - {}", doc.title);
            }
        }
    }

    fn search_loop(&self) {
        if let Some(library) = &self.state.library {
            loop {
                println!(
                    "\nEnter a search query (alternatively, 'random' for random book query and 'exit' to exit the program):"
                );

                // Get query
                let mut query = String::new();
                std::io::stdin().read_line(&mut query).unwrap();
                let query = query.trim();

                // Empty query handling
                if query.is_empty() {
                    println!("Please enter a search query.");
                    continue;
                }

                // Exit command
                if query == "exit" || query == "quit" || query == "q" {
                    println!("Goodbye!");
                    break;
                }

                // Random book command
                if query == "random" {
                    if let Some(doc) = library.random() {
                        println!("Opening random book: {}", doc.title);

                        open_file(doc);
                    } else {
                        println!("Library is empty.");
                    }

                    continue;
                }

                // Normal search
                let results = library.search(query);

                if results.is_empty() {
                    println!("No documents found.");
                    continue;
                }

                println!("\nFound {} results:", results.len());

                for (i, doc) in results.iter().enumerate() {
                    println!(" {}. {}", i + 1, doc.title);
                }

                println!("\nEnter number to open book (or press Enter to skip):");

                let mut selection = String::new();
                std::io::stdin().read_line(&mut selection).unwrap();
                let selection = selection.trim();

                if selection.is_empty() {
                    continue;
                }

                match selection.parse::<usize>() {
                    Ok(index) if index > 0 && index <= results.len() => {
                        let doc = results[index - 1];

                        println!("Opening: {}", doc.title);

                        open_file(doc);
                    }

                    _ => {
                        println!("Invalid selection.");
                    }
                }
            }
        }
    }
}
