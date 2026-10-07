# Leaf Roadmap for 1.2

Work is grouped into phases. Each phase ends with `cargo fmt`, `cargo clippy -- -D warnings`,
and `cargo test` passing, plus a review before moving on. Bug fixes get a unit test in the
same file.

## Phase 1: Portability and tooling

- [ ] Lower gtk4 feature from `v4_22` to `v4_10` (build currently needs GTK >= 4.21, so Debian 13, Ubuntu 24.04 and Fedora 42 cannot build it despite README claims)
- [ ] Cargo.toml metadata: `license`, `description`, `repository`, `readme`, `rust-version`
- [ ] Add approved crates: `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`, `tempfile` (dev)
- [ ] Add `#![forbid(unsafe_code)]` to `src/main.rs` and convert `main()` to `anyhow::Result`
- [ ] Apply `cargo fmt` (17 diffs across 7 files)
- [ ] Fix all 19 clippy lints
- [ ] Add CI workflow (fmt check, clippy -D warnings, test, build with GTK headers)

## Phase 2: Critical bug fixes

- [ ] Atomic and dirty-checked saves: temp file + rename, `save()` returns `Result`, no save on load (a crash mid-write truncates `reading_status.json` and the next load silently wipes all progress)
- [ ] Corrupt files are preserved: rename to `*.corrupt` instead of clobbering; `main()` must not overwrite a broken config with defaults
- [ ] Opening a Completed book must not reset its status to CurrentlyReading (CLI and GUI share the bug; GUI card and store also desync)
- [ ] Settings dropdown panic: `DropDown.selected` defaults to `GTK_INVALID_LIST_POSITION`, so Save indexes `viewers[u32::MAX]` when the configured viewer is missing from the list (also silently clobbers custom viewers)
- [ ] Scanner symlink loop: use `DirEntry::file_type()` (does not follow symlinks); a symlink to an ancestor hangs the scan forever
- [ ] Cache prune safety: never prune the whole metadata cache when the root `read_dir` fails (one failed scan currently forces re-extraction of every book)
- [ ] Invalid library directory must not be a silent empty session: CLI re-prompts with the dead path, GUI shows a notice
- [ ] Drop `Vec::with_capacity(file.size())` on untrusted zip entries in `epub.rs` and `cover_cache.rs` (garbage central directory can mean a multi-GB allocation)
- [ ] Replace CSS variables that do not exist in GTK4: `@theme_hover_bg_color`, `@theme_dim_label_color`, `@theme_shade_color` (use `alpha(@theme_fg_color, 0.55)` etc.); hook `parsing-error` to tracing
- [ ] Canonicalize the library directory on save (relative and `~` paths currently produce cwd-dependent cache/status keys)

## Phase 3: Robustness and performance

- [ ] GUI startup: show the window first, run scan and cover generation off the main thread with idle callbacks back to the UI
- [ ] Load covers asynchronously per card instead of spawning `pdftoppm` while building the list
- [ ] Search performance: precompute normalized title/acronym/author at index build, debounce GUI input (currently O(library) with allocations on every keystroke)
- [ ] Normalize queries through the same pipeline as titles (queries like `the-hobbit` currently return nothing)
- [ ] Cover cache: stable hash instead of `DefaultHasher`, mtime in the key (stale covers after file changes), atomic writes, prune orphans at startup
- [ ] Shared opener (`infrastructure/opener.rs`) used by CLI and GUI; reap child processes (viewer processes currently become zombies)
- [ ] Replace stray `println!`/`eprintln!`/`let _ = fs::write` with tracing and surfaced errors

## Phase 4: Maintainability refactor

- [ ] `struct Card` in the GUI replacing the `(ListBoxRow, Box, ReadingStatus, Button)` tuple (the `Box` is dead) plus one shared `set_card_status()` replacing three copies of the status/CSS update
- [ ] Single `Category::from_index()` (mapping is duplicated in `sidebar.rs` and `library_view.rs`)
- [ ] One OPF/container reader shared by metadata extraction and cover cache; add URL-decoding, `../` resolution and EPUB3 `properties="cover-image"` fallback (many EPUB3 covers are currently missed)
- [ ] Renames: `library_collection.rs` to `search.rs`, `library.rs` to `library_state.rs`, `mark_uncompleted` to `mark_unread`, `LibraryPath(String)` to `PathBuf`-backed, private `CliApp.state`
- [ ] Idiom pass: derive `Default`/`Copy`/`Eq`, `or_default()`, let-chains, `Display for MatchType`, `open_file(&Path)`, `as_deref()` instead of clones
- [ ] XDG paths via `dirs`: status to `$XDG_STATE_HOME/leaf/`, metadata cache to `$XDG_CACHE_HOME/leaf/`, no `/tmp` fallback, one-time migration for existing users
- [ ] thiserror error types with real messages (`ConfigError` currently discards causes)

## Phase 5: Tests, packaging, docs

- [ ] Integration tests with `tempfile`: scanner (symlink loop, unreadable dir, prune), metadata cache (freshness, prune, corruption), status store round-trip, config load/save/backup
- [ ] `install.sh`: never run `cargo build` as root (root-owned `target/` breaks later user builds), uninstall honors the system prefix, drop `eval`
- [ ] `flake.nix`: install the real `com.leaf.app` icon and PNGs, fix desktop-item icon name, wrap PATH with poppler so `pdftoppm`/`pdfinfo` exist under `nix run`, remove the no-op `postInstall`, version from Cargo.toml
- [ ] Docs: valid `- [ ]` checkboxes, README mentions `pdfinfo`, accurate distro/GTK support, drop the redundant pre-install `cargo build`

## Future features (after Phase 5)

- [ ] Rescan/refresh button or file watcher (GUI currently needs a restart to see new books)
- [ ] Use `last_opened` (already stored, never read): "Recently opened" category and sort options
- [ ] Book details pane and richer metadata: series, year, pages, publisher, description
- [ ] Cover grid view (Calibre-style) alongside the list
- [ ] Non-interactive CLI: `leaf open <query>`, `leaf random`, `leaf list --status reading`, `--help`, `--version`
- [ ] GUI keyboard support (Ctrl+F, arrow navigation, Enter to open, Escape to clear) and an About dialog
- [ ] Reveal in file manager and open-with-another-viewer
- [ ] CBZ/CBR support (the `zip` crate is already a dependency)
- [ ] Theme switcher (Nord, Catppuccin, Dracula, Solarized, Gruvbox, Everforest, Tokyo Night, Kanagawa, Rose Pine) after the CSS variable fix
- [ ] Tags/favorites, duplicate detection, multiple library roots
- [ ] Reading progress percentage and started/completed dates
- [ ] Export reading list (CSV/JSON)
