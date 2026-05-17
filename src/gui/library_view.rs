use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;
use crate::domain::reading_status::ReadingStatus;

pub fn show(content: &Box, state: &Rc<RefCell<AppState>>) {
    let state = state.borrow();
    let library = match &state.library {
        Some(lib) => lib,
        None => return,
    };

    let header = Label::builder()
        .label(format!("Leaf — {} books", library.document_count()))
        .halign(Align::Start)
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .build();

    content.append(&header);

    let list = ListBox::new();

    for doc in &library.documents {
        let status = state.get_reading_status(&doc.path);

        let status_str = match status {
            ReadingStatus::Unread => "[  ]",
            ReadingStatus::CurrentlyReading => "[>]",
            ReadingStatus::Completed => "[X]",
        };

        let author_str = match &doc.author {
            Some(a) => format!(" — {}", a),
            None => String::new(),
        };

        let row = Label::builder()
            .label(format!(" {} {}{}", status_str, doc.title, author_str))
            .halign(Align::Start)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();

        list.append(&row);
    }

    let scroll = ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();

    content.append(&scroll);
}
