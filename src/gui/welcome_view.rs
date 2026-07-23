use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;

pub fn show(
    window: &ApplicationWindow,
    content: &Box,
    state: &Rc<RefCell<AppState>>,
    header: &HeaderBar,
    hamburger: &MenuButton,
) {
    let wrapper = Box::new(Orientation::Vertical, 0);
    wrapper.set_halign(Align::Center);
    wrapper.set_valign(Align::Start);
    wrapper.add_css_class("welcome-wrapper");
    wrapper.set_spacing(0);

    let logo_bytes = include_bytes!("../../data/icons/hicolor/1024x1024/apps/com.leaf.app.png");
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(logo_bytes).expect("embedded icon bytes are valid");
    loader.close().expect("PixbufLoader close should succeed");
    let pixbuf = loader.pixbuf().expect("embedded icon should produce a pixbuf");
    let scaled = pixbuf.scale_simple(128, 128, gdk_pixbuf::InterpType::Bilinear)
        .expect("embedded icon should scale successfully");
    let pixel_bytes = scaled.read_pixel_bytes();
    let format = if scaled.has_alpha() {
        gtk4::gdk::MemoryFormat::R8g8b8a8
    } else {
        gtk4::gdk::MemoryFormat::R8g8b8
    };
    let texture = gtk4::gdk::MemoryTexture::new(
        scaled.width(),
        scaled.height(),
        format,
        &pixel_bytes,
        scaled.rowstride() as usize,
    );
    let logo = Picture::for_paintable(&texture);
    logo.set_content_fit(ContentFit::Contain);
    logo.set_margin_bottom(12);
    wrapper.append(&logo);

    let title = Label::builder()
        .label("Leaf")
        .css_classes(["welcome-title"])
        .build();
    wrapper.append(&title);

    let subtitle = Label::builder()
        .label("Your personal library manager")
        .css_classes(["welcome-subtitle"])
        .build();
    wrapper.append(&subtitle);

    let button = Button::builder()
        .label("Set Library Directory")
        .css_classes(["welcome-button"])
        .halign(Align::Center)
        .build();
    let state_clone = state.clone();
    let content_clone = content.clone();
    let header_clone = header.clone();
    let hamburger_clone = hamburger.clone();
    let win = window.clone();
    button.connect_clicked(move |_| {
        let dialog = FileDialog::new();
        let state = state_clone.clone();
        let content = content_clone.clone();
        let header = header_clone.clone();
        let hamburger = hamburger_clone.clone();
        let win_dialog = win.clone();
        let win_callback = win.clone();
        dialog.select_folder(
            Some(&win_dialog),
            None::<&gio::Cancellable>,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let path_str = path.to_string_lossy().to_string();

                        if let Err(e) = state.borrow_mut().set_library_directory(path_str) {
                            let alert = AlertDialog::builder()
                                .message(&format!("Error: {}", e))
                                .build();
                            alert.show(Some(&win_callback));
                            return;
                        }

                        state.borrow_mut().load_library();

                        win_callback.set_resizable(true);
                        win_callback.set_default_size(960, 720);
                        win_callback.set_title(Some(&format!(
                            "Leaf v{}",
                            env!("CARGO_PKG_VERSION")
                        )));
                        header.pack_start(&hamburger);

                        while let Some(child) = content.first_child() {
                            content.remove(&child);
                        }

                        crate::gui::library_view::show(&content, &state);
                    }
                }
            },
        );
    });
    wrapper.append(&button);

    content.append(&wrapper);
}
