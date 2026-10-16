//! Integration tests for the primary App::render v2 tab bar surface (Residual 2).
//!
//! Verifies that `App::render` produces the tab-bar + tab-body layout and that
//! switching tabs changes the rendered content.

use std::path::{Path, PathBuf};

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use vfa_tui::app::App;
use vfa_tui::catalog::store::CatalogStore;
use vfa_tui::ui::nav::{Tab, View};

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn app() -> App {
    let root = fixtures_root();
    let catalog = CatalogStore::load(&root);
    App::new(catalog, root, uuid::Uuid::new_v4(), true)
}

fn render_app_text(app: &mut App) -> String {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol().chars().next().unwrap_or(' '))
        .collect()
}

#[test]
fn primary_render_shows_tab_bar() {
    let mut app = app();
    let text = render_app_text(&mut app);
    // The tab bar title "Operator Console" or at least one tab label must appear.
    assert!(
        text.contains("Operator Console") || text.contains("Overview"),
        "primary render must show tab bar; first 200 chars: {:?}",
        &text[..200.min(text.len())]
    );
}

#[test]
fn switching_tabs_changes_active_body() {
    let mut app = app();
    // Start at Overview (default).
    assert_eq!(app.nav.current_tab, Tab::Overview);
    let text_overview = render_app_text(&mut app);

    // Switch to Dependencies tab.
    app.nav.current_tab = Tab::Dependencies;
    let text_deps = render_app_text(&mut app);

    // Content must differ between tabs.
    assert_ne!(
        text_overview, text_deps,
        "switching tabs must change rendered content"
    );
}

#[test]
fn all_tabs_render_without_panic() {
    for tab in Tab::ALL {
        let mut app = app();
        app.nav.current_tab = tab.clone();
        let text = render_app_text(&mut app);
        assert!(
            !text.trim().is_empty(),
            "tab {:?} produced empty frame in primary render",
            tab
        );
    }
}

/// Exercise the primary keyboard path, not a directly invoked detail widget.
#[test]
fn primary_catalog_mods_is_reachable_by_keyboard() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use vfa_tui::ui::nav::SIDEBAR_SECTIONS;

    let ws = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(ws.path().join("catalog")).unwrap();
    std::fs::write(ws.path().join("catalog/mods.json"), "[]").unwrap();
    let catalog = CatalogStore::load(ws.path());
    let mut app = App::new(catalog, ws.path().to_path_buf(), uuid::Uuid::new_v4(), true);
    let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
    for _ in 0..Tab::CatalogBrowser.index() {
        app.handle_key_event(key(KeyCode::Tab));
    }
    assert_eq!(app.nav.current_tab, Tab::CatalogBrowser);
    assert_eq!(app.nav.current_view, View::AgentList);
    assert_eq!(SIDEBAR_SECTIONS.last().unwrap().0, "Mods");

    // Left wraps the first sidebar section to the appended Mods section.
    app.handle_key_event(key(KeyCode::Left));
    assert_eq!(app.nav.sidebar_index, SIDEBAR_SECTIONS.len() - 1);
    let text = render_app_text(&mut app);
    assert!(text.contains("No mods"), "empty Mods must render: {text}");
    app.handle_key_event(key(KeyCode::Enter));
    assert!(app.nav.history.is_empty(), "empty list cannot drill in");
    app.handle_key_event(key(KeyCode::Right));
    assert_eq!(app.nav.current_view, View::AgentList);
    for _ in 1..SIDEBAR_SECTIONS.len() {
        app.handle_key_event(key(KeyCode::Right));
    }
    assert_eq!(
        app.nav.current_view,
        View::ModList,
        "builder sections must not trap sidebar keyboard navigation"
    );
}

fn mod_entry(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id, "name": format!("{id} name"), "version": "1.2.3",
        "description": "Catalog-only description", "reach_level": 3,
        "events": ["pre-tool-use"], "harnesses": ["claude-code", "codex"],
        "path": format!("mods/{id}"), "security_notes": "Review declared capabilities",
        "footprint": {
            "env_reads": ["VFA_TEST_ENV"], "file_writes": [".state/log.jsonl"],
            "spawns": ["node helper.mjs"], "egress_hosts": ["example.invalid"]
        }
    })
}

fn mods_app(entries: &[serde_json::Value]) -> (tempfile::TempDir, App) {
    let ws = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(ws.path().join("catalog")).unwrap();
    std::fs::write(
        ws.path().join("catalog/mods.json"),
        serde_json::to_vec(entries).unwrap(),
    )
    .unwrap();
    let catalog = CatalogStore::load(ws.path());
    let mut app = App::new(catalog, ws.path().to_path_buf(), uuid::Uuid::new_v4(), true);
    for _ in 0..Tab::CatalogBrowser.index() {
        press(&mut app, crossterm::event::KeyCode::Tab);
    }
    press(&mut app, crossterm::event::KeyCode::Left);
    (ws, app)
}

fn press(app: &mut App, code: crossterm::event::KeyCode) {
    app.handle_key_event(crossterm::event::KeyEvent::new(
        code,
        crossterm::event::KeyModifiers::NONE,
    ));
}

#[test]
fn primary_mods_list_detail_navigation_and_read_only_scope() {
    use crossterm::event::KeyCode;

    let (_ws, mut app) = mods_app(&[mod_entry("zeta"), mod_entry("alpha")]);
    app.provider_filter = Some("nonexistent-provider".to_string());
    app.harness_filter = Some("nonexistent-harness".to_string());
    let text = render_app_text(&mut app);
    for expected in ["alpha - alpha name [L3]", "zeta - zeta name [L3]"] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    press(&mut app, KeyCode::Down);
    assert_eq!(app.nav.selected_index(), 1);
    press(&mut app, KeyCode::Up);
    assert_eq!(app.nav.selected_index(), 0);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.nav.current_view, View::ModDetail("alpha".to_string()));
    let text = render_app_text(&mut app);
    for expected in [
        "ID: alpha",
        "Name: alpha name",
        "Version: 1.2.3",
        "Description: Catalog-only description",
        "Reach Level: L3",
        "Events: pre-tool-use",
        "Harnesses: claude-code, codex",
        "Path: mods/alpha",
        "Security Notes: Review declared capabilities",
        "Declared footprint",
        "env_reads: VFA_TEST_ENV",
        "file_writes: .state/log.jsonl",
        "spawns: node helper.mjs",
        "egress_hosts: example.invalid",
        "not enforced containment",
        "not live harness verification",
    ] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    press(&mut app, KeyCode::Char('m'));
    assert_eq!(app.nav.current_view, View::ModDetail("alpha".to_string()));
    assert!(app.subprocess_handle.is_none());
    assert!(app.pending_subprocess.is_none());
    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.nav.current_view,
        View::ModList,
        "Back ignores agent-only filters"
    );
    assert!(!app.should_quit);
}

#[test]
fn primary_mod_detail_distinguishes_unknown_and_empty_footprints() {
    use crossterm::event::KeyCode;
    for footprint in [None, Some(serde_json::Value::Null)] {
        let mut entry = mod_entry("legacy");
        if let Some(value) = footprint {
            entry["footprint"] = value;
        } else {
            entry.as_object_mut().unwrap().remove("footprint");
        }
        let (_ws, mut app) = mods_app(&[entry]);
        press(&mut app, KeyCode::Enter);
        let text = render_app_text(&mut app);
        assert!(text.contains("Declared footprint: not recorded"), "{text}");
        assert!(!text.contains("none declared"));
    }
    let mut entry = mod_entry("empty");
    entry["footprint"] = serde_json::json!({
        "env_reads": [], "file_writes": [], "spawns": [], "egress_hosts": []
    });
    let (_ws, mut app) = mods_app(&[entry]);
    press(&mut app, KeyCode::Enter);
    let text = render_app_text(&mut app);
    for label in ["env_reads", "file_writes", "spawns", "egress_hosts"] {
        assert!(text.contains(&format!("{label}: none declared")), "{text}");
    }
    assert!(!text.contains("not recorded"));
}

#[test]
fn primary_mod_detail_scrolls_long_metadata_and_back_resets_scroll() {
    use crossterm::event::KeyCode;

    let mut entry = mod_entry("long");
    entry["description"] = serde_json::Value::String("long description ".repeat(150));
    let (_ws, mut app) = mods_app(&[entry]);
    press(&mut app, KeyCode::Enter);
    let before = render_app_text(&mut app);
    assert!(!before.contains("egress_hosts: example.invalid"));
    for _ in 0..28 {
        press(&mut app, KeyCode::Down);
    }
    assert_eq!(app.nav.detail_scroll, 28);
    let after = render_app_text(&mut app);
    assert_ne!(before, after);
    assert!(after.contains("egress_hosts: example.invalid"), "{after}");
    press(&mut app, KeyCode::Up);
    assert_eq!(app.nav.detail_scroll, 27);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.current_view, View::ModList);
    assert_eq!(app.nav.detail_scroll, 0);
}
