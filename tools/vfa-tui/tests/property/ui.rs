use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use proptest::prelude::*;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use uuid::Uuid;
use vfa_tui::app::App;
use vfa_tui::catalog::store::CatalogStore;
use vfa_tui::models::{Agent, AgentType, ExecutionTier, Harness, Lifecycle, Provider, SourceType};
use vfa_tui::ui::theme::{Theme, ThemeMode};
use vfa_tui::ui::widgets::detail::{render_agent_detail, AGENT_DETAIL_REQUIRED_LABELS};

fn arbitrary_agent(
    id: String,
    name: String,
    summary: String,
    version: Option<String>,
    author: Option<String>,
    execution_tier: Option<ExecutionTier>,
    lifecycle: Option<Lifecycle>,
) -> Agent {
    Agent {
        id,
        name,
        entity_type: AgentType::Agent,
        provider: Provider::Aws,
        harnesses: vec![Harness::Kiro],
        summary,
        companion_skills: vec!["skill-a".to_string()],
        source_type: SourceType::Original,
        official_docs: vec!["https://example.com".to_string()],
        security_notes: "none".to_string(),
        last_verified: "2024-01-01".to_string(),
        path: "agents/test".to_string(),
        harness_variants: HashMap::new(),
        author,
        version,
        execution_tier,
        lifecycle,
        provider_coverage: None,
    }
}

fn option_string() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), "[a-z]{3,10}".prop_map(Some),]
}

fn option_execution_tier() -> impl Strategy<Value = Option<ExecutionTier>> {
    prop_oneof![
        Just(None),
        Just(Some(ExecutionTier::StaticReview)),
        Just(Some(ExecutionTier::ReadOnlyRuntime)),
        Just(Some(ExecutionTier::MutatingRuntime)),
    ]
}

fn option_lifecycle() -> impl Strategy<Value = Option<Lifecycle>> {
    prop_oneof![
        Just(None),
        Just(Some(Lifecycle::Experimental)),
        Just(Some(Lifecycle::Beta)),
        Just(Some(Lifecycle::Stable)),
        Just(Some(Lifecycle::Deprecated)),
    ]
}

fn workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

proptest! {
    // Feature: rust-tui, Property 4: Agent detail formatter includes all required fields
    // **Validates: Requirements 1.4**
    #[test]
    fn agent_detail_contains_all_required_labels(
        id in "[a-z][a-z0-9-]{2,20}",
        name in "[A-Z][a-zA-Z ]{2,20}",
        summary in "[a-zA-Z ]{5,50}",
        version in option_string(),
        author in option_string(),
        execution_tier in option_execution_tier(),
        lifecycle in option_lifecycle(),
    ) {
        let agent = arbitrary_agent(id, name, summary, version.clone(), author.clone(), execution_tier, lifecycle);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::new(false, ThemeMode::Dark);

        terminal.draw(|frame| {
            let area = Rect::new(0, 0, 120, 40);
            render_agent_detail(&agent, &[], &[], area, frame, 0, &theme);
        }).unwrap();

        let buffer = terminal.backend().buffer().clone();
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                text.push_str(cell.symbol());
            }
            text.push('\n');
        }

        // All required labels must appear
        for label in AGENT_DETAIL_REQUIRED_LABELS {
            prop_assert!(
                text.contains(&format!("{label}:")),
                "Missing label '{label}:' in rendered output"
            );
        }

        // Optional fields that are None should show N/A
        if version.is_none() {
            prop_assert!(text.contains("N/A"), "None version should show N/A");
        }
        if author.is_none() {
            prop_assert!(text.contains("N/A"), "None author should show N/A");
        }
    }

    // Property 17: Rendering is deterministic - same input produces same output.
    // **Validates: Requirements 18.1**
    // Tests that for any catalog data, any sequence of user input events, and any
    // fixed terminal dimensions, two executions produce byte-identical output frames.
    #[test]
    fn rendering_is_deterministic(
        id in "[a-z][a-z0-9-]{2,10}",
        name in "[A-Z][a-zA-Z]{2,10}",
        summary in "[a-zA-Z ]{5,30}",
        width in 40u16..200u16,
        height in 15u16..60u16,
        key_events in prop::collection::vec(
            prop_oneof![
                Just(KeyCode::Down),
                Just(KeyCode::Up),
                Just(KeyCode::Char('j')),
                Just(KeyCode::Char('k')),
                Just(KeyCode::Tab),
                Just(KeyCode::Enter),
                Just(KeyCode::Esc),
                Just(KeyCode::Char('g')),
                Just(KeyCode::Char('G')),
            ],
            0..8
        ),
    ) {
        // Build a minimal catalog with generated agent data
        let agent = arbitrary_agent(
            id, name, summary, Some("1.0".to_string()), Some("author".to_string()), None, None
        );

        let workspace_root = workspace_root();
        let fixed_session_id = Uuid::from_u128(0x12345678_1234_1234_1234_123456789abc);

        // Helper: create an App, apply key events, render, and extract buffer text
        let render_with_events = |agents: Vec<Agent>, keys: &[KeyCode]| -> String {
            let mut catalog = CatalogStore {
                agents: agents.clone(),
                skills: Vec::new(),
                roles: HashMap::new(),
                role_catalog_version: "1.0".to_string(),
                role_catalog_description: "test".to_string(),
                mcp_refs: Vec::new(),
                rules: Vec::new(),
                integrity: None,
                model_assignments: None,
                model_registry: None,
            workflows: None,
                load_errors: Vec::new(),
                content_hashes: HashMap::new(),
                catalog_root: workspace_root.clone(),
            };
            // Sort agents like the real loader does (stable case-insensitive by ID)
            catalog.agents.sort_by(|a, b| {
                a.id.to_lowercase().cmp(&b.id.to_lowercase())
            });

            let mut app = App::new(catalog, workspace_root.clone(), fixed_session_id, true);

            // Apply the sequence of key events
            for &code in keys {
                let key = KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                };
                app.handle_key_event(key);
                // Skip if app wants to quit
                if app.should_quit {
                    break;
                }
            }

            // Render to a test backend with the given dimensions
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| {
                app.render(frame);
            }).unwrap();

            // Extract the full buffer content as a string
            let buffer = terminal.backend().buffer().clone();
            let mut text = String::new();
            for y in 0..buffer.area.height {
                for x in 0..buffer.area.width {
                    let cell = &buffer[(x, y)];
                    text.push_str(cell.symbol());
                }
                text.push('\n');
            }
            text
        };

        // Execute the rendering pipeline twice with identical inputs
        let output1 = render_with_events(vec![agent.clone()], &key_events);
        let output2 = render_with_events(vec![agent], &key_events);

        prop_assert_eq!(
            output1, output2,
            "Rendering must be deterministic: same catalog data, same key events, \
             same terminal dimensions must produce identical output"
        );
    }
}
