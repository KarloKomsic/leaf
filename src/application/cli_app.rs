use crate::application::app_state::AppState;

use crate::domain::document::Document;
use crate::domain::library_collection::Library;

use std::io;
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
                match &doc.author {
                    Some(author) => println!(" - {} — {}", doc.title, author),
                    None => println!("- {}", doc.title),
                }
            }
        }
    }

    fn search_loop(&self) {
        if let Some(library) = &self.state.library {
            loop {
                let query = self.read_query();

                if query.is_empty() {
                    println!("Please enter a search query.");
                    continue;
                }

                if self.handle_command(&query, library) {
                    break;
                }

                if query.len() >= 2 && query.len() <= 3 {
                    let suggestions = library.suggest(&query);

                    if !suggestions.is_empty() {
                        println!("\nSuggestions:");
                        for (i, doc) in suggestions.iter().enumerate() {
                            println!(" {}. {}", i + 1, doc.title);
                        }
                        continue;
                    }
                }

                self.handle_search(&query, library);
            }
        }
    }

    fn read_query(&self) -> String {
        println!(
            "\nEnter a book name or author name (or 'random' for random book, or 'exit' for quitting the program):"
        );

        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();

        query.trim().to_string()
    }

    fn handle_command(&self, query: &str, library: &Library) -> bool {
        match query {
            "exit" | "quit" | "q" => {
                println!("Goodbye!");
                return true;
            }

            "random" => {
                if let Some(doc) = library.random() {
                    println!("Opening random book: {}", doc.title);
                    open_file(doc);
                } else {
                    println!("Library is empty.");
                }
            }

            _ => return false,
        }

        false
    }

    fn handle_search(&self, query: &str, library: &Library) {
        let results = library.search(query);

        if results.is_empty() {
            println!("No documents found.");
            return;
        }

        println!("\nFound {} results:", results.len());

        for (i, doc) in results.iter().enumerate() {
            println!(" {}. {}", i + 1, doc.title);
        }

        self.handle_selection(results);
    }

    fn handle_selection(&self, results: Vec<&Document>) {
        println!("\nEnter number to open book (or press Enter to skip):");

        let mut selection = String::new();
        io::stdin().read_line(&mut selection).unwrap();
        let selection = selection.trim();

        if selection.is_empty() {
            return;
        }

        match selection.parse::<usize>() {
            Ok(index) if index > 0 && index <= results.len() => {
                let doc = results[index - 1];
                println!("Opening: {}", doc.title);
                open_file(doc);
            }

            _ => println!("Invalid selection"),
        }
    }
}
