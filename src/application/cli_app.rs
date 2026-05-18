use crate::application::app_state::AppState;

use crate::domain::document::Document;
use crate::domain::reading_status::ReadingStatus;

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

enum CmdResult {
    Quit,
    Handled,
    NotFound,
}

// Show a simple text badge — [  ] unread, [>] reading, [X] done
fn status_symbol(status: &ReadingStatus) -> &'static str {
    match status {
        ReadingStatus::Unread => "[  ]",
        ReadingStatus::CurrentlyReading => "[>]",
        ReadingStatus::Completed => "[X]",
    }
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
        let library = match &self.state.library {
            Some(lib) => lib,
            None => return,
        };

        println!("Found {} documents:", library.document_count());

        let mut unread = 0u32;
        let mut reading = 0u32;
        let mut completed = 0u32;

        for doc in &library.documents {
            let status = self.state.get_reading_status(&doc.path);
            let sym = status_symbol(&status);

            match status {
                ReadingStatus::CurrentlyReading => reading += 1,
                ReadingStatus::Completed => completed += 1,
                ReadingStatus::Unread => unread += 1,
            }

            match &doc.author {
                Some(author) => println!(" {} {} — {}", sym, doc.title, author),
                None => println!(" {} {}", sym, doc.title),
            }
        }

        println!(
            "\nStats: {} unread, {} currently reading, {} completed",
            unread, reading, completed
        );
    }

    fn search_loop(&mut self) {
        loop {
            let query = self.read_query();

            if query.is_empty() {
                println!("Please enter a search query.");
                continue;
            }

            match self.handle_command(&query) {
                CmdResult::Quit => break,
                CmdResult::Handled => continue,
                CmdResult::NotFound => {}
            }

            // Short queries (2-3 chars) trigger auto-suggest
            if query.len() >= 2 && query.len() <= 3 {
                let should_suggest = self.state.library.as_ref().map(|lib| {
                    let suggestions = lib.suggest(&query);
                    if !suggestions.is_empty() {
                        println!("\nSuggestions:");
                        for (i, doc) in suggestions.iter().enumerate() {
                            println!(" {}. {}", i + 1, doc.title);
                        }
                        true
                    } else {
                        false
                    }
                }).unwrap_or(false);

                if should_suggest {
                    continue;
                }
            }

            self.handle_search(&query);
        }
    }

    fn read_query(&self) -> String {
        println!(
            "\nEnter a book name or author name (or 'random' for random book, 'list' to list all books, 'reading' for currently reading, 'completed' for completed, or 'exit' to quit):"
        );

        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();

        query.trim().to_string()
    }

    fn handle_command(&mut self, query: &str) -> CmdResult {
        match query {
            "exit" | "quit" | "q" => {
                println!("Goodbye!");
                CmdResult::Quit
            }

            "random" => {
                let doc = match &self.state.library {
                    Some(lib) => lib.random(),
                    None => None,
                };

                let (path, title) = match doc {
                    Some(d) => (d.path.clone(), d.title.clone()),
                    None => {
                        println!("Library is empty.");
                        return CmdResult::Handled;
                    }
                };

                println!("Opening random book: {}", title);
                self.state.mark_started(&path);
                let doc = Document {
                    title,
                    author: None,
                    path,
                };
                open_file(&doc);
                CmdResult::Handled
            }

            "list" => {
                self.show_library();
                CmdResult::Handled
            }

            "reading" => {
                self.list_by_status(ReadingStatus::CurrentlyReading);
                CmdResult::Handled
            }

            "completed" => {
                self.list_by_status(ReadingStatus::Completed);
                CmdResult::Handled
            }

            _ => CmdResult::NotFound,
        }
    }

    fn list_by_status(&self, status: ReadingStatus) {
        let library = match &self.state.library {
            Some(lib) => lib,
            None => return,
        };

        let label = match status {
            ReadingStatus::CurrentlyReading => "Currently Reading",
            ReadingStatus::Completed => "Completed",
            ReadingStatus::Unread => "Unread",
        };

        let docs: Vec<&Document> = library
            .documents
            .iter()
            .filter(|doc| self.state.get_reading_status(&doc.path) == status)
            .collect();

        if docs.is_empty() {
            println!("No books in '{}' category.", label);
            return;
        }

        println!("\n--- {} ({}) ---", label, docs.len());
        for doc in &docs {
            match &doc.author {
                Some(author) => println!(" {} — {}", doc.title, author),
                None => println!(" {}", doc.title),
            }
        }
    }

    fn handle_search(&mut self, query: &str) {
        let titles: Vec<_> = {
            let library = match &self.state.library {
                Some(lib) => lib,
                None => return,
            };

            let results = library.search(query);

            if results.is_empty() {
                println!("No documents found.");
                return;
            }

            println!("\nFound {} results:", results.len());

            for (i, result) in results.iter().enumerate() {
                println!(
                    " {}. {} [{:?}]",
                    i + 1,
                    result.document.title,
                    result.match_type
                );
            }

            results
                .into_iter()
                .map(|r| (r.document.path.clone(), r.document.title.clone()))
                .collect::<Vec<_>>()
        };

        self.handle_selection(titles);
    }

    fn handle_selection(&mut self, results: Vec<(std::path::PathBuf, String)>) {
        println!("\nEnter number to open book, or 'c <n>' to mark as completed (or press Enter to skip):");

        let mut selection = String::new();
        io::stdin().read_line(&mut selection).unwrap();
        let selection = selection.trim();

        if selection.is_empty() {
            return;
        }

        // "c 3" marks the 3rd result as completed without opening
        if let Some(rest) = selection.strip_prefix("c ") {
            if let Ok(index) = rest.parse::<usize>() {
                if index > 0 && index <= results.len() {
                    let (path, title) = &results[index - 1];

                    self.state.mark_completed(path);
                    println!("Marked as completed: {}", title);
                    return;
                }
            }
            println!("Invalid selection");
            return;
        }

        match selection.parse::<usize>() {
            Ok(index) if index > 0 && index <= results.len() => {
                let (path, title) = &results[index - 1];

                println!("Opening: {}", title);
                self.state.mark_started(path);
                let doc = Document {
                    title: title.clone(),
                    author: None,
                    path: path.clone(),
                };
                open_file(&doc);
            }

            _ => println!("Invalid selection"),
        }
    }

}
