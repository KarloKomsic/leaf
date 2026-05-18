use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::*;

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

// Three-row sidebar: All Books, Currently Reading, Completed.
// Selection changes tell library_view which cards to show/hide.
pub struct Sidebar {
    pub container: ListBox,
    active: Rc<Cell<Category>>,
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

    pub fn active_category(&self) -> Category {
        self.active.get()
    }
}
