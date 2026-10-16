use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::{Agent, McpReference, Mod, Rule, Skill, Workflow};

use super::super::theme::Theme;

fn format_option<T: std::fmt::Display>(opt: &Option<T>) -> String {
    match opt {
        Some(s) => s.to_string(),
        None => "N/A".to_string(),
    }
}

fn format_enum<T: serde::Serialize + std::fmt::Debug>(val: &T) -> String {
    serde_json::to_value(val)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{val:?}"))
}

fn format_option_enum<T: serde::Serialize + std::fmt::Debug>(opt: &Option<T>) -> String {
    match opt {
        Some(val) => format_enum(val),
        None => "N/A".to_string(),
    }
}

fn format_vec(v: &[String]) -> String {
    if v.is_empty() {
        "N/A".to_string()
    } else {
        v.join(", ")
    }
}

fn detail_line(key: &str, value: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key}: "), theme.detail_key()),
        Span::styled(value.to_string(), theme.detail_value()),
    ])
}

/// Render agent detail view with all fields.
///
/// `model_lines` carries the resolved per-harness model/reasoning assignments
/// (from catalog/model-assignments.json) as (harness, description) pairs;
/// pass an empty slice when the assignments index is absent.
pub fn render_agent_detail(
    agent: &Agent,
    roles: &[&str],
    model_lines: &[(String, String)],
    area: Rect,
    frame: &mut Frame,
    scroll: u16,
    theme: &Theme,
) {
    let harnesses_str = agent
        .harnesses
        .iter()
        .map(|h| {
            serde_json::to_value(h)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{h:?}"))
        })
        .collect::<Vec<_>>()
        .join(", ");

    let provider_str = serde_json::to_value(agent.provider)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", agent.provider));

    let source_str = serde_json::to_value(agent.source_type)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", agent.source_type));

    let docs_str = format_vec(&agent.official_docs);
    let skills_str = format_vec(&agent.companion_skills);
    let variants_str = if agent.harness_variants.is_empty() {
        "N/A".to_string()
    } else {
        agent
            .harness_variants
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let roles_str = if roles.is_empty() {
        "N/A".to_string()
    } else {
        roles.join(", ")
    };

    let lines = vec![
        detail_line("ID", &agent.id, theme),
        detail_line("Name", &agent.name, theme),
        detail_line("Type", &format_enum(&agent.entity_type), theme),
        detail_line("Provider", &provider_str, theme),
        detail_line("Harnesses", &harnesses_str, theme),
        detail_line("Summary", &agent.summary, theme),
        detail_line("Source Type", &source_str, theme),
        detail_line("Official Docs", &docs_str, theme),
        detail_line("Security Notes", &agent.security_notes, theme),
        detail_line("Last Verified", &agent.last_verified, theme),
        detail_line("Path", &agent.path, theme),
        detail_line("Companion Skills", &skills_str, theme),
        detail_line("Version", &format_option(&agent.version), theme),
        detail_line("Author", &format_option(&agent.author), theme),
        detail_line(
            "Execution Tier",
            &format_option_enum(&agent.execution_tier),
            theme,
        ),
        detail_line("Lifecycle", &format_option_enum(&agent.lifecycle), theme),
        detail_line("Harness Variants", &variants_str, theme),
        detail_line("Roles", &roles_str, theme),
    ];

    let mut lines = lines;
    if model_lines.is_empty() {
        lines.push(detail_line("Models", "auto (harness defaults)", theme));
    } else {
        lines.push(detail_line("Models", "", theme));
        for (harness, description) in model_lines {
            if harness == "warning" {
                // Provider-lifecycle warning row (see
                // App::build_model_lines / scripts/model-policy.mjs
                // resolveLifecycle): styled entirely in the theme's
                // warning colour so it reads distinctly from the
                // harness/model row above it.
                lines.push(Line::from(Span::styled(
                    format!("    {description}"),
                    theme.detail_key(),
                )));
            } else {
                lines.push(detail_line(&format!("  {harness}"), description, theme));
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(agent.name.clone())
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Render skill detail view with related agents.
pub fn render_skill_detail(
    skill: &Skill,
    related_agents: &[&Agent],
    area: Rect,
    frame: &mut Frame,
    scroll: u16,
    theme: &Theme,
) {
    let harnesses_str = skill
        .harnesses
        .iter()
        .map(|h| {
            serde_json::to_value(h)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{h:?}"))
        })
        .collect::<Vec<_>>()
        .join(", ");

    let provider_str = serde_json::to_value(skill.provider)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", skill.provider));

    let source_str = serde_json::to_value(skill.source_type)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", skill.source_type));

    let docs_str = format_vec(&skill.official_docs);

    let related_str = if related_agents.is_empty() {
        "N/A".to_string()
    } else {
        related_agents
            .iter()
            .map(|a| a.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let lines = vec![
        detail_line("ID", &skill.id, theme),
        detail_line("Name", &skill.name, theme),
        detail_line("Type", &format_enum(&skill.entity_type), theme),
        detail_line("Provider", &provider_str, theme),
        detail_line("Harnesses", &harnesses_str, theme),
        detail_line("Summary", &skill.summary, theme),
        detail_line("Source Type", &source_str, theme),
        detail_line("Official Docs", &docs_str, theme),
        detail_line("Security Notes", &skill.security_notes, theme),
        detail_line("Last Verified", &skill.last_verified, theme),
        detail_line("Path", &skill.path, theme),
        detail_line("Author", &format_option(&skill.author), theme),
        detail_line("Version", &format_option(&skill.version), theme),
        detail_line("Category", &format_option(&skill.category), theme),
        detail_line(
            "Execution Tier",
            &format_option_enum(&skill.execution_tier),
            theme,
        ),
        detail_line("Lifecycle", &format_option_enum(&skill.lifecycle), theme),
        detail_line("Related Agents", &related_str, theme),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(skill.name.clone())
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Render MCP reference detail view.
pub fn render_mcp_detail(
    mcp: &McpReference,
    area: Rect,
    frame: &mut Frame,
    scroll: u16,
    theme: &Theme,
) {
    let harnesses_str = mcp
        .harnesses
        .iter()
        .map(|h| {
            serde_json::to_value(h)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{h:?}"))
        })
        .collect::<Vec<_>>()
        .join(", ");

    let provider_str = serde_json::to_value(mcp.provider)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", mcp.provider));

    let docs_str = format_vec(&mcp.official_docs);

    let trust_str = mcp
        .trust_matrix
        .as_ref()
        .map(|tm| {
            format!(
                "mutation={}, egress={}, creds={}, signed={}, pin={}",
                tm.mutation_capable,
                tm.requires_egress,
                tm.requires_credentials,
                tm.signed_release,
                tm.pin_strategy
            )
        })
        .unwrap_or_else(|| "N/A".to_string());

    let lines = vec![
        detail_line("ID", &mcp.id, theme),
        detail_line("Name", &mcp.name, theme),
        detail_line("Type", &format_enum(&mcp.entity_type), theme),
        detail_line("Provider", &provider_str, theme),
        detail_line("Harnesses", &harnesses_str, theme),
        detail_line("Summary", &mcp.summary, theme),
        detail_line("Vendor", &mcp.vendor, theme),
        detail_line("Auth Model", &mcp.auth_model, theme),
        detail_line("Project URL", &mcp.official_project_url, theme),
        detail_line("Install Example", &mcp.install_example, theme),
        detail_line("Unofficial Warning", &mcp.unofficial_warning, theme),
        detail_line("Official Docs", &docs_str, theme),
        detail_line("Security Notes", &mcp.security_notes, theme),
        detail_line("Last Verified", &mcp.last_verified, theme),
        detail_line("Path", &mcp.path, theme),
        detail_line("Trust Matrix", &trust_str, theme),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(mcp.name.clone())
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Render rule detail view.
pub fn render_rule_detail(rule: &Rule, area: Rect, frame: &mut Frame, scroll: u16, theme: &Theme) {
    let harnesses_str = rule
        .harnesses
        .iter()
        .map(|h| {
            serde_json::to_value(h)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{h:?}"))
        })
        .collect::<Vec<_>>()
        .join(", ");

    let provider_str = serde_json::to_value(rule.provider)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", rule.provider));

    let source_str = serde_json::to_value(rule.source_type)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{:?}", rule.source_type));

    let docs_str = format_vec(&rule.official_docs);

    let lines = vec![
        detail_line("ID", &rule.id, theme),
        detail_line("Name", &rule.name, theme),
        detail_line("Type", &format_enum(&rule.entity_type), theme),
        detail_line("Provider", &provider_str, theme),
        detail_line("Harnesses", &harnesses_str, theme),
        detail_line("Summary", &rule.summary, theme),
        detail_line("Source Type", &source_str, theme),
        detail_line("Official Docs", &docs_str, theme),
        detail_line("Security Notes", &rule.security_notes, theme),
        detail_line("Last Verified", &rule.last_verified, theme),
        detail_line("Path", &rule.path, theme),
        detail_line("Author", &format_option(&rule.author), theme),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(rule.name.clone())
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Render a workflow's detail pane: what it is, when to reach for it, and the phase
/// list with the model tier each phase actually runs on.
pub fn render_workflow_detail(
    wf: &Workflow,
    area: Rect,
    frame: &mut Frame,
    scroll: u16,
    theme: &Theme,
) {
    let phases_str = if wf.phases.is_empty() {
        "N/A".to_string()
    } else {
        wf.phases
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}. {} [{}]", i + 1, p.title, p.model_label()))
            .collect::<Vec<_>>()
            .join("  ")
    };

    let lines = vec![
        detail_line("ID", &wf.id, theme),
        detail_line("Invoke", &wf.invocation(), theme),
        detail_line("Description", &wf.description, theme),
        detail_line("When To Use", &wf.when_to_use, theme),
        detail_line("Phases", &format!("{}", wf.phase_count()), theme),
        detail_line("Sequence", &phases_str, theme),
        detail_line("Model Tiers", &wf.model_tiers().join(", "), theme),
        detail_line("Path", &wf.path, theme),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(wf.name.clone())
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Labels that MUST appear in an agent detail rendering.
pub const AGENT_DETAIL_REQUIRED_LABELS: &[&str] = &[
    "ID",
    "Name",
    "Type",
    "Provider",
    "Harnesses",
    "Summary",
    "Source Type",
    "Official Docs",
    "Security Notes",
    "Last Verified",
    "Path",
    "Companion Skills",
    "Version",
    "Author",
    "Execution Tier",
    "Lifecycle",
    "Harness Variants",
    "Roles",
];

/// Display catalog disclosure only; never inspect paths or invoke a mod.
pub fn render_mod_detail(entry: &Mod, area: Rect, frame: &mut Frame, scroll: u16, theme: &Theme) {
    let mut lines = vec![
        detail_line("ID", &entry.id, theme),
        detail_line("Name", &entry.name, theme),
        detail_line("Version", &entry.version, theme),
        detail_line("Description", &entry.description, theme),
        detail_line("Reach Level", &format!("L{}", entry.reach_level), theme),
        detail_line("Events", &entry.events.join(", "), theme),
        detail_line("Harnesses", &entry.harnesses.join(", "), theme),
        detail_line("Path", &entry.path, theme),
        detail_line(
            "Security Notes",
            &format_option(&entry.security_notes),
            theme,
        ),
    ];
    if let Some(footprint) = &entry.footprint {
        lines.push(detail_line("Declared footprint", "", theme));
        for (label, values) in [
            ("env_reads", &footprint.env_reads),
            ("file_writes", &footprint.file_writes),
            ("spawns", &footprint.spawns),
            ("egress_hosts", &footprint.egress_hosts),
        ] {
            let value = if values.is_empty() {
                "none declared".to_string()
            } else {
                values.join(", ")
            };
            lines.push(detail_line(label, &value, theme));
        }
    } else {
        lines.push(detail_line("Declared footprint", "not recorded", theme));
    }
    lines.push(Line::from("Disclosure only; not enforced containment."));
    lines.push(Line::from(
        "Declared metadata is not live harness verification.",
    ));
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} — Esc: Back", entry.name))
                .border_style(theme.border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}
