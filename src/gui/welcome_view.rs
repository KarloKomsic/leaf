use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;

// First-run screen: pick a directory, hit confirm, and get dropped
// into the library view after scanning.
pub fn show(content: &Box, state: &Rc<RefCell<AppState>>) {
    let title = Label::builder()
        .label("Welcome to Leaf")
        .margin_top(24)
        .margin_bottom(12)
        .build();

    content.append(&title);

    let subtitle = Label::builder()
        .label("Please select your library directory to get started:")
        .margin_bottom(12)
        .build();

    content.append(&subtitle);

    let entry = Entry::builder()
        .placeholder_text("Path to your books folder...")
        .hexpand(true)
        .margin_bottom(12)
        .margin_start(48)
        .margin_end(48)
        .build();

    content.append(&entry);

    let browse_btn = Button::with_label("Browse...");
    let entry_b = entry.clone();

    browse_btn.connect_clicked(move |_| {
        let dialog = FileDialog::new();
        let entry = entry_b.clone();

        dialog.open(
            None::<&Window>,
            None::<&gio::Cancellable>,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let path_str = path.to_string_lossy().to_string();
                        entry.set_text(&path_str);
                    }
                }
            },
        );
    });

    content.append(&browse_btn);

    let confirm_btn = Button::with_label("Set Library Directory");
    confirm_btn.set_margin_top(12);

    confirm_btn.connect_clicked({
        let state = state.clone();
        let entry = entry.clone();
        let content = content.clone();

        move |_| {
            let path = entry.text().to_string();
            if path.is_empty() {
                return;
            }

            if let Err(e) = state.borrow_mut().set_library_directory(path) {
                let dialog = AlertDialog::builder()
                    .message(&format!("Error: {}", e))
                    .build();
                dialog.show(None::<&Window>);
                return;
            }

            state.borrow_mut().load_library();

            while let Some(child) = content.first_child() {
                content.remove(&child);
            }

            crate::gui::library_view::show(&content, &state);
        }
    });

    content.append(&confirm_btn);
}
