//! Integration test for the TUI live-reload methods wired into the watcher
//! event loop (Task 9.1). The `tokio::select!` plumbing is exercised at runtime;
//! here we verify the reload behavior the watcher branch invokes.
//! Validates: Requirements 4.x (live reload), 5.x (safe rollback on parse error)

use std::path::{Path, PathBuf};

use vfa_tui::app::App;
use vfa_tui::catalog::store::{CatalogStore, ReloadOutcome};

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Copy `tests/fixtures/catalog/*.json` into a fresh temp workspace root.
fn temp_workspace() -> tempfile::TempDir {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = fixtures_root().join("catalog");
    let dst = tmp.path().join("catalog");
    std::fs::create_dir_all(&dst).unwrap();
    for entry in std::fs::read_dir(&src).unwrap() {
        let e = entry.unwrap();
        std::fs::copy(e.path(), dst.join(e.file_name())).unwrap();
    }
    tmp
}

fn app_for(ws: &Path) -> App {
    let catalog = CatalogStore::load(ws);
    App::new(catalog, ws.to_path_buf(), uuid::Uuid::new_v4(), true)
}

#[test]
fn reload_catalog_picks_up_deleted_file() {
    let ws = temp_workspace();
    let mut app = app_for(ws.path());
    assert_eq!(app.catalog.skill_count(), 3);

    // Remove skills.json on disk, then reload the whole catalog.
    std::fs::remove_file(ws.path().join("catalog").join("skills.json")).unwrap();
    app.reload_catalog();

    assert_eq!(
        app.catalog.skill_count(),
        0,
        "deleted skills picked up on reload"
    );
    assert!(app.dirty, "reload marks the app dirty for re-render");
}

#[test]
fn reload_catalog_file_retains_previous_on_parse_error() {
    let ws = temp_workspace();
    let mut app = app_for(ws.path());
    let before = app.catalog.agent_count();
    assert_eq!(before, 5);

    // Corrupt agents.json, then reload just that file.
    let agents = ws.path().join("catalog").join("agents.json");
    std::fs::write(&agents, b"{ this is not valid json").unwrap();
    let outcome = app.reload_catalog_file(&agents);

    assert!(
        matches!(outcome, ReloadOutcome::RetainedPrevious { .. }),
        "a parse error must retain the previous good catalog, got {outcome:?}"
    );
    assert_eq!(
        app.catalog.agent_count(),
        5,
        "previous agents retained on parse error"
    );
}

#[test]
fn reload_catalog_file_unchanged_is_noop() {
    let ws = temp_workspace();
    let mut app = app_for(ws.path());
    let agents = ws.path().join("catalog").join("agents.json");

    // No modification since load → the content hash matches → Unchanged.
    let outcome = app.reload_catalog_file(&agents);
    assert!(
        matches!(outcome, ReloadOutcome::Unchanged),
        "unmodified file should be Unchanged, got {outcome:?}"
    );
    assert_eq!(app.catalog.agent_count(), 5);
}

fn write_mods(ws: &Path, entries: &[(&str, &str)]) -> PathBuf {
    let path = ws.join("catalog/mods.json");
    let values: Vec<_> = entries
        .iter()
        .map(|(id, description)| {
            serde_json::json!({
                "id": id, "name": format!("{id} name"), "version": "1.0.0",
                "description": description, "reach_level": 1,
                "events": ["pre-tool-use"], "harnesses": ["codex"],
                "path": format!("mods/{id}"),
                "footprint": {"env_reads": [], "file_writes": [], "spawns": [], "egress_hosts": []}
            })
        })
        .collect();
    std::fs::write(&path, serde_json::to_vec(&values).unwrap()).unwrap();
    path
}

fn mods_key(app: &mut App, code: crossterm::event::KeyCode) {
    app.handle_key_event(crossterm::event::KeyEvent::new(
        code,
        crossterm::event::KeyModifiers::NONE,
    ));
}

fn open_mods(app: &mut App) {
    use crossterm::event::KeyCode;
    for _ in 0..vfa_tui::ui::nav::Tab::CatalogBrowser.index() {
        mods_key(app, KeyCode::Tab);
    }
    mods_key(app, KeyCode::Left);
}

fn mods_frame(app: &mut App) -> String {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 30)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

#[test]
fn actual_app_mod_reload_clamps_selection_and_deliberate_empty_clears() {
    use crossterm::event::KeyCode;
    let ws = temp_workspace();
    let path = write_mods(
        ws.path(),
        &[("alpha", "first"), ("beta", "second"), ("zeta", "last")],
    );
    let mut app = app_for(ws.path());
    open_mods(&mut app);
    mods_key(&mut app, KeyCode::Char('G'));
    assert_eq!(app.nav.selected_index(), 2);
    write_mods(ws.path(), &[("alpha", "updated description")]);
    assert!(
        matches!(app.reload_catalog_file(&path), ReloadOutcome::Reloaded { catalog } if catalog == "mods")
    );
    assert_eq!(app.nav.selected_index(), 0);
    assert!(mods_frame(&mut app).contains("alpha - alpha name [L1]"));
    mods_key(&mut app, KeyCode::Enter);
    assert!(mods_frame(&mut app).contains("updated description"));
    mods_key(&mut app, KeyCode::Esc);
    write_mods(ws.path(), &[]);
    assert!(matches!(
        app.reload_catalog_file(&path),
        ReloadOutcome::Reloaded { .. }
    ));
    assert!(app.nav.list_state.selected().is_none());
    assert!(mods_frame(&mut app).contains("No mods"));
    mods_key(&mut app, KeyCode::Enter);
    assert!(app.nav.history.is_empty());
}

#[test]
fn actual_app_mod_detail_reload_updates_retains_corrects_and_handles_removed_id() {
    use crossterm::event::KeyCode;
    let ws = temp_workspace();
    let path = write_mods(ws.path(), &[("alpha", "original description")]);
    let mut app = app_for(ws.path());
    open_mods(&mut app);
    mods_key(&mut app, KeyCode::Enter);
    assert!(mods_frame(&mut app).contains("original description"));
    let accepted_hash = app.catalog.content_hashes.get(&path).cloned();
    assert!(accepted_hash.is_some());
    std::fs::write(&path, "{broken").unwrap();
    for _ in 0..2 {
        assert!(matches!(
            app.reload_catalog_file(&path),
            ReloadOutcome::RetainedPrevious { .. }
        ));
        assert_eq!(
            app.catalog.content_hashes.get(&path),
            accepted_hash.as_ref()
        );
        assert!(mods_frame(&mut app).contains("original description"));
        assert!(app
            .status_message
            .as_ref()
            .is_some_and(|(message, _)| message.contains("mods")));
    }
    write_mods(ws.path(), &[("alpha", "corrected description")]);
    app.nav.detail_scroll = 100;
    assert!(matches!(
        app.reload_catalog_file(&path),
        ReloadOutcome::Reloaded { .. }
    ));
    assert_eq!(
        app.nav.detail_scroll, 0,
        "changed detail must not remain scrolled past new metadata"
    );
    assert!(mods_frame(&mut app).contains("corrected description"));
    write_mods(ws.path(), &[("beta", "replacement description")]);
    assert!(matches!(
        app.reload_catalog_file(&path),
        ReloadOutcome::Reloaded { .. }
    ));
    assert!(mods_frame(&mut app).contains("Mod no longer present"));
    mods_key(&mut app, KeyCode::Esc);
    assert!(mods_frame(&mut app).contains("beta - beta name [L1]"));
    assert!(!app.should_quit);
}

#[test]
fn actual_app_full_mod_reload_retains_missing_or_invalid_then_recovers() {
    use crossterm::event::KeyCode;
    let ws = temp_workspace();
    let path = write_mods(ws.path(), &[("alpha", "last valid description")]);
    let mut app = app_for(ws.path());
    open_mods(&mut app);
    mods_key(&mut app, KeyCode::Enter);
    let accepted_hash = app.catalog.content_hashes.get(&path).cloned();
    assert!(accepted_hash.is_some());
    std::fs::remove_file(&path).unwrap();
    app.reload_catalog();
    assert!(mods_frame(&mut app).contains("last valid description"));
    assert_eq!(
        app.catalog.content_hashes.get(&path),
        accepted_hash.as_ref()
    );
    assert!(app.status_message.as_ref().unwrap().0.contains("mods"));
    std::fs::write(&path, "{broken").unwrap();
    app.reload_catalog();
    assert!(mods_frame(&mut app).contains("last valid description"));
    assert_eq!(
        app.catalog.content_hashes.get(&path),
        accepted_hash.as_ref()
    );
    write_mods(ws.path(), &[("alpha", "full reload corrected")]);
    app.reload_catalog();
    assert!(mods_frame(&mut app).contains("full reload corrected"));
    write_mods(ws.path(), &[]);
    app.reload_catalog();
    assert!(mods_frame(&mut app).contains("Mod no longer present"));
    mods_key(&mut app, KeyCode::Esc);
    assert!(mods_frame(&mut app).contains("No mods"));
    assert!(app.nav.list_state.selected().is_none());
}
