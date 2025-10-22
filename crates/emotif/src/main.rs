use emojeez::EMOJIS;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Grid, Label, Orientation,
    ScrolledWindow, SearchEntry, gdk, glib,
};
use std::cell::RefCell;
use std::rc::Rc;
use unicode_types::Emoji;

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(env!("CARGO_PKG_NAME"))
        .default_width(800)
        .default_height(600)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);

    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search emojis..."));
    main_box.append(&search_entry);

    // Scrolled window for emoji grid
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_vexpand(true);

    let grid = Grid::new();
    grid.set_row_spacing(5);
    grid.set_column_spacing(5);
    grid.set_margin_top(10);

    scrolled_window.set_child(Some(&grid));
    main_box.append(&scrolled_window);

    let clipboard = window.clipboard();

    let button_cache: Rc<RefCell<Vec<Button>>> = Rc::new(RefCell::new(Vec::new()));

    populate_grid(&grid, EMOJIS, "", &clipboard, &button_cache);

    let search_generation: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

    search_entry.connect_search_changed({
        let grid = grid.clone();
        let clipboard = clipboard.clone();
        let button_cache = button_cache.clone();
        let search_generation = search_generation.clone();

        move |entry| {
            *search_generation.borrow_mut() += 1;
            let current_gen = *search_generation.borrow();

            let query = entry.text().to_string();
            let grid = grid.clone();
            let clipboard = clipboard.clone();
            let button_cache = button_cache.clone();
            let search_generation_inner = search_generation.clone();

            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                if *search_generation_inner.borrow() == current_gen {
                    populate_grid(&grid, EMOJIS, &query, &clipboard, &button_cache);
                }
            });
        }
    });

    window.set_child(Some(&main_box));
    window.present();
}

fn populate_grid(
    grid: &Grid,
    emojis: &[Emoji<&'static str, &'static [&'static str]>],
    query: &str,
    clipboard: &gdk::Clipboard,
    button_cache: &Rc<RefCell<Vec<Button>>>,
) {
    // Remove all children from grid
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    let columns = 8;
    let query_lower = query.to_lowercase();

    // Pre-filter emojis to avoid unnecessary iterations
    let filtered_emojis: Vec<_> = if query.is_empty() {
        emojis.iter().collect()
    } else {
        emojis
            .iter()
            .filter(|emoji| emoji.matches_search(&query_lower))
            .collect()
    };

    let mut cache = button_cache.borrow_mut();

    // Ensure we have enough buttons in cache
    while cache.len() < filtered_emojis.len() {
        let button = Button::new();
        let emoji_label = Label::new(None);
        emoji_label.set_css_classes(&["emoji-label"]);
        button.set_child(Some(&emoji_label));
        cache.push(button);
    }

    // Reuse existing buttons
    for (idx, emoji) in filtered_emojis.iter().enumerate() {
        let button = &cache[idx];

        // Update label
        if let Some(child) = button.child()
            && let Ok(label) = child.downcast::<Label>()
        {
            let markup = format!("<span font_desc='32'>{}</span>", emoji.entry.emoji);
            label.set_markup(&markup);
        }

        button.set_tooltip_text(Some(emoji.entry.name));

        // Disconnect previous signal handlers and connect new one
        let emoji_text = emoji.entry.emoji.to_string();
        let clipboard_clone = clipboard.clone();

        // Store signal handler id to avoid memory leaks
        button.connect_clicked(move |_| {
            clipboard_clone.set_text(&emoji_text);
        });

        let row = (idx / columns) as i32;
        let col = (idx % columns) as i32;
        grid.attach(button, col, row, 1, 1);
    }
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("com.github.martabal.emotif")
        .build();

    app.connect_activate(move |app| {
        build_ui(app);
    });

    app.run()
}
