mod application;
mod domain;
mod infrastructure;

use application::app_state::AppState;
use application::cli_app::CliApp;
use infrastructure::config::Config;

fn main() {
    let config = Config::load().unwrap_or_else(|_| {
        let config = Config::new();
        config.save().expect("Failed to save default config");
        config
    });

    let mut state = AppState::new(config);
    state.load_library();

    let mut app = CliApp::new(state);
    app.run();
}
