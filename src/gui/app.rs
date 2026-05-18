use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::ApplicationExtManual;
use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;
use crate::domain::library::LibraryState;
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

        let configured = state.borrow().is_library_configured();
        let (default_width, default_height) = if configured {
            (960, 720)
        } else {
            (380, 380)
        };

        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(default_width)
            .default_height(default_height)
            .build();

        window.set_icon_name(Some("com.leaf.app"));

        let header = HeaderBar::builder().show_title_buttons(true).build();

        let content = Box::new(Orientation::Vertical, 0);
        window.set_child(Some(&content));

        let hamburger = MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .build();

        let menu_popover = Popover::new();
        let menu_box = Box::new(Orientation::Vertical, 0);

        let settings_item = Button::with_label("Settings");
        settings_item.add_css_class("menu-item");
        let window_settings = window.clone();
        let state_settings = state.clone();
        let content_settings = content.clone();
        settings_item.connect_clicked(move |_| {
            show_settings_dialog(&window_settings, &state_settings, &content_settings);
        });
        menu_box.append(&settings_item);

        menu_popover.set_child(Some(&menu_box));
        hamburger.set_popover(Some(&menu_popover));

        if configured {
            window.set_resizable(true);
            window.set_title(Some(&format!("Leaf v{}", env!("CARGO_PKG_VERSION"))));
            window.set_titlebar(Some(&header));
            header.pack_start(&hamburger);
            super::library_view::show(&content, &state);
        } else {
            window.set_resizable(false);
            window.set_title(Some(""));
            window.set_titlebar(Some(&header));
            super::welcome_view::show(&window, &content, &state, &header, &hamburger);
        }

        window.present();
    });

    let args: Vec<String> = std::env::args().filter(|a| a != "--gui").collect();
    app.run_with_args(&args);
}

fn show_settings_dialog(parent: &ApplicationWindow, state: &Rc<RefCell<AppState>>, content: &Box) {
    let dialog = Window::builder()
        .title("Settings")
        .modal(true)
        .transient_for(parent)
        .default_width(500)
        .default_height(200)
        .build();

    let vbox = Box::new(Orientation::Vertical, 12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);

    let section = Box::new(Orientation::Vertical, 4);
    section.add_css_class("settings-section");

    let dir_label = Label::builder()
        .label("Library Directory")
        .css_classes(["settings-label"])
        .halign(Align::Start)
        .build();
    section.append(&dir_label);

    let current_dir = match &state.borrow().library_state {
        LibraryState::Ready(p) => p.as_str().to_string(),
        _ => String::new(),
    };
    let entry = Entry::builder()
        .text(&current_dir)
        .hexpand(true)
        .build();
    section.append(&entry);

    let browse_btn = Button::with_label("Browse...");
    let entry_browse = entry.clone();
    let parent_clone = parent.clone();
    browse_btn.connect_clicked(move |_| {
        let browse_dialog = FileDialog::new();
        let entry = entry_browse.clone();
        browse_dialog.select_folder(
            Some(&parent_clone),
            None::<&gio::Cancellable>,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        entry.set_text(&path.to_string_lossy());
                    }
                }
            },
        );
    });
    section.append(&browse_btn);

    vbox.append(&section);

    let btn_box = Box::new(Orientation::Horizontal, 8);
    btn_box.set_halign(Align::End);

    let cancel_btn = Button::with_label("Cancel");
    let dialog_cancel = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_cancel.close();
    });
    btn_box.append(&cancel_btn);

    let save_btn = Button::with_label("Save");
    let dialog_save = dialog.clone();
    let state_save = state.clone();
    let content_save = content.clone();
    let parent_save = parent.clone();
    save_btn.connect_clicked(move |_| {
        let new_path = entry.text().to_string();
        if new_path.is_empty() {
            return;
        }

        if let Err(e) = state_save.borrow_mut().set_library_directory(new_path) {
            let alert = AlertDialog::builder()
                .message(&format!("Error: {}", e))
                .build();
            alert.show(Some(&parent_save));
            return;
        }

        state_save.borrow_mut().load_library();

        while let Some(child) = content_save.first_child() {
            content_save.remove(&child);
        }

        crate::gui::library_view::show(&content_save, &state_save);

        dialog_save.close();
    });
    btn_box.append(&save_btn);

    vbox.append(&btn_box);

    dialog.set_child(Some(&vbox));
    dialog.present();
}

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
