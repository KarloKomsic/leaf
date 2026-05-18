// Renders a single book as a list row with a cover thumbnail on the
// left, the title and author in the middle, and room for a complete
// button on the right. The border color changes based on the reading
// status: default for unread, gold for in progress, green for finished.

use gtk4::prelude::*;
use gtk4::*;

use crate::domain::document::Document;
use crate::domain::reading_status::ReadingStatus;
use crate::infrastructure::cover_cache;

pub fn create(doc: &Document, status: &ReadingStatus) -> (ListBoxRow, Box) {
    let row = ListBoxRow::new();
    row.set_valign(Align::Start);
    row.set_vexpand(false);
    row.set_css_classes(&["book-card"]);

    let extra_class = match status {
        ReadingStatus::Unread => "unread",
        ReadingStatus::CurrentlyReading => "reading",
        ReadingStatus::Completed => "completed",
    };
    row.add_css_class(extra_class);

    let hbox = Box::new(Orientation::Horizontal, 8);
    hbox.set_valign(Align::Start);
    hbox.set_margin_top(4);
    hbox.set_margin_bottom(4);

    // Cover thumbnail or a grey placeholder box if we couldn't
    // generate one (e.g. pdftoppm is not installed)
    if let Some(cover_path) = cover_cache::cached_cover_path(&doc.path) {
        let picture = Picture::for_filename(&cover_path);
        picture.set_size_request(60, 80);
        picture.set_content_fit(ContentFit::Cover);
        picture.set_valign(Align::Start);
        hbox.append(&picture);
    } else {
        let placeholder = Frame::new(None);
        placeholder.set_css_classes(&["cover-placeholder"]);
        placeholder.set_size_request(60, 80);
        placeholder.set_valign(Align::Start);
        hbox.append(&placeholder);
    }

    // Title and author stacked vertically, taking up remaining space
    let vbox = Box::new(Orientation::Vertical, 2);
    vbox.set_valign(Align::Center);
    vbox.set_hexpand(true);

    let title = Label::builder()
        .label(&doc.title)
        .halign(Align::Start)
        .wrap(true)
        .css_classes(["book-title"])
        .build();
    vbox.append(&title);

    let author = Label::builder()
        .label(doc.author.as_deref().unwrap_or("Unknown Author"))
        .halign(Align::Start)
        .css_classes(["book-author"])
        .build();
    vbox.append(&author);

    hbox.append(&vbox);
    row.set_child(Some(&hbox));

    (row, hbox)
}
