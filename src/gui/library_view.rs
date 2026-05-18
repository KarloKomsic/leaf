use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;
use crate::domain::reading_status::ReadingStatus;

use super::card;
use super::sidebar::{Category, Sidebar};

fn mark_completed(
    state: &Rc<RefCell<AppState>>,
    cards: &Rc<RefCell<Vec<(ListBoxRow, Box, ReadingStatus)>>>,
    doc_path: &std::path::Path,
    idx: usize,
) {
    state.borrow_mut().mark_completed(doc_path);
    if let Some(entry) = cards.borrow_mut().get_mut(idx) {
        entry.2 = ReadingStatus::Completed;
        entry.0.remove_css_class("reading");
        entry.0.remove_css_class("unread");
        entry.0.add_css_class("completed");
        if let Some(child) = entry.1.last_child() {
            entry.1.remove(&child);
        }
    }
}

pub fn show(content: &Box, state: &Rc<RefCell<AppState>>) {
    let header = {
        let s = state.borrow();
        let count = s.library.as_ref().map(|l| l.document_count()).unwrap_or(0);
        Label::builder()
            .label(format!("{} books", count))
            .halign(Align::Start)
            .margin_top(12)
            .margin_bottom(6)
            .margin_start(12)
            .build()
    };
    content.append(&header);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_wide_handle(true);

    // Sidebar
    let sidebar = Sidebar::new();
    let scroll_sidebar = ScrolledWindow::builder()
        .child(&sidebar.container)
        .min_content_width(200)
        .build();
    paned.set_start_child(Some(&scroll_sidebar));
    paned.set_resize_start_child(false);
    paned.set_shrink_start_child(false);

    // Book list with cards
    let book_list = ListBox::new();
    book_list.set_selection_mode(SelectionMode::Single);
    book_list.set_css_classes(&["card-box"]);
    // Align top so cards keep natural height in every category view
    book_list.set_valign(Align::Start);

    let scroll_books = ScrolledWindow::builder()
        .child(&book_list)
        .vexpand(true)
        .hexpand(true)
        .build();
    paned.set_end_child(Some(&scroll_books));

    let cards: Rc<RefCell<Vec<(ListBoxRow, Box, ReadingStatus)>>> = Rc::new(RefCell::new(Vec::new()));

    let library_docs: Vec<(crate::domain::reading_status::ReadingStatus, std::path::PathBuf)>;

    // Phase 1: build all cards
    {
        let s = state.borrow();
        let library = match &s.library {
            Some(lib) => lib,
            None => return,
        };

        library_docs = library
            .documents
            .iter()
            .map(|doc| (s.get_reading_status(&doc.path), doc.path.clone()))
            .collect();

        let mut entries = Vec::with_capacity(library.documents.len());

        for (idx, doc) in library.documents.iter().enumerate() {
            let status = library_docs[idx].0.clone();
            let (row, hbox) = card::create(doc, &status);
            book_list.append(&row);
            entries.push((row, hbox, status));
        }

        *cards.borrow_mut() = entries;
    }

    // Phase 2: add mark-complete button to non-completed cards
    {
        let cards_ref = cards.borrow();
        for (idx, (status, _)) in library_docs.iter().enumerate() {
            if *status == ReadingStatus::Completed {
                continue;
            }

            let (_, hbox, _) = &cards_ref[idx];

            let state_mc = state.clone();
            let cards_mc = cards.clone();
            let doc_path_mc = library_docs[idx].1.clone();

            let button = Button::builder()
                .label("✓")
                .css_classes(["complete-btn"])
                .build();
            button.set_margin_start(4);

            button.connect_clicked(move |_| {
                mark_completed(&state_mc, &cards_mc, &doc_path_mc, idx);
            });

            hbox.append(&button);

            // Right-click gesture on the row
            let state_gc = state.clone();
            let cards_gc = cards.clone();
            let doc_path_gc = library_docs[idx].1.clone();

            let gesture = GestureClick::new();
            gesture.set_button(3);
            gesture.connect_pressed(move |_, _, _, _| {
                mark_completed(&state_gc, &cards_gc, &doc_path_gc, idx);
            });

            let (row, _, _) = &cards_ref[idx];
            row.add_controller(gesture);
        }
    }

    // Open book on activation
    book_list.connect_row_activated({
        let state = state.clone();
        let cards = cards.clone();
        move |_list, row| {
            if let Some(idx) = usize::try_from(row.index()).ok() {
                if idx >= cards.borrow().len() {
                    return;
                }

                let doc_path: std::path::PathBuf = {
                    let s = state.borrow();
                    let lib = s.library.as_ref().expect("library not loaded");
                    let doc: &crate::domain::document::Document = &lib.documents[idx];
                    doc.path.clone()
                };

                state.borrow_mut().mark_started(&doc_path);
                if let Some(entry) = cards.borrow_mut().get_mut(idx) {
                    if entry.2 == ReadingStatus::Unread {
                        entry.2 = ReadingStatus::CurrentlyReading;
                        entry.0.remove_css_class("unread");
                        entry.0.add_css_class("reading");
                    }
                }
                std::process::Command::new("xdg-open")
                    .arg(&doc_path)
                    .spawn()
                    .expect("Failed to open file");
            }
        }
    });

    // Wire sidebar selection to show/hide cards (no rebuild)
    let updating = Rc::new(Cell::new(false));

    sidebar.container.connect_row_selected({
        let cards = cards.clone();
        let updating = updating.clone();
        move |_list, row| {
            if updating.get() {
                return;
            }
            updating.set(true);

            let category = row.and_then(|r| match r.index() {
                0 => Some(Category::All),
                1 => Some(Category::CurrentlyReading),
                2 => Some(Category::Completed),
                _ => None,
            });

            if let Some(category) = category {
                for (card_row, _hbox, status) in cards.borrow().iter() {
                    let visible = match category {
                        Category::All => true,
                        Category::CurrentlyReading => {
                            *status == ReadingStatus::CurrentlyReading
                        }
                        Category::Completed => *status == ReadingStatus::Completed,
                    };
                    card_row.set_visible(visible);
                }
            }

            updating.set(false);
        }
    });

    content.append(&paned);

    paned.set_vexpand(true);
    paned.set_hexpand(true);
}
