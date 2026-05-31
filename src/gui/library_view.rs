use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::*;

use crate::application::app_state::AppState;
use crate::domain::library_collection::Library;
use crate::domain::reading_status::ReadingStatus;

use super::card;
use super::sidebar::{Category, Sidebar};

/// Rebuilds the book list so only matching cards are in the container.
/// Cards that fail the filter are pulled out completely so they
/// cannot affect any layout calculation.
fn update_card_visibility(
    cards: &[(ListBoxRow, Box, ReadingStatus, Button)],
    library: &Library,
    category: Category,
    query: &str,
    scroll_books: &ScrolledWindow,
) {
    let search_matches: Option<HashSet<usize>> = if query.is_empty() {
        None
    } else if query.len() <= 2 {
        let suggestions = library.suggest(query);
        Some(
            suggestions
                .iter()
                .map(|doc| {
                    library
                        .documents
                        .iter()
                        .position(|d| d.path == doc.path)
                        .unwrap()
                })
                .collect(),
        )
    } else {
        let results = library.search(query);
        Some(
            results
                .iter()
                .map(|r| {
                    library
                        .documents
                        .iter()
                        .position(|d| d.path == r.document.path)
                        .unwrap()
                })
                .collect(),
        )
    };

    let mut visible_count = 0;
    for (idx, (_card_row, _hbox, status, _btn)) in cards.iter().enumerate() {
        let cat_match = match category {
            Category::All => true,
            Category::CurrentlyReading => *status == ReadingStatus::CurrentlyReading,
            Category::Completed => *status == ReadingStatus::Completed,
        };

        let search_match = match &search_matches {
            None => true,
            Some(matches) => matches.contains(&idx),
        };

        if cat_match && search_match {
            visible_count += 1;
        }
    }

    // Apply ScrolledWindow constraints BEFORE changing visibility,
    // so GTK doesn't process a layout pass with stale caps from
    // the previous category.
    let card_total_height = 114;
    let content_height = visible_count as i32 * card_total_height;

    if visible_count == 0 {
        // Give empty categories a non-zero max, so that switching to
        // a category with visible cards uses a fresh max instead of 0.
        scroll_books.set_vexpand(false);
        scroll_books.set_min_content_height(0);
        scroll_books.set_max_content_height(400);
    } else if content_height <= 700 {
        // Content fits comfortably in the viewport (~6 cards):
        // cap the scroll window so it never allocates more than needed.
        scroll_books.set_vexpand(false);
        scroll_books.set_min_content_height(content_height);
        scroll_books.set_max_content_height(content_height);
    } else {
        scroll_books.set_vexpand(true);
        scroll_books.set_min_content_height(-1);
        scroll_books.set_max_content_height(-1);
    }

    // Now apply visibility after the constraints are in place.
    for (idx, (card_row, _hbox, status, _btn)) in cards.iter().enumerate() {
        let cat_match = match category {
            Category::All => true,
            Category::CurrentlyReading => *status == ReadingStatus::CurrentlyReading,
            Category::Completed => *status == ReadingStatus::Completed,
        };

        let search_match = match &search_matches {
            None => true,
            Some(matches) => matches.contains(&idx),
        };

        card_row.set_visible(cat_match && search_match);
    }
}

/// Shows a right-click popover over the card. The options change
/// based on the current status:
///   Completed: un-mark it back to Unread.
///   CurrentlyReading: mark as Completed, or move back to Unread.
///   Unread: mark as Completed.
fn show_context_menu(
    state: &Rc<RefCell<AppState>>,
    cards: &Rc<RefCell<Vec<(ListBoxRow, Box, ReadingStatus, Button)>>>,
    idx: usize,
    x: f64,
    y: f64,
) {
    let (row, status) = {
        let cards_ref = cards.borrow();
        let entry = &cards_ref[idx];
        (entry.0.clone(), entry.2.clone())
    };

    let popover = Popover::new();
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));

    let vbox = Box::new(Orientation::Vertical, 0);
    vbox.set_margin_top(4);
    vbox.set_margin_bottom(4);

    let add_mark_completed = |vb: &Box| {
        let btn = Button::with_label("Mark as Completed");
        btn.add_css_class("context-btn");
        let state_c = state.clone();
        let cards_c = cards.clone();
        let popover_c = popover.clone();
        btn.connect_clicked(move |_| {
            let path = {
                let s = state_c.borrow();
                let lib = s.library.as_ref().unwrap();
                lib.documents[idx].path.clone()
            };
            state_c.borrow_mut().mark_completed(&path);
            if let Some(entry) = cards_c.borrow_mut().get_mut(idx) {
                entry.2 = ReadingStatus::Completed;
                entry.0.remove_css_class("reading");
                entry.0.remove_css_class("unread");
                entry.0.add_css_class("completed");
                entry.3.set_visible(false);
            }
            popover_c.popdown();
        });
        vb.append(&btn);
    };

    let add_mark_unread = |vb: &Box, label: &str| {
        let btn = Button::with_label(label);
        btn.add_css_class("context-btn");
        let state_c = state.clone();
        let cards_c = cards.clone();
        let popover_c = popover.clone();
        btn.connect_clicked(move |_| {
            let path = {
                let s = state_c.borrow();
                let lib = s.library.as_ref().unwrap();
                lib.documents[idx].path.clone()
            };
            state_c.borrow_mut().mark_uncompleted(&path);
            if let Some(entry) = cards_c.borrow_mut().get_mut(idx) {
                entry.2 = ReadingStatus::Unread;
                entry.0.remove_css_class("reading");
                entry.0.remove_css_class("completed");
                entry.0.add_css_class("unread");
                entry.3.set_visible(true);
            }
            popover_c.popdown();
        });
        vb.append(&btn);
    };

    match status {
        ReadingStatus::Completed => {
            add_mark_unread(&vbox, "Un-mark this book as Completed");
        }
        ReadingStatus::CurrentlyReading => {
            add_mark_completed(&vbox);
            add_mark_unread(&vbox, "Move back to Unread");
        }
        ReadingStatus::Unread => {
            add_mark_completed(&vbox);
        }
    }

    popover.set_child(Some(&vbox));
    popover.set_parent(&row);
    popover.popup();
}

/// Opens a book at the given card index: saves the started-reading
/// timestamp and launches the system PDF viewer.
fn open_book(
    state: &Rc<RefCell<AppState>>,
    cards: &Rc<RefCell<Vec<(ListBoxRow, Box, ReadingStatus, Button)>>>,
    idx: usize,
) {
    let doc_path: std::path::PathBuf = {
        let s = state.borrow();
        let lib = s.library.as_ref().expect("library not loaded");
        lib.documents[idx].path.clone()
    };

    state.borrow_mut().mark_started(&doc_path);
    if let Some(entry) = cards.borrow_mut().get_mut(idx) {
        if entry.2 == ReadingStatus::Unread {
            entry.2 = ReadingStatus::CurrentlyReading;
            entry.0.remove_css_class("unread");
            entry.0.add_css_class("reading");
        }
    }
    let viewer = state.borrow().pdf_viewer_or_default().to_string();
    if let Err(e) = std::process::Command::new(&viewer).arg(&doc_path).spawn() {
        eprintln!("Failed to open file with {}: {}", viewer, e);
    }
}

/// Lays out the main library screen: sidebar on the left, search bar
/// and scrollable book list on the right. Every book gets a card,
/// and the cards hide and show based on the active sidebar tab and
/// whatever the user types in the search box.
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

    // Sidebar panel
    let sidebar = Sidebar::new();
    let scroll_sidebar = ScrolledWindow::builder()
        .child(&sidebar.container)
        .min_content_width(200)
        .build();
    paned.set_start_child(Some(&scroll_sidebar));
    paned.set_resize_start_child(false);
    paned.set_shrink_start_child(false);

    // Right panel: search bar on top, scrollable book list underneath
    let right_panel = Box::new(Orientation::Vertical, 0);

    let search_entry = SearchEntry::builder()
        .placeholder_text("Search books...")
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    right_panel.append(&search_entry);

    // Non-matching cards are hidden with set_visible(false) rather than
    // removed, because the ListBox is Scrollable — no Viewport wrapper
    // means the ScrolledWindow never allocates extra space to it.
    let book_list = ListBox::new();
    book_list.set_selection_mode(SelectionMode::None);
    book_list.set_activate_on_single_click(false);
    book_list.set_valign(Align::Start);
    book_list.set_vexpand(false);

    let scroll_books = ScrolledWindow::builder()
        .child(&book_list)
        .vexpand(true)
        .hexpand(true)
        .build();
    right_panel.append(&scroll_books);

    paned.set_end_child(Some(&right_panel));

    let cards: Rc<RefCell<Vec<(ListBoxRow, Box, ReadingStatus, Button)>>> =
        Rc::new(RefCell::new(Vec::new()));

    // Build every card with a complete button. For books that are
    // already finished the button starts hidden so every row has
    // the same layout regardless of reading status.
    {
        let s = state.borrow();
        let library = match &s.library {
            Some(lib) => lib,
            None => return,
        };

        let mut entries = Vec::with_capacity(library.documents.len());

        for (idx, doc) in library.documents.iter().enumerate() {
            let status = s.get_reading_status(&doc.path);
            let (row, hbox) = card::create(doc, &status);
            book_list.insert(&row, -1);

            let button = Button::builder()
                .label("\u{2713}")
                .css_classes(["complete-btn"])
                .build();
            button.set_margin_start(4);

            if status == ReadingStatus::Completed {
                button.set_visible(false);
            } else {
                let state_mc = state.clone();
                let cards_mc = cards.clone();
                let doc_path_mc = doc.path.clone();
                button.connect_clicked(move |_| {
                    state_mc.borrow_mut().mark_completed(&doc_path_mc);
                    if let Some(entry) = cards_mc.borrow_mut().get_mut(idx) {
                        entry.2 = ReadingStatus::Completed;
                        entry.0.remove_css_class("reading");
                        entry.0.remove_css_class("unread");
                        entry.0.add_css_class("completed");
                        entry.3.set_visible(false);
                    }
                });
            }

            hbox.append(&button);
            row.set_tooltip_text(Some("Double-click to open this book"));

            // Double-click opens the book (single-click might hit the
            // check button, so we use two clicks to avoid conflicts).
            let open_state = state.clone();
            let open_cards = cards.clone();
            let click = GestureClick::new();
            click.set_button(1);
            click.connect_pressed(move |_, n_press, _, _| {
                if n_press == 2 {
                    open_book(&open_state, &open_cards, idx);
                }
            });
            row.add_controller(click);

            // Right-click shows the context menu
            let state_gc = state.clone();
            let cards_gc = cards.clone();
            let gesture = GestureClick::new();
            gesture.set_button(3);
            gesture.connect_pressed(move |_, _, x, y| {
                show_context_menu(&state_gc, &cards_gc, idx, x, y);
            });
            row.add_controller(gesture);

            entries.push((row, hbox, status, button));
        }

        *cards.borrow_mut() = entries;
    }

    // When the sidebar selection changes, fully rebuild which cards
    // are in the book list. The search text is also taken into account.
    let search_query: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    sidebar.container.connect_row_selected({
        let cards = cards.clone();
        let state = state.clone();
        let search_query = search_query.clone();
        let scroll_books = scroll_books.clone();
        move |_list, row| {
            let category = row.and_then(|r| match r.index() {
                0 => Some(Category::All),
                1 => Some(Category::CurrentlyReading),
                2 => Some(Category::Completed),
                _ => None,
            });

            if let Some(category) = category {
                let s = state.borrow();
                if let Some(library) = &s.library {
                    let cards_ref = cards.borrow();
                    let query = search_query.borrow();
                    update_card_visibility(&cards_ref, library, category, &query, &scroll_books);
                }
            }
        }
    });

    // Real-time search filtering. Every keystroke updates the results.
    search_entry.connect_search_changed({
        let cards = cards.clone();
        let state = state.clone();
        let search_query = search_query.clone();
        let sidebar_active = sidebar.active.clone();
        let scroll_books = scroll_books.clone();
        move |entry| {
            let query = entry.text().to_string();
            *search_query.borrow_mut() = query.clone();
            let s = state.borrow();
            if let Some(library) = &s.library {
                let category = sidebar_active.get();
                let cards_ref = cards.borrow();
                update_card_visibility(&cards_ref, library, category, &query, &scroll_books);
            }
        }
    });

    content.append(&paned);

    paned.set_vexpand(true);
    paned.set_hexpand(true);
}
