use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::ApplicationExtManual;
use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;

pub fn run(state: AppState) {
    let app = Application::builder()
        .application_id("com.leaf.app")
        .build();

    let state = Rc::new(RefCell::new(state));

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Leaf")
            .default_width(960)
            .default_height(720)
            .build();

        if state.borrow().is_library_configured() {
            state.borrow_mut().load_library();
        }

        let content = Box::new(Orientation::Vertical, 0);
        window.set_child(Some(&content));

        if state.borrow().is_library_configured() {
            super::library_view::show(&content, &state);
        } else {
            super::welcome_view::show(&content, &state);
        }

        window.present();
    });

    let args: Vec<String> = std::env::args().filter(|a| a != "--gui").collect();
    app.run_with_args(&args);
}
