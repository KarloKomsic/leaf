use gtk4::CssProvider;
use gtk4::gdk::Display;

pub fn load() {
    let provider = CssProvider::new();

    provider.load_from_string(
        r#"
        .sidebar {
            background-color: @theme_bg_color;
            border-right: 1px solid @borders;
            min-width: 180px;
        }

        .sidebar-label {
            padding: 8px 16px;
            font-weight: 600;
            color: @theme_fg_color;
        }

        .sidebar-label:selected {
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }

        .book-card {
            background-color: @theme_base_color;
            border: 1px solid @borders;
            border-radius: 8px;
            padding: 8px;
            margin: 4px;
            min-width: 220px;
            min-height: 88px;
            max-height: 88px;
        }

        .book-card:hover {
            background-color: @theme_hover_bg_color;
        }

        .book-card.unread {
            border-color: @borders;
        }

        .book-card.reading {
            border-color: #e5a50a;
            background-color: rgba(229, 165, 10, 0.06);
        }

        .book-card.completed {
            border-color: #26a269;
            background-color: rgba(38, 162, 105, 0.06);
        }

        .book-title {
            font-size: 14px;
            font-weight: 700;
            margin-bottom: 2px;
        }

        .book-author {
            font-size: 12px;
            color: @theme_dim_label_color;
        }

        .cover-placeholder {
            background-color: @theme_shade_color;
            border-radius: 4px;
        }

        .card-box {
            padding: 0;
        }

        .complete-btn {
            min-width: 32px;
            min-height: 32px;
            padding: 0;
            font-size: 16px;
            font-weight: bold;
            border-radius: 16px;
        }

        .complete-btn:hover {
            background-color: #26a269;
            color: white;
        }

        .context-btn {
            padding: 4px 12px;
            border: none;
            background: transparent;
            min-width: 160px;
        }

        .context-btn:hover {
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }

        .menu-item {
            padding: 4px 12px;
            border: none;
            background: transparent;
            min-width: 120px;
        }

        .menu-item:hover {
            background-color: @theme_selected_bg_color;
            color: @theme_selected_fg_color;
        }

        .welcome-wrapper {
            margin-top: 0;
        }

        .welcome-title {
            font-size: 28px;
            font-weight: 700;
            margin-top: 8px;
            margin-bottom: 2px;
        }

        .welcome-subtitle {
            font-size: 13px;
            color: @theme_dim_label_color;
            margin-bottom: 16px;
        }

        .welcome-button {
            min-width: 200px;
            padding: 8px 20px;
            font-size: 13px;
        }

        .settings-section {
            margin-bottom: 16px;
        }

        .settings-label {
            font-weight: 600;
            margin-bottom: 4px;
        }
        "#,
    );

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
