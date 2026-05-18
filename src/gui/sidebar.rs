use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::*;

// These match the three rows in the sidebar. The library view uses
// them to decide which cards should be visible at any given time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    All,
    CurrentlyReading,
    Completed,
}

impl Category {
    fn label(&self) -> &'static str {
        match self {
            Category::All => "All Books",
            Category::CurrentlyReading => "Currently Reading",
            Category::Completed => "Completed",
        }
    }
}

// Three-row sidebar that lets the user switch between seeing every
// book, only the ones they are currently reading, or only the ones
// they have marked as finished.
pub struct Sidebar {
    pub container: ListBox,
    pub active: Rc<Cell<Category>>,
}

impl Sidebar {
    pub fn new() -> Self {
        let container = ListBox::new();
        container.set_css_classes(&["sidebar"]);
        container.set_selection_mode(SelectionMode::Single);

        let active = Rc::new(Cell::new(Category::All));

        for category in [Category::All, Category::CurrentlyReading, Category::Completed] {
            let label = Label::builder()
                .label(category.label())
                .halign(Align::Start)
                .css_classes(["sidebar-label"])
                .build();

            let row = ListBoxRow::new();
            row.set_child(Some(&label));
            container.append(&row);
        }

        // Select the first row (All Books) by default
        if let Some(first) = container.first_child() {
            container.select_row(first.downcast::<ListBoxRow>().ok().as_ref());
        }

        container.connect_row_selected({
            let active = active.clone();
            move |_list, row| {
                if let Some(row) = row {
                    let index = row.index();
                    let category = match index {
                        0 => Category::All,
                        1 => Category::CurrentlyReading,
                        2 => Category::Completed,
                        _ => return,
                    };
                    active.set(category);
                }
            }
        });

        Self { container, active }
    }

}
