mod application;
mod domain;
mod gui;
mod infrastructure;

use application::app_state::AppState;
use application::cli_app::CliApp;
use infrastructure::config::Config;

// CLI-first — --gui opens the GTK window, otherwise we stay in the terminal.
// The --gui flag is filtered before passing to GTK so it doesn't choke on it.
fn main() {
    let use_gui = std::env::args().any(|a| a == "--gui");

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
