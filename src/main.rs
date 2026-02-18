mod application;
mod domain;
mod infrastructure;

use application::app_state::AppState;
use infrastructure::config::Config;

fn main() {
    let config = Config::new();

    let app_state = AppState::new(config);

    println!("AppState: {:#?}", app_state);
}
