// Leaf is a book tracker that can run either as a terminal app or a
// graphical window. By default you get the terminal, but passing
// --gui opens the GTK interface instead.

mod application;
mod domain;
mod gui;
mod infrastructure;

use application::app_state::AppState;
use application::cli_app::CliApp;
use infrastructure::config::Config;

fn main() {
    let use_gui = std::env::args().any(|a| a == "--gui");

    // Load saved config, or create a fresh one if this is the first run
    let config = Config::load().unwrap_or_else(|_| {
        let config = Config::new();
        config.save().expect("Failed to save default config");
        config
    });

    if use_gui {
        let state = AppState::new(config);
        gui::run(state);
    } else {
        let mut state = AppState::new(config);
        state.load_library();
        let mut app = CliApp::new(state);
        app.run();
    }
}
