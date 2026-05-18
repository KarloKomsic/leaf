use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::ApplicationExtManual;
use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;
use crate::infrastructure::cover_cache;

pub fn run(state: AppState) {
    let app = Application::builder()
        .application_id("com.leaf.app")
        .build();

    let state = Rc::new(RefCell::new(state));

    app.connect_activate(move |app| {
        super::styles::load();

        if state.borrow().is_library_configured() {
            state.borrow_mut().load_library();
            generate_covers(&state);
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Leaf")
            .default_width(960)
            .default_height(720)
            .build();

        let header = HeaderBar::builder().show_title_buttons(true).build();
        window.set_titlebar(Some(&header));

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

// Every book hopefully needs a cover
// This makes sure the covers appear in the GUI
fn generate_covers(state: &Rc<RefCell<AppState>>) {
    let library = {
        let s = state.borrow();
        s.library.as_ref().map(|l| l.documents.clone())
    };

    if let Some(docs) = library {
        eprintln!("Generating covers for {} books...", docs.len());
        for doc in &docs {
            let result = cover_cache::cached_cover_path(&doc.path);
            if result.is_none() {
                eprintln!(
                    "  Cover failed: {}",
                    doc.path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
        eprintln!("Cover generation complete.");
    }
}
