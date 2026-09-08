use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use uuid::Uuid;

use crate::catalog::store::CatalogStore;
use crate::models::export::{ExportCommand, ExportSelection};
use crate::models::gate::{extract_validation_gates, GateStatus, ValidationGate};
use crate::models::model_policy::{ModelPolicyCommand, ModelScope, CAPABLE_HARNESSES};
use crate::models::model_registry::{ModelChoice, ModelRegistry};
use crate::search::fuzzy::SearchEngine;
use crate::security::sanitize::sanitize_subprocess_output;
use crate::subprocess::{SubprocessExecutor, SubprocessHandle};

use crate::ui::layout::compute_layout;
use crate::ui::nav::{NavigationState, View, SIDEBAR_SECTIONS};
use crate::ui::theme::{Theme, ThemeMode};
use crate::ui::widgets::{
    audit_log, coverage_grid, dep_graph, detail, help_bar, list_view, output, search, status_bar,
    violations,
};

const MAX_SUBPROCESS_OUTPUT_LINES: usize = 10_000;
const MAX_SEARCH_QUERY_LEN: usize = 256;
/// Default timeout for validation gate subprocesses (300 seconds per requirement 6.5).
const VALIDATION_GATE_TIMEOUT_SECS: u64 = 300;
/// Label for the "Run All Validations" meta-entry in the validation list.
const RUN_ALL_LABEL: &str = "validate (Run All)";

/// State for the export command builder UI.
pub struct ExportBuilderState {
    pub platform: String,
    pub selection: ExportSelection,
    pub target_repo: String,
    pub dry_run: bool,
    pub force: bool,
    pub no_skills: bool,
    pub focused_field: usize,
}

impl ExportBuilderState {
    pub fn new() -> Self {
        Self {
            platform: "kiro".to_string(),
            selection: ExportSelection::All,
            target_repo: String::new(),
            dry_run: true,
            force: false,
            no_skills: false,
            focused_field: 0,
        }
    }
}

impl Default for ExportBuilderState {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback reasoning vocabulary for the model policy builder, used only when
/// `catalog/model-registry.json` is missing or unparsable. The live cycle is
/// built from the registry per (harness, model) by
/// [`ModelPolicyBuilderState::refresh_choices`], so the builder offers exactly
/// what `scripts/model-policy.mjs` will accept instead of a static union that
/// drifts. This list is the pre-registry behaviour, preserved verbatim so a
/// checkout without the registry degrades rather than breaks.
const MODEL_POLICY_REASONING_FALLBACK: &[&str] =
    &["none", "minimal", "low", "medium", "high", "xhigh", "max"];

/// First entry of every reasoning cycle: leave the field untouched (no
/// `--reasoning` flag is passed at all).
const MODEL_POLICY_REASONING_UNCHANGED: &str = "(unchanged)";

/// Stage of the model-policy publish pipeline. After a successful non-dry-run
/// apply the TUI automatically chains `npm run asset-integrity:write` so the
/// integrity manifest can never silently drift from the projected files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelPolicyStage {
    /// `node scripts/model-policy.mjs set …` is running.
    Apply,
    /// `npm run asset-integrity:write` chained after a successful apply.
    IntegrityRefresh,
}

/// State for the model policy builder UI.
pub struct ModelPolicyBuilderState {
    pub scope: ModelScope,
    /// Index into [`CAPABLE_HARNESSES`].
    pub harness_index: usize,
    /// Model name or "auto"; empty leaves the model field of the rule untouched.
    /// Free text is still accepted — open registry namespaces (Ollama tags,
    /// OpenRouter slugs) are too large to enumerate, so the picker offers the
    /// verified and example values and typing covers everything else.
    pub model: String,
    /// Registry-verified models selectable for the current harness, refreshed
    /// by [`Self::refresh_choices`]. Empty when no registry is loaded.
    pub model_choices: Vec<ModelChoice>,
    /// Position within `model_choices`; `None` means the model field holds a
    /// free-typed value (or is empty) rather than a picked entry.
    pub model_choice_index: Option<usize>,
    /// Index into `reasoning_cycle`.
    pub reasoning_index: usize,
    /// Live reasoning cycle for the current (harness, model) pair. Index 0 is
    /// always [`MODEL_POLICY_REASONING_UNCHANGED`]; when the pair supports an
    /// effort at all, "auto" and the verified values follow. A single-entry
    /// cycle therefore means "this pair has no projectable reasoning field".
    pub reasoning_cycle: Vec<String>,
    pub dry_run: bool,
    /// Chain `npm run asset-integrity:write` after a successful apply.
    pub refresh_integrity: bool,
    pub focused_field: usize,
}

impl ModelPolicyBuilderState {
    pub fn new() -> Self {
        Self {
            scope: ModelScope::All,
            harness_index: 0,
            model: String::new(),
            model_choices: Vec::new(),
            model_choice_index: None,
            reasoning_index: 0,
            reasoning_cycle: vec![MODEL_POLICY_REASONING_UNCHANGED.to_string()],
            dry_run: true,
            refresh_integrity: true,
            focused_field: 0,
        }
    }

    /// The harness id currently selected.
    pub fn harness(&self) -> &'static str {
        CAPABLE_HARNESSES[self.harness_index % CAPABLE_HARNESSES.len()]
    }

    /// Recompute both pickers from the registry for the current harness and
    /// model. Called whenever the harness or the model text changes.
    ///
    /// The narrowing is the registry's, not ours: a model that declares its own
    /// `reasoning_efforts` gets exactly those, otherwise its namespace's,
    /// otherwise the harness vocabulary. A free-typed model the registry does
    /// not enumerate keeps the full harness vocabulary and is left for
    /// `scripts/model-policy.mjs` to accept or reject — the TUI never decides
    /// legality itself.
    ///
    /// `registry: None` (missing or unparsable catalog file) restores the
    /// pre-registry behaviour: free-text model entry and the static union,
    /// codex only.
    pub fn refresh_choices(&mut self, registry: Option<&ModelRegistry>) {
        let harness = self.harness();

        // Remember whether the box currently holds a *picked* entry: if the
        // new harness does not offer it, carrying the text over would hand the
        // script a model it is certain to reject (pick gpt-6-astra on codex,
        // cycle to claude-code). A free-typed value is never dropped — the
        // caller clears `model_choice_index` before an edit for that reason.
        let was_picked = self.model_choice_index.is_some();

        self.model_choices = registry
            .map(|r| r.choices_for_harness(harness))
            .unwrap_or_default();
        let matched = self
            .model_choices
            .iter()
            .position(|c| c.model == self.model)
            .filter(|_| !self.model.is_empty());
        if was_picked && matched.is_none() {
            self.model.clear();
        }
        self.model_choice_index = matched;

        let harness_has_reasoning = match registry {
            Some(r) => r.supports_reasoning(harness),
            None => harness == "codex",
        };
        let efforts: Vec<String> = match registry {
            _ if !harness_has_reasoning => Vec::new(),
            Some(r) if self.model.is_empty() || self.model == "auto" => r.harness_efforts(harness),
            Some(r) => r.efforts_for(harness, &self.model),
            None => MODEL_POLICY_REASONING_FALLBACK
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        };

        let previous = self
            .reasoning_cycle
            .get(self.reasoning_index)
            .cloned()
            .unwrap_or_else(|| MODEL_POLICY_REASONING_UNCHANGED.to_string());

        let mut cycle = vec![MODEL_POLICY_REASONING_UNCHANGED.to_string()];
        // "auto" is offered whenever the *harness* has a reasoning field, even
        // when the selected model supports no effort value. It clears the
        // managed field, and that is exactly what an operator needs when
        // moving a rule onto such a model: the engine rejects the inherited
        // effort with "set reasoning to auto", so collapsing the cycle to
        // "(unchanged)" would make its own remediation unreachable.
        if harness_has_reasoning {
            cycle.push("auto".to_string());
            cycle.extend(efforts);
        }
        // Keep the operator's selection across a refresh when the new cycle
        // still offers it; otherwise fall back to "leave it alone" rather than
        // silently landing on a neighbouring effort.
        self.reasoning_index = cycle.iter().position(|e| *e == previous).unwrap_or(0);
        self.reasoning_cycle = cycle;
    }

    /// Whether the reasoning field can be cycled at all — true when the
    /// harness has a reasoning field, even if the selected model accepts no
    /// concrete effort (`auto` is still reachable to clear it).
    pub fn reasoning_supported(&self) -> bool {
        self.reasoning_cycle.len() > 1
    }

    /// True when the harness has a reasoning field but the selected model
    /// accepts no concrete effort — the cycle holds only "(unchanged)" and
    /// "auto".
    pub fn reasoning_clear_only(&self) -> bool {
        self.reasoning_cycle.len() == 2
    }

    /// The reasoning value currently shown for the focused field.
    pub fn reasoning_display(&self) -> &str {
        self.reasoning_cycle
            .get(self.reasoning_index)
            .map(String::as_str)
            .unwrap_or(MODEL_POLICY_REASONING_UNCHANGED)
    }

    /// Step to the next verified model for this harness, wrapping through a
    /// free-text slot so the picker never traps the operator inside the list.
    pub fn cycle_model(&mut self) {
        if self.model_choices.is_empty() {
            return;
        }
        match self.model_choice_index {
            Some(i) if i + 1 >= self.model_choices.len() => {
                self.model_choice_index = None;
                self.model.clear();
            }
            Some(i) => {
                self.model_choice_index = Some(i + 1);
                self.model = self.model_choices[i + 1].model.clone();
            }
            None => {
                self.model_choice_index = Some(0);
                self.model = self.model_choices[0].model.clone();
            }
        }
    }

    /// The picked registry entry, when the model field holds one.
    pub fn selected_choice(&self) -> Option<&ModelChoice> {
        self.model_choice_index
            .and_then(|i| self.model_choices.get(i))
    }

    /// Build the subprocess command from the current form state.
    pub fn command(&self) -> ModelPolicyCommand {
        ModelPolicyCommand {
            scope: self.scope.clone(),
            harness: self.harness().to_string(),
            model: if self.model.is_empty() {
                None
            } else {
                Some(self.model.clone())
            },
            reasoning: if self.reasoning_index == 0 {
                None
            } else {
                self.reasoning_cycle.get(self.reasoning_index).cloned()
            },
            dry_run: self.dry_run,
        }
    }
}

impl Default for ModelPolicyBuilderState {
    fn default() -> Self {
        Self::new()
    }
}

/// Main application state.
pub struct App {
    pub nav: NavigationState,
    pub catalog: CatalogStore,
    pub search_engine: SearchEngine,
    pub search_query: String,
    pub search_active: bool,
    pub filtered_indices: Vec<usize>,
    pub subprocess_output: Vec<output::OutputLine>,
    pub subprocess_handle: Option<SubprocessHandle>,
    pub validation_gates: Vec<ValidationGate>,
    pub export_state: ExportBuilderState,
    pub model_policy_state: ModelPolicyBuilderState,
    /// Set while a model-policy subprocess (apply or chained integrity
    /// refresh) is running; drives completion handling in `tick()`.
    pub model_policy_stage: Option<ModelPolicyStage>,
    pub status_message: Option<(String, Instant)>,
    pub session_id: Uuid,
    pub should_quit: bool,
    pub no_color: bool,
    /// Resolved theme mode (Dark/Light) for the session; toggled at runtime
    /// via the `t` keybinding (Req 35.6). Defaults to Dark; `main` overrides
    /// it from the `--theme` flag / system detection.
    pub theme_mode: ThemeMode,
    pub workspace_root: PathBuf,
    /// Name of the currently running validation gate (for status tracking).
    /// When set, prevents concurrent execution of the same gate.
    pub running_gate: Option<String>,
    /// Timestamp when the currently running gate started (for animated indicator).
    pub running_gate_start: Option<Instant>,
    /// Pending subprocess spawn result — polled in `tick()` to avoid `block_on` inside tokio runtime.
    pub pending_subprocess:
        Option<tokio::sync::oneshot::Receiver<anyhow::Result<SubprocessHandle>>>,
    // v0.2.0 enhancements
    pub provider_filter: Option<String>,
    pub harness_filter: Option<String>,
    pub show_help_overlay: bool,
    pub completion_suggestions: Vec<String>,
    pub completion_index: usize,
    /// Dirty flag for async event loop — render only when true (250ms tick resets).
    pub dirty: bool,
}

impl App {
    pub fn new(
        catalog: CatalogStore,
        workspace_root: PathBuf,
        session_id: Uuid,
        no_color: bool,
    ) -> Self {
        let validation_gates = extract_validation_gates(&workspace_root);
        let mut search_engine = SearchEngine::new();
        let filtered_indices = search_engine.search_agents("", &catalog.agents, None, None);
        Self {
            nav: NavigationState::new(),
            catalog,
            search_engine,
            search_query: String::new(),
            search_active: false,
            filtered_indices,
            subprocess_output: Vec::new(),
            subprocess_handle: None,
            validation_gates,
            export_state: ExportBuilderState::new(),
            model_policy_state: ModelPolicyBuilderState::new(),
            model_policy_stage: None,
            status_message: None,
            session_id,
            should_quit: false,
            no_color,
            theme_mode: ThemeMode::Dark,
            workspace_root,
            running_gate: None,
            running_gate_start: None,
            pending_subprocess: None,
            provider_filter: None,
            harness_filter: None,
            show_help_overlay: false,
            completion_suggestions: Vec::new(),
            completion_index: 0,
            dirty: true,
        }
    }

    /// Handle a key event. Dispatches based on search mode and current view.
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Help overlay takes priority
        if self.show_help_overlay {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.show_help_overlay = false;
                }
                _ => {}
            }
            return;
        }

        if self.search_active {
            self.handle_search_key(key);
            return;
        }

        // Export views have their own key handling
        match &self.nav.current_view {
            View::ExportBuilder => {
                self.handle_export_builder_key(key);
                return;
            }
            View::ExportConfirm => {
                self.handle_export_confirm_key(key);
                return;
            }
            View::ExportOutput => {
                self.handle_export_output_key(key);
                return;
            }
            View::ModelPolicyBuilder => {
                self.handle_model_policy_builder_key(key);
                return;
            }
            View::ModelPolicyConfirm => {
                self.handle_model_policy_confirm_key(key);
                return;
            }
            View::ModelPolicyOutput => {
                self.handle_model_policy_output_key(key);
                return;
            }
            _ => {}
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.show_help_overlay = true;
            }
            KeyCode::Char('t') => {
                // Runtime light/dark toggle (Req 35.6). Search mode is handled
                // earlier and returns before reaching this match, so `t` is only
                // a toggle outside of search.
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                self.dirty = true;
            }
            KeyCode::Char('p') if self.nav.current_view == View::AgentList => {
                self.cycle_provider_filter();
            }
            KeyCode::Char('h') if self.nav.current_view == View::AgentList => {
                self.cycle_harness_filter();
            }
            KeyCode::Char('m') => {
                self.open_model_policy_builder();
            }
            KeyCode::Esc => {
                // Clear filters first, then pop view
                if self.provider_filter.is_some() || self.harness_filter.is_some() {
                    self.provider_filter = None;
                    self.harness_filter = None;
                    self.update_filtered();
                } else if !self.nav.pop_view() {
                    self.should_quit = true;
                }
                self.update_filtered();
            }
            KeyCode::Tab => {
                self.nav.next_tab();
                self.sync_view_to_tab();
                self.dirty = true;
            }
            KeyCode::BackTab => {
                self.nav.prev_tab();
                self.sync_view_to_tab();
                self.dirty = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.handle_down(),
            KeyCode::Char('k') | KeyCode::Up => self.handle_up(),
            KeyCode::Char('g') => self.nav.select_first(),
            KeyCode::Char('G') => {
                let max = self.current_list_len();
                self.nav.select_last(max);
            }
            KeyCode::Char('/') => {
                // Only activate search on views where filtering works (agent list)
                if self.nav.current_view == View::AgentList {
                    self.search_active = true;
                    self.search_query.clear();
                }
            }
            KeyCode::Enter => self.handle_enter(),
            _ => {}
        }
    }

    /// Handle key events during active search mode.
    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_active = false;
            }
            KeyCode::Enter => {
                self.search_active = false;
                self.update_filtered();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_filtered();
            }
            KeyCode::Char(c) if self.search_query.len() < MAX_SEARCH_QUERY_LEN => {
                self.search_query.push(c);
                self.update_filtered();
            }
            _ => {}
        }
    }

    /// Number of fields in the export builder form.
    const EXPORT_FIELD_COUNT: usize = 6;

    /// Handle key events in the ExportBuilder view.
    /// Supports j/k for field navigation, Enter to edit/toggle/confirm,
    /// Tab to accept completion, and Esc to go back.
    fn handle_export_builder_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                if !self.nav.pop_view() {
                    self.should_quit = true;
                }
            }
            KeyCode::Tab => {
                // Accept completion suggestion if available
                if !self.completion_suggestions.is_empty() {
                    let suggestion = self.completion_suggestions[self.completion_index].clone();
                    match self.export_state.focused_field {
                        0 => self.export_state.platform = suggestion,
                        1 => {
                            self.export_state.selection = ExportSelection::Role(suggestion);
                        }
                        _ => {}
                    }
                    self.completion_suggestions.clear();
                    self.completion_index = 0;
                } else {
                    // Switch to next sidebar section
                    let next = (self.nav.sidebar_index + 1) % SIDEBAR_SECTIONS.len();
                    self.nav.set_sidebar_index(next);
                    self.search_query.clear();
                    self.provider_filter = None;
                    self.harness_filter = None;
                    self.update_filtered();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Navigate to next field, or next completion suggestion
                if !self.completion_suggestions.is_empty() {
                    self.completion_index =
                        (self.completion_index + 1) % self.completion_suggestions.len();
                } else {
                    let next = self.export_state.focused_field + 1;
                    if next < Self::EXPORT_FIELD_COUNT {
                        self.export_state.focused_field = next;
                    }
                    // Clear completions when navigating fields
                    self.completion_suggestions.clear();
                    self.completion_index = 0;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Navigate to previous field, or previous completion suggestion
                if !self.completion_suggestions.is_empty() {
                    self.completion_index = if self.completion_index == 0 {
                        self.completion_suggestions.len() - 1
                    } else {
                        self.completion_index - 1
                    };
                } else {
                    if self.export_state.focused_field > 0 {
                        self.export_state.focused_field -= 1;
                    }
                    // Clear completions when navigating fields
                    self.completion_suggestions.clear();
                    self.completion_index = 0;
                }
            }
            KeyCode::Enter => {
                self.handle_export_builder_enter();
            }
            KeyCode::Char(' ') => {
                // Toggle boolean fields with space
                match self.export_state.focused_field {
                    3 => self.export_state.dry_run = !self.export_state.dry_run,
                    4 => self.export_state.force = !self.export_state.force,
                    5 => self.export_state.no_skills = !self.export_state.no_skills,
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                // Edit text fields with backspace
                match self.export_state.focused_field {
                    0 => {
                        self.export_state.platform.pop();
                        self.update_completion_suggestions();
                    }
                    2 => {
                        self.export_state.target_repo.pop();
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                // Type into text fields
                match self.export_state.focused_field {
                    0 => {
                        self.export_state.platform.push(c);
                        self.update_completion_suggestions();
                    }
                    2 => {
                        self.export_state.target_repo.push(c);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Handle Enter key in the export builder — either cycle selection or confirm.
    fn handle_export_builder_enter(&mut self) {
        match self.export_state.focused_field {
            1 => {
                // Cycle through selection methods: All -> Role -> Provider -> Agents -> All
                self.export_state.selection = match &self.export_state.selection {
                    ExportSelection::All => {
                        let roles = self.catalog.role_ids();
                        if let Some(first) = roles.first() {
                            ExportSelection::Role(first.clone())
                        } else {
                            ExportSelection::Provider("aws".to_string())
                        }
                    }
                    ExportSelection::Role(current) => {
                        let roles = self.catalog.role_ids();
                        let idx = roles.iter().position(|r| r == current);
                        match idx {
                            Some(i) if i + 1 < roles.len() => {
                                ExportSelection::Role(roles[i + 1].clone())
                            }
                            _ => {
                                let providers = self.catalog.provider_names();
                                if let Some(first) = providers.first() {
                                    ExportSelection::Provider(first.clone())
                                } else {
                                    ExportSelection::All
                                }
                            }
                        }
                    }
                    ExportSelection::Provider(current) => {
                        let providers = self.catalog.provider_names();
                        let idx = providers.iter().position(|p| p == current);
                        match idx {
                            Some(i) if i + 1 < providers.len() => {
                                ExportSelection::Provider(providers[i + 1].clone())
                            }
                            _ => ExportSelection::All,
                        }
                    }
                    ExportSelection::Agents(_) => ExportSelection::All,
                };
            }
            3 => self.export_state.dry_run = !self.export_state.dry_run,
            4 => self.export_state.force = !self.export_state.force,
            5 => self.export_state.no_skills = !self.export_state.no_skills,
            _ => {
                // On any other field, if we have enough info, go to confirm
                if !self.export_state.platform.is_empty()
                    && !self.export_state.target_repo.is_empty()
                {
                    // Validate target path exists and is writable
                    let target = PathBuf::from(&self.export_state.target_repo);
                    if !target.exists() {
                        self.status_message = Some((
                            format!(
                                "Error: target path does not exist: {}",
                                self.export_state.target_repo
                            ),
                            Instant::now(),
                        ));
                        return;
                    }
                    if !target.is_dir() {
                        self.status_message = Some((
                            format!(
                                "Error: target path is not a directory: {}",
                                self.export_state.target_repo
                            ),
                            Instant::now(),
                        ));
                        return;
                    }
                    // Check writable by attempting to access metadata
                    if target.metadata().is_err() {
                        self.status_message = Some((
                            format!(
                                "Error: target path is not accessible: {}",
                                self.export_state.target_repo
                            ),
                            Instant::now(),
                        ));
                        return;
                    }

                    // Build the command and validate arguments
                    let cmd = ExportCommand {
                        platform: self.export_state.platform.clone(),
                        selection: self.export_state.selection.clone(),
                        target_repo: target,
                        dry_run: self.export_state.dry_run,
                        force: self.export_state.force,
                        no_skills: self.export_state.no_skills,
                    };

                    if let Err(e) = cmd.validate() {
                        self.status_message =
                            Some((format!("Validation error: {e}"), Instant::now()));
                        return;
                    }

                    self.nav.push_view(View::ExportConfirm);
                } else if self.export_state.platform.is_empty() {
                    self.status_message =
                        Some(("Platform is required".to_string(), Instant::now()));
                } else {
                    self.status_message =
                        Some(("Target repo path is required".to_string(), Instant::now()));
                }
            }
        }
    }

    /// Handle key events in the ExportConfirm view.
    /// Enter executes the export, Esc cancels back to builder.
    fn handle_export_confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                // Cancel back to builder, preserving selections
                self.nav.pop_view();
            }
            KeyCode::Enter => {
                // Execute the export command
                self.execute_export();
            }
            _ => {}
        }
    }

    /// Handle key events in the ExportOutput view.
    /// Esc cancels/goes back, Ctrl+C during execution cancels subprocess.
    fn handle_export_output_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') if self.subprocess_handle.is_none() => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.subprocess_handle.is_some() {
                    // Cancel the running subprocess
                    self.cancel_export_subprocess();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Esc => {
                if self.subprocess_handle.is_some() {
                    // Cancel the running subprocess and go back to builder
                    self.cancel_export_subprocess();
                }
                // Go back to builder, preserving selections
                self.nav.pop_view();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.nav.detail_scroll = self.nav.detail_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.nav.detail_scroll = self.nav.detail_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Execute the export command as a subprocess.
    fn execute_export(&mut self) {
        // Defense in depth: never launch over a live gate/subprocess — this
        // would otherwise silently overwrite `pending_subprocess`/
        // `subprocess_handle` out from under a model-policy stage or a
        // running validation gate, losing track of that operation's exit
        // status (or clobbering its handle so it can no longer be polled or
        // cancelled). Mirrors execute_model_policy's own guard.
        if self.subprocess_busy() {
            self.status_message = Some((
                "A subprocess is already running; wait for it to finish".to_string(),
                Instant::now(),
            ));
            return;
        }
        let target = PathBuf::from(&self.export_state.target_repo);
        let cmd = ExportCommand {
            platform: self.export_state.platform.clone(),
            selection: self.export_state.selection.clone(),
            target_repo: target,
            dry_run: self.export_state.dry_run,
            force: self.export_state.force,
            no_skills: self.export_state.no_skills,
        };

        // Clear previous output
        self.subprocess_output.clear();

        let args = cmd.to_args();
        let workspace = self.workspace_root.clone();

        // Spawn the subprocess asynchronously via oneshot channel to avoid
        // `block_on` panic inside an existing tokio runtime.
        let (tx, rx) = tokio::sync::oneshot::channel();
        let all_args = [
            vec!["scripts/export-marketplace-agents.mjs".to_string()],
            args,
        ]
        .concat();
        tokio::spawn(async move {
            let result =
                SubprocessExecutor::spawn("node", &all_args, &workspace, Duration::from_secs(300))
                    .await;
            let _ = tx.send(result);
        });
        // Store receiver to poll in tick()
        self.pending_subprocess = Some(rx);
        self.nav.push_view(View::ExportOutput);
    }

    /// Cancel the export subprocess.
    fn cancel_export_subprocess(&mut self) {
        if let Some(mut handle) = self.subprocess_handle.take() {
            tokio::spawn(async move {
                handle.cancel().await.ok();
            });
            self.status_message = Some(("Export cancelled by user".to_string(), Instant::now()));
        }
    }

    /// Number of fields in the model policy builder form (the last entry is
    /// the `[ Continue ]` action row).
    const MODEL_POLICY_FIELD_COUNT: usize = 7;

    /// True when a subprocess (validation gate, export, or a model-policy
    /// apply/integrity-refresh) is spawning or already running. Only one
    /// subprocess may own `pending_subprocess`/`subprocess_handle` at a time,
    /// so new launches must be refused while any of these is set — otherwise a
    /// second spawn overwrites the first handle and its exit code is recorded
    /// against the wrong operation (e.g. a model-policy run clobbering a
    /// validation gate's result).
    fn subprocess_busy(&self) -> bool {
        self.running_gate.is_some()
            || self.subprocess_handle.is_some()
            || self.pending_subprocess.is_some()
            || self.model_policy_stage.is_some()
    }

    /// Open the model policy builder, pre-filling the scope from the current
    /// view (agent detail → that agent, provider/role views → that provider
    /// or role, otherwise all agents).
    fn open_model_policy_builder(&mut self) {
        if self.subprocess_busy() {
            self.status_message = Some((
                "Cannot open the model policy builder while another task is running".to_string(),
                Instant::now(),
            ));
            return;
        }
        let idx = self.nav.selected_index();
        let scope = match self.nav.current_view.clone() {
            View::AgentDetail(id) => ModelScope::Agent(id),
            View::AgentList => self
                .filtered_indices
                .get(idx)
                .and_then(|&real_idx| self.catalog.agents.get(real_idx))
                .map(|a| ModelScope::Agent(a.id.clone()))
                .unwrap_or(ModelScope::All),
            View::ProviderAgents(provider) => ModelScope::Provider(provider),
            View::ProviderList => self
                .get_provider_list()
                .get(idx)
                .map(|(p, _)| ModelScope::Provider(p.clone()))
                .unwrap_or(ModelScope::All),
            View::RoleDetail(role) => ModelScope::Role(role),
            View::RoleList => self
                .get_role_list()
                .get(idx)
                .map(|r| ModelScope::Role(r.0.clone()))
                .unwrap_or(ModelScope::All),
            _ => ModelScope::All,
        };
        self.model_policy_state = ModelPolicyBuilderState::new();
        self.model_policy_state.scope = scope;
        self.refresh_model_policy_choices();
        self.nav.push_view(View::ModelPolicyBuilder);
    }

    /// Handle key events in the ModelPolicyBuilder view.
    fn handle_model_policy_builder_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                if !self.nav.pop_view() {
                    self.should_quit = true;
                }
            }
            KeyCode::Down => {
                let next = self.model_policy_state.focused_field + 1;
                if next < Self::MODEL_POLICY_FIELD_COUNT {
                    self.model_policy_state.focused_field = next;
                }
            }
            KeyCode::Up => {
                if self.model_policy_state.focused_field > 0 {
                    self.model_policy_state.focused_field -= 1;
                }
            }
            // j/k navigate fields unless a text field is focused (then they type).
            KeyCode::Char('j') if !self.model_policy_text_field_focused() => {
                let next = self.model_policy_state.focused_field + 1;
                if next < Self::MODEL_POLICY_FIELD_COUNT {
                    self.model_policy_state.focused_field = next;
                }
            }
            KeyCode::Char('k') if !self.model_policy_text_field_focused() => {
                if self.model_policy_state.focused_field > 0 {
                    self.model_policy_state.focused_field -= 1;
                }
            }
            KeyCode::Enter => self.handle_model_policy_builder_enter(),
            KeyCode::Char(' ') => self.cycle_model_policy_field(),
            KeyCode::Backspace => match self.model_policy_state.focused_field {
                0 => {
                    match &mut self.model_policy_state.scope {
                        ModelScope::All => {}
                        ModelScope::Provider(id) | ModelScope::Role(id) | ModelScope::Agent(id) => {
                            id.pop();
                        }
                    };
                }
                2 => {
                    self.model_policy_state.model.pop();
                    // An edited value is free text, not a picked entry.
                    self.model_policy_state.model_choice_index = None;
                    self.refresh_model_policy_choices();
                }
                _ => {}
            },
            KeyCode::Char(c) => match self.model_policy_state.focused_field {
                0 => match &mut self.model_policy_state.scope {
                    ModelScope::All => {}
                    ModelScope::Provider(id) | ModelScope::Role(id) | ModelScope::Agent(id) => {
                        id.push(c);
                    }
                },
                2 => {
                    self.model_policy_state.model.push(c);
                    // An edited value is free text, not a picked entry.
                    self.model_policy_state.model_choice_index = None;
                    self.refresh_model_policy_choices();
                }
                _ => {}
            },
            _ => {}
        }
        self.dirty = true;
    }

    /// Re-derive the builder's model and reasoning pickers from
    /// `catalog/model-registry.json` for the harness and model now selected.
    ///
    /// The registry is the same fail-closed allowlist `scripts/model-policy.mjs`
    /// validates against, so offering it here means the builder proposes only
    /// values the script will accept. A missing registry is not fatal: the
    /// model field stays free text and the reasoning cycle falls back to the
    /// pre-registry union.
    fn refresh_model_policy_choices(&mut self) {
        self.model_policy_state
            .refresh_choices(self.catalog.model_registry.as_ref());
    }

    /// True when the focused model-policy builder field accepts free text
    /// (the scope id for provider/role/agent scopes, or the model name).
    fn model_policy_text_field_focused(&self) -> bool {
        match self.model_policy_state.focused_field {
            0 => !matches!(self.model_policy_state.scope, ModelScope::All),
            2 => true,
            _ => false,
        }
    }

    /// Cycle/toggle the focused model-policy builder field.
    fn cycle_model_policy_field(&mut self) {
        match self.model_policy_state.focused_field {
            0 => {
                // Cycle scope kind: All -> Provider -> Role -> Agent -> All.
                self.model_policy_state.scope = match &self.model_policy_state.scope {
                    ModelScope::All => ModelScope::Provider(
                        self.get_provider_list()
                            .first()
                            .map(|(p, _)| p.clone())
                            .unwrap_or_default(),
                    ),
                    ModelScope::Provider(_) => ModelScope::Role(
                        self.get_role_list()
                            .first()
                            .map(|r| r.0.clone())
                            .unwrap_or_default(),
                    ),
                    ModelScope::Role(_) => ModelScope::Agent(
                        self.catalog
                            .agents
                            .first()
                            .map(|a| a.id.clone())
                            .unwrap_or_default(),
                    ),
                    ModelScope::Agent(_) => ModelScope::All,
                };
            }
            1 => {
                self.model_policy_state.harness_index =
                    (self.model_policy_state.harness_index + 1) % CAPABLE_HARNESSES.len();
                // Both pickers are harness-scoped; re-derive them rather than
                // carrying the previous harness's models or efforts over.
                self.refresh_model_policy_choices();
            }
            2 => {
                if self.model_policy_state.model_choices.is_empty() {
                    self.status_message = Some((
                        "No model registry loaded — type a model name".to_string(),
                        Instant::now(),
                    ));
                    return;
                }
                self.model_policy_state.cycle_model();
                // A different model can support a different effort set.
                self.refresh_model_policy_choices();
            }
            3 => {
                if self.model_policy_state.reasoning_supported() {
                    let len = self.model_policy_state.reasoning_cycle.len();
                    self.model_policy_state.reasoning_index =
                        (self.model_policy_state.reasoning_index + 1) % len;
                } else {
                    let harness = self.model_policy_state.harness();
                    self.status_message = Some((
                        format!("The {harness} harness has no reasoning-effort field"),
                        Instant::now(),
                    ));
                }
            }
            4 => self.model_policy_state.dry_run = !self.model_policy_state.dry_run,
            5 => {
                self.model_policy_state.refresh_integrity =
                    !self.model_policy_state.refresh_integrity;
            }
            _ => {}
        }
    }

    /// Handle Enter in the model policy builder — cycle/toggle the focused
    /// field, or validate and continue to the confirm view on `[ Continue ]`.
    fn handle_model_policy_builder_enter(&mut self) {
        if self.model_policy_state.focused_field != Self::MODEL_POLICY_FIELD_COUNT - 1 {
            self.cycle_model_policy_field();
            return;
        }
        // Validate scope references against the loaded catalog before
        // shelling out — the script re-validates, but fail fast in the UI.
        let scope_error = match &self.model_policy_state.scope {
            ModelScope::All => None,
            ModelScope::Provider(id) => {
                if self.get_provider_list().iter().any(|(p, _)| p == id) {
                    None
                } else {
                    Some(format!("Unknown provider: {id}"))
                }
            }
            ModelScope::Role(id) => {
                if self.catalog.roles.contains_key(id) {
                    None
                } else {
                    Some(format!("Unknown role: {id}"))
                }
            }
            ModelScope::Agent(id) => {
                if self.catalog.agent_by_id(id).is_some() {
                    None
                } else {
                    Some(format!("Unknown agent: {id}"))
                }
            }
        };
        if let Some(msg) = scope_error {
            self.status_message = Some((msg, Instant::now()));
            return;
        }
        let cmd = self.model_policy_state.command();
        if let Err(e) = cmd.validate() {
            self.status_message = Some((format!("Validation error: {e}"), Instant::now()));
            return;
        }
        self.nav.push_view(View::ModelPolicyConfirm);
    }

    /// Handle key events in the ModelPolicyConfirm view.
    fn handle_model_policy_confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                self.nav.pop_view();
            }
            KeyCode::Enter => self.execute_model_policy(),
            _ => {}
        }
    }

    /// Handle key events in the ModelPolicyOutput view.
    ///
    /// All three "leave" keys must gate on [`Self::subprocess_busy`], not
    /// just `subprocess_handle.is_some()`: after `execute_model_policy` (or
    /// the auto-chained integrity refresh) spawns the subprocess, there is a
    /// window — until `tick()` drains `pending_subprocess` — where a real
    /// child process is already running but `subprocess_handle` is still
    /// `None`. Popping the view or quitting during that window would let
    /// `pending_subprocess` (and the `SubprocessHandle` it eventually
    /// delivers, which has `kill_on_drop(true)`) get dropped without a
    /// graceful SIGTERM, SIGKILLing an in-flight catalog write mid-file.
    fn handle_model_policy_output_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') if !self.subprocess_busy() => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.subprocess_busy() {
                    self.cancel_model_policy_subprocess();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Esc => {
                if self.subprocess_busy() {
                    // Attempt a graceful cancel (a no-op until tick() has
                    // adopted a real subprocess_handle to SIGTERM) and stay
                    // on this view rather than silently popping — popping
                    // away would reach ModelPolicyConfirm/other views whose
                    // 'q' handlers quit unconditionally, which is the same
                    // ungraceful-kill hazard one level removed.
                    self.cancel_model_policy_subprocess();
                } else {
                    self.nav.pop_view();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.nav.detail_scroll = self.nav.detail_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.nav.detail_scroll = self.nav.detail_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Execute the model policy mutation as a subprocess:
    /// `node scripts/model-policy.mjs set …`.
    fn execute_model_policy(&mut self) {
        // Defense in depth: never launch over a live gate/subprocess, even if
        // the builder was somehow reached while one was running.
        if self.subprocess_busy() {
            self.status_message = Some((
                "A subprocess is already running; wait for it to finish".to_string(),
                Instant::now(),
            ));
            return;
        }
        let cmd = self.model_policy_state.command();
        self.subprocess_output.clear();
        self.nav.detail_scroll = 0;

        let args = [vec!["scripts/model-policy.mjs".to_string()], cmd.to_args()].concat();
        let workspace = self.workspace_root.clone();

        let (tx, rx) = tokio::sync::oneshot::channel();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let result =
                    SubprocessExecutor::spawn("node", &args, &workspace, Duration::from_secs(300))
                        .await;
                let _ = tx.send(result);
            });
            self.pending_subprocess = Some(rx);
        }
        self.model_policy_stage = Some(ModelPolicyStage::Apply);
        self.nav.push_view(View::ModelPolicyOutput);
        tracing::info!(
            command = %cmd.display_command(),
            dry_run = cmd.dry_run,
            "model policy set started"
        );
    }

    /// Chain `npm run asset-integrity:write` after a successful policy apply
    /// so the integrity manifest matches the projected harness files.
    fn spawn_integrity_refresh(&mut self) {
        self.subprocess_output.push(output::OutputLine {
            content: "── refreshing asset-integrity manifest ──".to_string(),
            stream: crate::subprocess::OutputStream::Stdout,
        });
        let workspace = self.workspace_root.clone();
        let args = vec!["run".to_string(), "asset-integrity:write".to_string()];
        let (tx, rx) = tokio::sync::oneshot::channel();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let result =
                    SubprocessExecutor::spawn("npm", &args, &workspace, Duration::from_secs(300))
                        .await;
                let _ = tx.send(result);
            });
            self.pending_subprocess = Some(rx);
        }
        self.model_policy_stage = Some(ModelPolicyStage::IntegrityRefresh);
        tracing::info!("asset-integrity refresh started after model policy apply");
    }

    /// Cancel the model policy subprocess.
    fn cancel_model_policy_subprocess(&mut self) {
        if let Some(mut handle) = self.subprocess_handle.take() {
            tokio::spawn(async move {
                handle.cancel().await.ok();
            });
            self.model_policy_stage = None;
            self.status_message = Some((
                "Model policy operation cancelled by user".to_string(),
                Instant::now(),
            ));
        }
    }

    fn handle_down(&mut self) {
        match &self.nav.current_view {
            View::AgentDetail(_)
            | View::SkillDetail(_)
            | View::McpDetail(_)
            | View::RuleDetail(_)
            | View::WorkflowDetail(_)
            | View::IntegrityDetail(_)
            | View::RoleDetail(_) => {
                self.nav.detail_scroll = self.nav.detail_scroll.saturating_add(1);
            }
            _ => {
                let max = self.current_list_len();
                self.nav.select_next(max);
            }
        }
    }

    fn handle_up(&mut self) {
        match &self.nav.current_view {
            View::AgentDetail(_)
            | View::SkillDetail(_)
            | View::McpDetail(_)
            | View::RuleDetail(_)
            | View::WorkflowDetail(_)
            | View::IntegrityDetail(_)
            | View::RoleDetail(_) => {
                self.nav.detail_scroll = self.nav.detail_scroll.saturating_sub(1);
            }
            _ => self.nav.select_prev(),
        }
    }

    fn handle_enter(&mut self) {
        let idx = self.nav.selected_index();
        match self.nav.current_view.clone() {
            View::AgentList => {
                if let Some(&real_idx) = self.filtered_indices.get(idx) {
                    if let Some(agent) = self.catalog.agents.get(real_idx) {
                        self.nav.push_view(View::AgentDetail(agent.id.clone()));
                    }
                }
            }
            View::SkillList => {
                if let Some(skill) = self.catalog.skills.get(idx) {
                    self.nav.push_view(View::SkillDetail(skill.id.clone()));
                }
            }
            View::RoleList => {
                let roles: Vec<String> = self.get_role_list().iter().map(|r| r.0.clone()).collect();
                if let Some(role_id) = roles.get(idx) {
                    self.nav.push_view(View::RoleDetail(role_id.clone()));
                }
            }
            View::ProviderList => {
                let providers = self.get_provider_list();
                if let Some((provider, _)) = providers.get(idx) {
                    self.nav.push_view(View::ProviderAgents(provider.clone()));
                }
            }
            View::McpList => {
                if let Some(mcp) = self.catalog.mcp_refs.get(idx) {
                    self.nav.push_view(View::McpDetail(mcp.id.clone()));
                }
            }
            View::RuleList => {
                if let Some(rule) = self.catalog.rules.get(idx) {
                    self.nav.push_view(View::RuleDetail(rule.id.clone()));
                }
            }
            View::WorkflowList => {
                if let Some(wf) = self.workflows().get(idx) {
                    self.nav.push_view(View::WorkflowDetail(wf.id.clone()));
                }
            }
            View::ProviderAgents(ref provider) => {
                let agents = self.catalog.agents_by_provider(provider);
                if let Some(agent) = agents.get(idx) {
                    self.nav.push_view(View::AgentDetail(agent.id.clone()));
                }
            }
            View::IntegrityOverview => {
                if let Some(integrity) = &self.catalog.integrity {
                    // idx 0 is the summary line, idx 1..=trees.len() are trees,
                    // last entry (if root_files exist) is root_files
                    let tree_count = integrity.trees.len();
                    if idx == 0 {
                        // Summary line — no drill-down
                    } else if idx <= tree_count {
                        if let Some(tree) = integrity.trees.get(idx - 1) {
                            self.nav.push_view(View::IntegrityDetail(tree.tree.clone()));
                        }
                    } else if !integrity.root_files.is_empty() && idx == tree_count + 1 {
                        self.nav
                            .push_view(View::IntegrityDetail("__root_files__".to_string()));
                    }
                }
            }
            View::ValidationList => {
                self.handle_validation_enter(idx);
            }
            _ => {}
        }
    }

    /// Handle Enter key on the ValidationList view.
    ///
    /// The validation list has N+1 entries: the first entry is "Run All Validations"
    /// (invokes `npm run validate`), followed by individual gates.
    fn handle_validation_enter(&mut self, idx: usize) {
        // Index 0 = "Run All Validations", 1..N = individual gates
        if idx == 0 {
            // "Run All Validations" — check if any gate is already running
            if self.running_gate.is_some() {
                self.status_message = Some((
                    "A validation gate is already running".to_string(),
                    Instant::now(),
                ));
                return;
            }
            self.spawn_validation_gate(RUN_ALL_LABEL.to_string(), "validate".to_string());
        } else {
            let gate_idx = idx - 1;
            if let Some(gate) = self.validation_gates.get(gate_idx) {
                // Prevent concurrent execution of the same gate (Requirement 6.6, 6.7)
                if gate.status == GateStatus::Running {
                    self.status_message = Some((
                        format!("{} is already running", gate.script_name),
                        Instant::now(),
                    ));
                    return;
                }
                // Also prevent if any gate is running (single subprocess at a time)
                if self.running_gate.is_some() {
                    self.status_message = Some((
                        "A validation gate is already running".to_string(),
                        Instant::now(),
                    ));
                    return;
                }
                let script_name = gate.script_name.clone();
                self.spawn_validation_gate(script_name.clone(), script_name);
            }
        }
    }

    /// Spawn a validation gate subprocess.
    ///
    /// `gate_label` is the name used for tracking (stored in `running_gate`).
    /// `script_name` is the npm script name to invoke (e.g., "validate:lint" or "validate").
    fn spawn_validation_gate(&mut self, gate_label: String, script_name: String) {
        // Defense in depth: never launch over a live gate/subprocess, mirrors
        // execute_model_policy's guard. Callers already check `running_gate`,
        // but that's `None` during a model-policy stage's pending/running
        // window, so without this a gate launch here would silently
        // overwrite `pending_subprocess`/`subprocess_handle` out from under
        // that operation.
        if self.subprocess_busy() {
            self.status_message = Some((
                "A subprocess is already running; wait for it to finish".to_string(),
                Instant::now(),
            ));
            return;
        }
        // Mark the gate as Running
        if gate_label == RUN_ALL_LABEL {
            // Mark all gates as Running for "Run All"
            for gate in &mut self.validation_gates {
                gate.status = GateStatus::Running;
            }
        } else {
            for gate in &mut self.validation_gates {
                if gate.script_name == gate_label {
                    gate.status = GateStatus::Running;
                    break;
                }
            }
        }

        // Clear previous output
        self.subprocess_output.clear();

        // Spawn the subprocess asynchronously via oneshot channel to avoid
        // `block_on` panic inside an existing tokio runtime.
        let workspace = self.workspace_root.clone();
        let timeout = Duration::from_secs(VALIDATION_GATE_TIMEOUT_SECS);
        let args = vec!["run".to_string(), script_name.clone()];

        let (tx, rx) = tokio::sync::oneshot::channel();
        // Only spawn when a Tokio runtime is in context — always true inside the
        // real TUI event loop (`run_tui_async`). Synchronous tests that drive
        // `handle_key_event` directly have no reactor, so skip the spawn there
        // rather than panicking ("there is no reactor running").
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let result =
                    crate::subprocess::SubprocessExecutor::spawn("npm", &args, &workspace, timeout)
                        .await;
                let _ = tx.send(result);
            });
            // Store receiver to poll in tick().
            self.pending_subprocess = Some(rx);
        }

        // Set gate tracking state immediately (deterministic regardless of spawn).
        self.running_gate = Some(gate_label.clone());
        self.running_gate_start = Some(Instant::now());
        self.nav.push_view(View::ValidationOutput(gate_label));
        tracing::info!(gate = %script_name, "validation gate started");
    }

    /// Mark a gate (or all gates for "Run All") as failed.
    fn mark_gate_failed(&mut self, gate_label: &str) {
        if gate_label == RUN_ALL_LABEL {
            for gate in &mut self.validation_gates {
                if gate.status == GateStatus::Running {
                    gate.status = GateStatus::Failed;
                }
            }
        } else {
            for gate in &mut self.validation_gates {
                if gate.script_name == *gate_label {
                    gate.status = GateStatus::Failed;
                    break;
                }
            }
        }
        self.running_gate = None;
        self.running_gate_start = None;
    }

    /// Mark the app as dirty (needs rendering) — used by async event loop.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Keep the legacy `current_view` consistent with the active v2 tab so key
    /// dispatch (Enter / j / k / g) targets what the tab actually renders.
    ///
    /// Only the tabs backed by a legacy view need syncing: `ValidationGates`
    /// renders the validation list, and `CatalogBrowser` renders the legacy
    /// catalog browser. Other v2 tabs render non-interactive widgets, so the
    /// legacy view is left untouched.
    fn sync_view_to_tab(&mut self) {
        use crate::ui::nav::Tab;
        // Land the CatalogBrowser tab on a catalog list view, but keep an
        // existing catalog sub-view (guard, not a nested `if`, to satisfy
        // clippy::collapsible_match).
        match self.nav.current_tab {
            Tab::ValidationGates => self.nav.current_view = View::ValidationList,
            Tab::CatalogBrowser
                if !matches!(
                    self.nav.current_view,
                    View::AgentList
                        | View::SkillList
                        | View::McpList
                        | View::RuleList
                        | View::WorkflowList
                        | View::RoleList
                        | View::ProviderList
                        | View::IntegrityOverview
                ) =>
            {
                self.nav.current_view = View::AgentList;
            }
            _ => {}
        }
    }

    /// Reload the entire catalog from the workspace root and refresh the
    /// filtered view (Task 9.1 — live reload on filesystem change).
    ///
    /// Best-effort: catalog load errors are surfaced via the status message but
    /// never panic, so a malformed edit keeps the previous render usable.
    pub fn reload_catalog(&mut self) {
        self.catalog = CatalogStore::load(&self.workspace_root);
        self.update_filtered();
        // The builder caches its pickers; a reload that changes (or removes)
        // catalog/model-registry.json must rebuild them, or the view would go
        // on offering values the freshly loaded registry no longer verifies.
        self.refresh_model_policy_choices();
        let msg = if self.catalog.load_errors.is_empty() {
            "Catalog reloaded".to_string()
        } else {
            format!(
                "Catalog reloaded with {} error(s)",
                self.catalog.load_errors.len()
            )
        };
        self.status_message = Some((msg, Instant::now()));
        self.mark_dirty();
    }

    /// Reload a single changed catalog file in place (Task 9.1).
    ///
    /// Returns the [`ReloadOutcome`] so callers (and tests) can observe whether
    /// the file was reloaded, retained on parse error, or unchanged.
    pub fn reload_catalog_file(
        &mut self,
        path: &std::path::Path,
    ) -> crate::catalog::store::ReloadOutcome {
        let outcome = self.catalog.reload_file(path);
        self.update_filtered();
        // Same reason as reload_catalog: keep the cached pickers in step with
        // whatever the store now holds.
        self.refresh_model_policy_choices();
        self.mark_dirty();
        outcome
    }

    /// Tick: clear expired status messages, drain subprocess output with sanitization,
    /// and check subprocess timeout and completion.
    pub fn tick(&mut self) {
        if let Some((_, created)) = &self.status_message {
            if created.elapsed().as_secs() > 10 {
                self.status_message = None;
            }
        }

        // Poll pending subprocess spawn result (from async tokio::spawn).
        if let Some(ref mut rx) = self.pending_subprocess {
            match rx.try_recv() {
                Ok(Ok(subprocess_handle)) => {
                    self.subprocess_handle = Some(subprocess_handle);
                    self.pending_subprocess = None;
                }
                Ok(Err(e)) => {
                    // Subprocess spawn failed
                    if let Some(ref gate_label) = self.running_gate.clone() {
                        self.mark_gate_failed(gate_label);
                        tracing::error!(gate = %gate_label, error = %e, "validation gate spawn failed");
                    }
                    self.model_policy_stage = None;
                    self.status_message =
                        Some((format!("Failed to start subprocess: {e}"), Instant::now()));
                    self.pending_subprocess = None;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                    // Not ready yet — keep waiting
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    // Sender dropped without sending — treat as error
                    if let Some(ref gate_label) = self.running_gate.clone() {
                        self.mark_gate_failed(gate_label);
                    }
                    self.model_policy_stage = None;
                    self.status_message = Some((
                        "Subprocess spawn channel closed unexpectedly".to_string(),
                        Instant::now(),
                    ));
                    self.pending_subprocess = None;
                }
            }
        }

        // Drain subprocess output lines, sanitizing before storage
        if let Some(handle) = &mut self.subprocess_handle {
            while let Some(line) = handle.try_recv_stdout() {
                self.subprocess_output.push(output::OutputLine {
                    content: crate::security::redact::redact_secrets(&sanitize_subprocess_output(
                        &line.content,
                    )),
                    stream: line.stream,
                });
            }
            while let Some(line) = handle.try_recv_stderr() {
                self.subprocess_output.push(output::OutputLine {
                    content: crate::security::redact::redact_secrets(&sanitize_subprocess_output(
                        &line.content,
                    )),
                    stream: line.stream,
                });
            }

            // Poll for subprocess exit (non-blocking)
            handle.try_poll_exit();

            // Check timeout synchronously by comparing elapsed time
            if handle.is_running() && handle.is_timed_out() {
                // Mark any active validation gate as timed out
                for gate in &mut self.validation_gates {
                    if gate.status == GateStatus::Running {
                        gate.status = GateStatus::TimedOut;
                    }
                }
                self.running_gate = None;
                self.running_gate_start = None;
                self.status_message = Some(("Subprocess timed out".to_string(), Instant::now()));
                self.subprocess_handle = None;
            } else if !handle.is_running() {
                // Subprocess has finished — update gate status based on exit code
                let exit_code = handle.exit_code();
                let duration = self.running_gate_start.map(|s| s.elapsed());

                if let Some(ref gate_label) = self.running_gate.clone() {
                    if gate_label == RUN_ALL_LABEL {
                        // "Run All" — mark all gates based on overall exit code
                        let status = if exit_code == Some(0) {
                            GateStatus::Passed
                        } else {
                            GateStatus::Failed
                        };
                        for gate in &mut self.validation_gates {
                            if gate.status == GateStatus::Running {
                                gate.status = status.clone();
                                gate.last_exit_code = exit_code;
                                gate.last_duration = duration;
                            }
                        }
                    } else {
                        // Individual gate
                        for gate in &mut self.validation_gates {
                            if gate.script_name == *gate_label && gate.status == GateStatus::Running
                            {
                                gate.status = if exit_code == Some(0) {
                                    GateStatus::Passed
                                } else {
                                    GateStatus::Failed
                                };
                                gate.last_exit_code = exit_code;
                                gate.last_duration = duration;
                                break;
                            }
                        }
                    }
                    tracing::info!(
                        gate = %gate_label,
                        exit_code = ?exit_code,
                        duration_ms = ?duration.map(|d| d.as_millis()),
                        "validation gate completed"
                    );
                } else if let Some(stage) = self.model_policy_stage {
                    match stage {
                        ModelPolicyStage::Apply => {
                            if exit_code == Some(0) {
                                if self.model_policy_state.dry_run {
                                    self.model_policy_stage = None;
                                    self.status_message = Some((
                                        "Dry-run complete — review the planned changes above"
                                            .to_string(),
                                        Instant::now(),
                                    ));
                                    tracing::info!("model policy dry-run completed");
                                } else if self.model_policy_state.refresh_integrity {
                                    // Chain the integrity manifest refresh so the
                                    // published tree and manifest stay in lockstep.
                                    self.spawn_integrity_refresh();
                                } else {
                                    self.model_policy_stage = None;
                                    self.status_message = Some((
                                        "Model policy applied — run asset-integrity:write before committing"
                                            .to_string(),
                                        Instant::now(),
                                    ));
                                    tracing::info!(
                                        "model policy applied (integrity refresh skipped)"
                                    );
                                }
                            } else {
                                self.model_policy_stage = None;
                                self.status_message = Some((
                                    format!(
                                        "Model policy update failed with exit code {}",
                                        exit_code.unwrap_or(-1)
                                    ),
                                    Instant::now(),
                                ));
                                tracing::warn!(exit_code = ?exit_code, "model policy set failed");
                            }
                        }
                        ModelPolicyStage::IntegrityRefresh => {
                            self.model_policy_stage = None;
                            if exit_code == Some(0) {
                                self.status_message = Some((
                                    "Model policy applied and integrity manifest refreshed"
                                        .to_string(),
                                    Instant::now(),
                                ));
                                tracing::info!("model policy apply + integrity refresh completed");
                            } else {
                                self.status_message = Some((
                                    format!(
                                        "Integrity refresh failed with exit code {} — run npm run asset-integrity:write manually",
                                        exit_code.unwrap_or(-1)
                                    ),
                                    Instant::now(),
                                ));
                                tracing::warn!(exit_code = ?exit_code, "integrity refresh failed");
                            }
                        }
                    }
                } else {
                    // Export subprocess completed (no running_gate set)
                    if exit_code != Some(0) {
                        self.status_message = Some((
                            format!("Export failed with exit code {}", exit_code.unwrap_or(-1)),
                            Instant::now(),
                        ));
                        tracing::warn!(
                            exit_code = ?exit_code,
                            "export command failed"
                        );
                    } else {
                        self.status_message =
                            Some(("Export completed successfully".to_string(), Instant::now()));
                        tracing::info!("export command completed successfully");
                    }
                }

                self.running_gate = None;
                self.running_gate_start = None;
                // Final drain after process exits — capture any remaining buffered lines
                if let Some(ref mut handle) = self.subprocess_handle {
                    while let Some(line) = handle.try_recv_stdout() {
                        self.subprocess_output.push(output::OutputLine {
                            content: crate::security::redact::redact_secrets(
                                &sanitize_subprocess_output(&line.content),
                            ),
                            stream: line.stream,
                        });
                    }
                    while let Some(line) = handle.try_recv_stderr() {
                        self.subprocess_output.push(output::OutputLine {
                            content: crate::security::redact::redact_secrets(
                                &sanitize_subprocess_output(&line.content),
                            ),
                            stream: line.stream,
                        });
                    }
                }
                self.subprocess_handle = None;
            }
        }

        // Cap subprocess output to prevent unbounded memory growth
        if self.subprocess_output.len() > MAX_SUBPROCESS_OUTPUT_LINES {
            let overflow = self.subprocess_output.len() - MAX_SUBPROCESS_OUTPUT_LINES;
            self.subprocess_output.drain(0..overflow);
        }
    }

    /// Render the full UI — v2 primary surface: tab bar + tab body.
    pub fn render(&mut self, frame: &mut Frame) {
        use crate::ui::nav::Tab;
        use ratatui::layout::{Constraint, Direction, Layout};

        let theme = Theme::new(self.no_color, self.theme_mode);
        let area = frame.area();

        // Split into: tab bar (3 rows), body (flexible), status (1), help (1).
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // tab bar
                Constraint::Min(0),    // tab body
                Constraint::Length(1), // status bar
                Constraint::Length(1), // help bar
            ])
            .split(area);

        // Render the tab bar.
        self.render_tab_bar(chunks[0], frame, &theme);

        // Render the active tab body.
        // Tabs that need &mut self are dispatched here; all others delegate to
        // render_tab (which is &self).
        let body = chunks[1];
        match self.nav.current_tab {
            Tab::CatalogBrowser => {
                // Legacy sidebar + main-content layout so all legacy render helpers
                // remain reachable and exercised when the user visits this tab.
                let legacy = compute_layout(body);
                self.render_sidebar(&legacy.sidebar, frame, &theme);
                self.render_main_content(&legacy.main_content, frame, &theme);
            }
            Tab::ValidationGates => {
                // Validation-gate list uses stateful rendering (&mut self) and is
                // wired to this tab so render_validation_list stays reachable.
                self.render_validation_list(body, frame, &theme);
            }
            _ => self.render_tab(body, frame, &theme),
        }

        // Status bar.
        let (visible, total) = self.get_counts();
        let filter_str = if let Some((msg, _)) = &self.status_message {
            msg.clone()
        } else if self.search_query.is_empty() {
            String::new()
        } else {
            self.search_query.clone()
        };
        let session_str = self.session_id.to_string();
        status_bar::render_status_bar(
            visible,
            total,
            &filter_str,
            &session_str,
            chunks[2],
            frame,
            &theme,
        );

        help_bar::render_help_bar(&self.nav.current_view, chunks[3], frame, &theme);

        // Render help overlay on top if active.
        if self.show_help_overlay {
            self.render_help_overlay(frame, &theme);
        }
    }

    /// Render the v2 tab bar showing all tabs with the current tab highlighted.
    fn render_tab_bar(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        use crate::ui::nav::Tab;
        use ratatui::text::Line;
        use ratatui::widgets::{Block, Borders, Tabs};

        let titles: Vec<Line> = Tab::ALL.iter().map(|t| Line::from(t.label())).collect();
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Operator Console "),
            )
            .select(self.nav.current_tab.index())
            .style(theme.list_item())
            .highlight_style(theme.list_selected());
        frame.render_widget(tabs, area);
    }

    /// Render the active v2 operator-console tab into `area` (Task 11.3).
    ///
    /// Dispatches to the v2 widgets based on `self.nav.current_tab`. The
    /// Dependencies tab renders the real catalog dependency graph and Overview a
    /// catalog summary; tabs whose live data depends on a workspace scan/index
    /// (coverage, violations, audit) render their widget with empty data until
    /// that pipeline is wired into the TUI flow.
    pub fn render_tab(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        use crate::ui::nav::Tab;
        match self.nav.current_tab {
            Tab::Dependencies => {
                let graph = crate::federation::dep_graph::DependencyGraph::build(&self.catalog);
                let state = dep_graph::DepGraphState::default();
                dep_graph::render_dep_graph(&graph, &state, area, frame, theme);
            }
            Tab::CoverageMatrix => {
                let matrix = crate::models::coverage::CoverageMatrix {
                    rows: Vec::new(),
                    columns: Vec::new(),
                    cells: std::collections::HashMap::new(),
                    workspace_scores: std::collections::HashMap::new(),
                };
                let state = coverage_grid::CoverageGridState::default();
                let filter = coverage_grid::CoverageGridFilter::default();
                coverage_grid::render_coverage_grid(&matrix, &state, &filter, area, frame, theme);
            }
            Tab::PolicyViolations => {
                let state = violations::ViolationsState::default();
                violations::render_violations(&[], &[], &state, area, frame, theme);
            }
            Tab::AuditLog => {
                let state = audit_log::AuditLogState::default();
                audit_log::render_audit_log(&[], &state, area, frame, theme);
            }
            _ => self.render_tab_overview(area, frame, theme),
        }
    }

    /// Render the Overview tab — a catalog summary (Task 11.3).
    fn render_tab_overview(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
        let body = format!(
            "Vanguard Frontier — Operator Console\n\n\
             Agents:    {}\n\
             Skills:    {}\n\
             MCP refs:  {}\n\
             Rules:     {}\n\
             Providers: {}\n\n\
             Tab/Shift-Tab to switch tabs.",
            self.catalog.agent_count(),
            self.catalog.skill_count(),
            self.catalog.mcp_refs.len(),
            self.catalog.rules.len(),
            self.catalog.provider_count(),
        );
        let paragraph = Paragraph::new(body)
            .style(theme.list_item())
            .block(Block::default().borders(Borders::ALL).title(" Overview "))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}

/// Build the (harness, description) rows for the agent-detail "Models"
/// section from resolved model assignments (`catalog/model-assignments.json`
/// via `CatalogStore::model_assignments_for_agent`).
///
/// When an assignment's `model_warning` is set (provider lifecycle:
/// `status: "retiring"` on the pinned model, or `"retired"` with a
/// successor substituted into `model`), an additional synthetic row is
/// emitted directly under that harness's row with label `"warning"` and a
/// `"warning: "`-prefixed value carrying the engine-composed text verbatim —
/// the TUI never derives lifecycle wording itself, it only renders what
/// `scripts/model-policy.mjs` already decided. `detail::render_agent_detail`
/// recognizes the `"warning"` label and styles that row in the theme's
/// warning colour.
fn build_model_lines(assignments: &[&crate::models::ModelAssignment]) -> Vec<(String, String)> {
    let mut lines = Vec::new();
    for a in assignments {
        let model = a.model.as_deref().unwrap_or("auto (harness default)");
        let mut description = model.to_string();
        if let Some(provider) = a.model_provider.as_deref() {
            description.push_str(&format!(" via {provider}"));
        }
        if a.harness == "codex" {
            let reasoning = a.reasoning_effort.as_deref().unwrap_or("auto");
            description.push_str(&format!(" · reasoning={reasoning}"));
        }
        if a.model_source != "default" {
            description.push_str(&format!(" · rule: {}", a.model_source));
        }
        lines.push((a.harness.clone(), description));
        if let Some(warning) = a.model_warning.as_deref() {
            lines.push(("warning".to_string(), format!("warning: {warning}")));
        }
    }
    lines
}

// Legacy sidebar / main-content render helpers — called from render() when the
// CatalogBrowser or ValidationGates tab is active (so they are never dead code).
impl App {
    fn render_sidebar(&mut self, area: &ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        let items: Vec<String> = SIDEBAR_SECTIONS
            .iter()
            .map(|(s, _)| s.to_string())
            .collect();
        let mut sidebar_state = ratatui::widgets::ListState::default();
        sidebar_state.select(Some(self.nav.sidebar_index));
        list_view::render_list_view(&items, &mut sidebar_state, "Catalog", *area, frame, theme);
    }

    fn render_main_content(
        &mut self,
        area: &ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        // Render filter chips at the top if any filters are active (agent list view)
        let (content_area, has_chips) = if self.nav.current_view == View::AgentList
            && (self.provider_filter.is_some()
                || self.harness_filter.is_some()
                || !self.search_query.is_empty())
        {
            let chips_height = 1u16;
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(chips_height),
                    ratatui::layout::Constraint::Min(0),
                ])
                .split(*area);
            self.render_filter_chips(chunks[0], frame, theme);
            (chunks[1], true)
        } else {
            (*area, false)
        };
        let _ = has_chips;

        match self.nav.current_view.clone() {
            View::AgentList => self.render_agent_list(content_area, frame, theme),
            View::AgentDetail(ref id) => {
                self.render_agent_detail_view(id, content_area, frame, theme)
            }
            View::SkillList => self.render_skill_list(content_area, frame, theme),
            View::SkillDetail(ref id) => {
                self.render_skill_detail_view(id, content_area, frame, theme)
            }
            View::RoleList => self.render_role_list(content_area, frame, theme),
            View::RoleDetail(ref id) => {
                self.render_role_detail_view(id, content_area, frame, theme)
            }
            View::ProviderList => self.render_provider_list(content_area, frame, theme),
            View::ProviderAgents(ref p) => {
                self.render_provider_agents(p, content_area, frame, theme)
            }
            View::McpList => self.render_mcp_list(content_area, frame, theme),
            View::McpDetail(ref id) => self.render_mcp_detail_view(id, content_area, frame, theme),
            View::RuleList => self.render_rule_list(content_area, frame, theme),
            View::WorkflowList => self.render_workflow_list(content_area, frame, theme),
            View::WorkflowDetail(ref id) => {
                self.render_workflow_detail_view(id, content_area, frame, theme)
            }
            View::RuleDetail(ref id) => {
                self.render_rule_detail_view(id, content_area, frame, theme)
            }
            View::ValidationList => self.render_validation_list(content_area, frame, theme),
            View::ValidationOutput(ref name) => {
                let name = name.clone();
                self.render_validation_output(&name, content_area, frame, theme);
            }
            View::ExportBuilder => self.render_export_builder(content_area, frame, theme),
            View::ExportConfirm => self.render_export_confirm(content_area, frame, theme),
            View::ExportOutput => self.render_export_output(content_area, frame, theme),
            View::ModelPolicyBuilder => {
                self.render_model_policy_builder(content_area, frame, theme)
            }
            View::ModelPolicyConfirm => {
                self.render_model_policy_confirm(content_area, frame, theme)
            }
            View::ModelPolicyOutput => self.render_model_policy_output(content_area, frame, theme),
            View::IntegrityOverview => self.render_integrity_view(content_area, frame, theme),
            View::IntegrityDetail(ref tree) => {
                let tree = tree.clone();
                self.render_integrity_detail(&tree, content_area, frame, theme);
            }
        }

        if self.search_active || !self.search_query.is_empty() {
            let search_area = ratatui::layout::Rect {
                x: content_area.x,
                y: content_area.y,
                width: content_area.width.min(40),
                height: 3,
            };
            search::render_search_input(
                &self.search_query,
                self.search_active,
                search_area,
                frame,
                theme,
            );
        }
    }

    fn render_agent_list(&mut self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        let items: Vec<String> = self
            .filtered_indices
            .iter()
            .filter_map(|&i| {
                self.catalog.agents.get(i).map(|a| {
                    format!(
                        "{} - {}",
                        a.id,
                        a.summary.chars().take(60).collect::<String>()
                    )
                })
            })
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Agents",
            area,
            frame,
            theme,
        );
    }

    fn render_agent_detail_view(
        &self,
        id: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        if let Some(agent) = self.catalog.agents.iter().find(|a| a.id == id) {
            let roles = self.catalog.roles_containing_agent(id);
            let assignments = self.catalog.model_assignments_for_agent(id);
            let model_lines = build_model_lines(&assignments);
            detail::render_agent_detail(
                agent,
                &roles,
                &model_lines,
                area,
                frame,
                self.nav.detail_scroll,
                theme,
            );
        }
    }

    fn render_skill_list(&mut self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        let items: Vec<String> = self
            .catalog
            .skills
            .iter()
            .map(|s| {
                format!(
                    "{} - {}",
                    s.id,
                    s.summary.chars().take(60).collect::<String>()
                )
            })
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Skills",
            area,
            frame,
            theme,
        );
    }

    fn render_skill_detail_view(
        &self,
        id: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        if let Some(skill) = self.catalog.skills.iter().find(|s| s.id == id) {
            let related = self.catalog.agents_with_skill(id);
            detail::render_skill_detail(
                skill,
                &related,
                area,
                frame,
                self.nav.detail_scroll,
                theme,
            );
        }
    }

    fn render_role_list(&mut self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        let roles = self.get_role_list();
        let items: Vec<String> = roles
            .iter()
            .map(|(id, label, count)| format!("{id} ({label}) - {count} agents"))
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Roles",
            area,
            frame,
            theme,
        );
    }

    fn render_role_detail_view(
        &self,
        id: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let agents = self.catalog.agents_for_role(id);
        let items: Vec<ratatui::text::Line> = agents
            .iter()
            .map(|a| ratatui::text::Line::from(format!("  - {}", a.id)))
            .collect();
        let mut lines = vec![ratatui::text::Line::from(format!("Role: {id}"))];
        if let Some(role) = self.catalog.roles.get(id) {
            lines.push(ratatui::text::Line::from(format!("Label: {}", role.label)));
            lines.push(ratatui::text::Line::from(format!(
                "Description: {}",
                role.description
            )));
        }
        lines.push(ratatui::text::Line::from(format!(
            "Agents ({}): ",
            agents.len()
        )));
        lines.extend(items);
        let paragraph = ratatui::widgets::Paragraph::new(lines)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Role Detail")
                    .border_style(theme.border_style()),
            )
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((self.nav.detail_scroll, 0));
        frame.render_widget(paragraph, area);
    }

    fn render_provider_list(
        &mut self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let providers = self.get_provider_list();
        let max_count = providers.iter().map(|(_, c)| *c).max().unwrap_or(1);
        let bar_width = 20usize;
        let items: Vec<String> = providers
            .iter()
            .map(|(p, count)| {
                let filled = if max_count > 0 {
                    ((*count as f64 / max_count as f64) * bar_width as f64).round() as usize
                } else {
                    0
                };
                let empty = bar_width.saturating_sub(filled);
                let bar: String = "█".repeat(filled) + &"░".repeat(empty);
                format!("{p} ({count} agents) {bar}")
            })
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Providers",
            area,
            frame,
            theme,
        );
    }

    fn render_provider_agents(
        &mut self,
        provider: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let agents = self.catalog.agents_by_provider(provider);
        let items: Vec<String> = agents
            .iter()
            .map(|a| {
                format!(
                    "{} - {}",
                    a.id,
                    a.summary.chars().take(60).collect::<String>()
                )
            })
            .collect();
        let title = format!("Agents: {provider}");
        list_view::render_list_view(&items, &mut self.nav.list_state, &title, area, frame, theme);
    }

    fn render_mcp_list(&mut self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        let items: Vec<String> = self
            .catalog
            .mcp_refs
            .iter()
            .map(|m| {
                format!(
                    "{} - {}",
                    m.id,
                    m.summary.chars().take(60).collect::<String>()
                )
            })
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "MCP References",
            area,
            frame,
            theme,
        );
    }

    fn render_mcp_detail_view(
        &self,
        id: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        if let Some(mcp) = self.catalog.mcp_refs.iter().find(|m| m.id == id) {
            detail::render_mcp_detail(mcp, area, frame, self.nav.detail_scroll, theme);
        }
    }

    fn render_rule_list(&mut self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        let items: Vec<String> = self
            .catalog
            .rules
            .iter()
            .map(|r| {
                format!(
                    "{} - {}",
                    r.id,
                    r.summary.chars().take(60).collect::<String>()
                )
            })
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Rules",
            area,
            frame,
            theme,
        );
    }

    /// Workflows from the generated catalog, or an empty slice when the catalog file is
    /// absent — a checkout without any workflow renders an empty list rather than an
    /// error, matching how the loader treats a missing file.
    fn workflows(&self) -> &[crate::models::Workflow] {
        self.catalog
            .workflows
            .as_ref()
            .map(|c| c.workflows.as_slice())
            .unwrap_or(&[])
    }

    fn render_workflow_list(
        &mut self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let items: Vec<String> = self
            .workflows()
            .iter()
            .map(|w| {
                format!(
                    "{} [{}] - {}",
                    w.invocation(),
                    w.model_tiers().join("/"),
                    w.description.chars().take(60).collect::<String>()
                )
            })
            .collect();
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Workflows",
            area,
            frame,
            theme,
        );
    }

    fn render_workflow_detail_view(
        &self,
        id: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        if let Some(wf) = self.workflows().iter().find(|w| w.id == id) {
            detail::render_workflow_detail(wf, area, frame, self.nav.detail_scroll, theme);
        }
    }

    fn render_rule_detail_view(
        &self,
        id: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        if let Some(rule) = self.catalog.rules.iter().find(|r| r.id == id) {
            detail::render_rule_detail(rule, area, frame, self.nav.detail_scroll, theme);
        }
    }

    fn render_validation_list(
        &mut self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        use ratatui::style::Modifier;
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, List, ListItem};

        let mut list_items: Vec<ListItem> = Vec::new();

        // First entry: "Run All Validations"
        let run_all_style = if self.running_gate.as_deref() == Some(RUN_ALL_LABEL) {
            theme.gate_running()
        } else {
            theme.gate_not_run()
        };
        let run_all_indicator = if self.running_gate.as_deref() == Some(RUN_ALL_LABEL) {
            self.animated_spinner()
        } else {
            String::new()
        };
        list_items.push(ListItem::new(Line::from(vec![Span::styled(
            format!("▶ Run All Validations{}", run_all_indicator),
            run_all_style,
        )])));

        // Individual gates
        for g in &self.validation_gates {
            let status_str = format!("{:?}", g.status);
            let timing_str = g
                .last_duration
                .map(|d| format!(" ({:.1}s)", d.as_secs_f64()))
                .unwrap_or_default();
            let style = match g.status {
                GateStatus::NotRun => theme.gate_not_run(),
                GateStatus::Running => theme.gate_running(),
                GateStatus::Passed => theme.gate_passed(),
                GateStatus::Failed => theme.gate_failed(),
                GateStatus::TimedOut => theme.gate_timed_out(),
            };
            let spinner = if g.status == GateStatus::Running {
                self.animated_spinner()
            } else {
                String::new()
            };
            let line = Line::from(vec![Span::styled(
                format!("{} [{}]{}{spinner}", g.script_name, status_str, timing_str),
                style,
            )]);
            list_items.push(ListItem::new(line));
        }

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Validation Gates")
                    .border_style(theme.border_style()),
            )
            .highlight_style(theme.list_selected().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.nav.list_state);
    }

    /// Generate an animated spinner character based on elapsed time.
    fn animated_spinner(&self) -> String {
        let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let elapsed = self
            .running_gate_start
            .map(|s| s.elapsed().as_millis() as usize)
            .unwrap_or(0);
        let idx = (elapsed / 100) % frames.len();
        format!(" {}", frames[idx])
    }

    fn render_validation_output(
        &self,
        _name: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        output::render_output_scrolled(
            &self.subprocess_output,
            "Validation Output",
            area,
            frame,
            self.nav.detail_scroll,
            theme,
        );
    }

    fn render_export_builder(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

        let selection_str = match &self.export_state.selection {
            ExportSelection::All => "All".to_string(),
            ExportSelection::Role(r) => format!("Role: {r}"),
            ExportSelection::Provider(p) => format!("Provider: {p}"),
            ExportSelection::Agents(ids) => format!("Agents: {}", ids.join(", ")),
        };

        let focused = self.export_state.focused_field;
        let fields = [
            format!("Platform: {}", self.export_state.platform),
            format!("Selection: {selection_str}"),
            format!("Target Repo: {}", self.export_state.target_repo),
            format!("Dry Run: {}", self.export_state.dry_run),
            format!("Force: {}", self.export_state.force),
            format!("No Skills: {}", self.export_state.no_skills),
        ];

        let mut lines: Vec<Line> = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if i == focused {
                    Line::from(Span::styled(format!("> {f}"), theme.list_selected()))
                } else {
                    Line::from(f.as_str().to_string())
                }
            })
            .collect();

        lines.push(Line::from(""));
        lines.push(Line::from("[Enter to confirm, Esc to cancel]"));

        // Tab completion suggestions
        if !self.completion_suggestions.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Suggestions:",
                theme.help_overlay_section(),
            )));
            for (i, suggestion) in self.completion_suggestions.iter().enumerate() {
                let style = if i == self.completion_index {
                    theme.completion_highlight()
                } else {
                    theme.completion_normal()
                };
                lines.push(Line::from(Span::styled(format!("  {suggestion}"), style)));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Export Builder")
                    .border_style(theme.border_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn render_export_confirm(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        use ratatui::text::Line;
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

        let target = PathBuf::from(&self.export_state.target_repo);
        let cmd = ExportCommand {
            platform: self.export_state.platform.clone(),
            selection: self.export_state.selection.clone(),
            target_repo: target,
            dry_run: self.export_state.dry_run,
            force: self.export_state.force,
            no_skills: self.export_state.no_skills,
        };

        let command_preview = cmd.display_command();

        let lines = vec![
            Line::from("Confirm export execution?"),
            Line::from(""),
            Line::from("Command:"),
            Line::from(format!("  {command_preview}")),
            Line::from(""),
            Line::from(format!("Dry Run: {}", self.export_state.dry_run)),
            Line::from(""),
            Line::from("[Enter to execute, Esc to cancel]"),
        ];
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Confirm Export")
                    .border_style(theme.border_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn render_export_output(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        // If dry-run, parse output for tree structure
        if self.export_state.dry_run && !self.subprocess_output.is_empty() {
            use ratatui::style::Color;
            use ratatui::text::{Line, Span};
            use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

            let mut tree_lines: Vec<Line> = Vec::new();
            let mut agent_entries: Vec<String> = Vec::new();
            let mut skill_entries: Vec<String> = Vec::new();

            for ol in &self.subprocess_output {
                let trimmed = ol.content.trim();
                if let Some(rest) = trimmed.strip_prefix("export agent:") {
                    agent_entries.push(rest.trim().to_string());
                } else if let Some(rest) = trimmed.strip_prefix("export skill:") {
                    skill_entries.push(rest.trim().to_string());
                }
            }

            if !agent_entries.is_empty() || !skill_entries.is_empty() {
                tree_lines.push(Line::from(Span::styled(
                    "Dry-Run Export Tree:",
                    theme.help_overlay_title(),
                )));
                tree_lines.push(Line::from(""));

                if !agent_entries.is_empty() {
                    tree_lines.push(Line::from(Span::styled("├── agents/", theme.detail_key())));
                    for (i, entry) in agent_entries.iter().enumerate() {
                        let prefix = if i == agent_entries.len() - 1 && skill_entries.is_empty() {
                            "│   └── "
                        } else {
                            "│   ├── "
                        };
                        let style = if theme.no_color {
                            ratatui::style::Style::default()
                        } else {
                            ratatui::style::Style::default().fg(Color::White)
                        };
                        tree_lines
                            .push(Line::from(Span::styled(format!("{prefix}{entry}"), style)));
                    }
                }

                if !skill_entries.is_empty() {
                    tree_lines.push(Line::from(Span::styled("└── skills/", theme.detail_key())));
                    for (i, entry) in skill_entries.iter().enumerate() {
                        let prefix = if i == skill_entries.len() - 1 {
                            "    └── "
                        } else {
                            "    ├── "
                        };
                        let style = if theme.no_color {
                            ratatui::style::Style::default()
                        } else {
                            ratatui::style::Style::default().fg(Color::White)
                        };
                        tree_lines
                            .push(Line::from(Span::styled(format!("{prefix}{entry}"), style)));
                    }
                }

                tree_lines.push(Line::from(""));
                tree_lines.push(Line::from(format!(
                    "Total: {} agents, {} skills",
                    agent_entries.len(),
                    skill_entries.len()
                )));

                // Append raw output below the tree
                tree_lines.push(Line::from(""));
                tree_lines.push(Line::from("─── Raw Output ───"));
                for ol in &self.subprocess_output {
                    let style = if theme.no_color {
                        ratatui::style::Style::default()
                    } else {
                        match ol.stream {
                            crate::subprocess::OutputStream::Stdout => {
                                ratatui::style::Style::default().fg(Color::White)
                            }
                            crate::subprocess::OutputStream::Stderr => {
                                ratatui::style::Style::default().fg(Color::Red)
                            }
                        }
                    };
                    tree_lines.push(Line::from(Span::styled(ol.content.clone(), style)));
                }

                let paragraph = Paragraph::new(tree_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Export Output (Dry-Run)")
                            .border_style(theme.border_style()),
                    )
                    .wrap(Wrap { trim: false })
                    .scroll((self.nav.detail_scroll, 0));
                frame.render_widget(paragraph, area);
            } else {
                output::render_output_scrolled(
                    &self.subprocess_output,
                    "Export Output",
                    area,
                    frame,
                    self.nav.detail_scroll,
                    theme,
                );
            }
        } else {
            output::render_output_scrolled(
                &self.subprocess_output,
                "Export Output",
                area,
                frame,
                self.nav.detail_scroll,
                theme,
            );
        }
    }

    fn render_model_policy_builder(
        &self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

        let state = &self.model_policy_state;
        let harness = state.harness();
        let total_choices = state.model_choices.len();
        let model_display = if state.model.is_empty() {
            if total_choices > 0 {
                format!("(unchanged — Space picks 1 of {total_choices} verified, or type)")
            } else {
                "(unchanged — type a model name, or \"auto\" to clear)".to_string()
            }
        } else {
            match state.selected_choice() {
                Some(choice) => format!(
                    "{}{}  [{} {}/{}]",
                    choice.model,
                    choice.label_suffix(),
                    choice.namespace,
                    state.model_choice_index.unwrap_or(0) + 1,
                    total_choices
                ),
                None => format!("{} (typed — validated by model-policy.mjs)", state.model),
            }
        };
        let reasoning_display = if state.reasoning_clear_only() {
            format!(
                "{}   [auto only — \"{}\" supports no effort on {harness}]",
                state.reasoning_display(),
                state.model
            )
        } else if state.reasoning_supported() {
            format!(
                "{}   [{}]",
                state.reasoning_display(),
                state.reasoning_cycle[1..].join(" ")
            )
        } else {
            format!("n/a ({harness} has no reasoning-effort field)")
        };

        let focused = state.focused_field;
        let fields = [
            format!("Scope: {}", state.scope.display()),
            format!("Harness: {harness}"),
            format!("Model: {model_display}"),
            format!("Reasoning: {reasoning_display}"),
            format!("Dry Run: {}", state.dry_run),
            format!("Refresh Integrity: {}", state.refresh_integrity),
            "[ Continue ]".to_string(),
        ];

        let mut lines: Vec<Line> = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if i == focused {
                    Line::from(Span::styled(format!("> {f}"), theme.list_selected()))
                } else {
                    Line::from(f.as_str().to_string())
                }
            })
            .collect();

        lines.push(Line::from(""));
        lines.push(Line::from(
            "Space/Enter cycles the field; type into Scope id and Model.",
        ));
        lines.push(Line::from(
            "\"auto\" removes the field so the harness default applies.",
        ));
        match self.catalog.model_registry.as_ref() {
            Some(registry) => {
                lines.push(Line::from(format!(
                    "Models and efforts come from catalog/model-registry.json (verified {}).",
                    registry.last_refreshed
                )));
                if let Some(choice) = state.selected_choice() {
                    if let Some(note) = &choice.note {
                        lines.push(Line::from(format!("Note: {note}")));
                    }
                }
            }
            None => lines.push(Line::from(
                "No model registry loaded — free-text model, fallback efforts.",
            )),
        }
        lines.push(Line::from("[Enter on Continue to preview, Esc to cancel]"));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Model Policy Builder")
                    .border_style(theme.border_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn render_model_policy_confirm(
        &self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        use ratatui::text::Line;
        use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

        let cmd = self.model_policy_state.command();
        let mut lines = vec![
            Line::from("Confirm model policy update?"),
            Line::from(""),
            Line::from("Command:"),
            Line::from(format!("  {}", cmd.display_command())),
            Line::from(""),
            Line::from(format!("Dry Run: {}", self.model_policy_state.dry_run)),
            Line::from(format!(
                "Refresh integrity manifest after apply: {}",
                self.model_policy_state.refresh_integrity
            )),
        ];
        if matches!(self.model_policy_state.scope, ModelScope::All)
            && !self.model_policy_state.dry_run
        {
            lines.push(Line::from(""));
            lines.push(Line::from(
                "WARNING: scope is ALL agents — this rewrites every capable harness file.",
            ));
        }
        lines.push(Line::from(""));
        lines.push(Line::from("[Enter to execute, Esc to cancel]"));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Confirm Model Policy")
                    .border_style(theme.border_style()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn render_model_policy_output(
        &self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let title = if self.model_policy_state.dry_run {
            "Model Policy Output (Dry-Run)"
        } else {
            "Model Policy Output"
        };
        output::render_output_scrolled(
            &self.subprocess_output,
            title,
            area,
            frame,
            self.nav.detail_scroll,
            theme,
        );
    }

    fn render_integrity_view(
        &mut self,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let items: Vec<String> = match &self.catalog.integrity {
            Some(integrity) => {
                let total_files: usize =
                    integrity.trees.iter().map(|t| t.files.len()).sum::<usize>()
                        + integrity.root_files.len();
                let mut entries = vec![format!(
                    "Manifest v{} | {} | {} files | SHA: {}...",
                    integrity.manifest_version,
                    integrity.algorithm,
                    total_files,
                    &integrity.aggregate_sha256[..8.min(integrity.aggregate_sha256.len())]
                )];
                for t in &integrity.trees {
                    entries.push(format!(
                        "  [tree] {} ({} files, SHA: {}...)",
                        t.tree,
                        t.files.len(),
                        &t.aggregate_sha256[..8.min(t.aggregate_sha256.len())]
                    ));
                }
                if !integrity.root_files.is_empty() {
                    entries.push(format!(
                        "  [root] root_files ({} files)",
                        integrity.root_files.len()
                    ));
                }
                entries
            }
            None => vec!["No integrity data available".to_string()],
        };
        list_view::render_list_view(
            &items,
            &mut self.nav.list_state,
            "Asset Integrity",
            area,
            frame,
            theme,
        );
    }

    fn render_integrity_detail(
        &self,
        tree_name: &str,
        area: ratatui::layout::Rect,
        frame: &mut Frame,
        theme: &Theme,
    ) {
        let lines: Vec<ratatui::text::Line> = if let Some(integrity) = &self.catalog.integrity {
            if tree_name == "__root_files__" {
                // Requirement 21.5: Display root files with path, SHA-256, and size
                let mut l = vec![
                    ratatui::text::Line::from("Root Files"),
                    ratatui::text::Line::from(format!("Files: {}", integrity.root_files.len())),
                    ratatui::text::Line::from(""),
                ];
                for f in &integrity.root_files {
                    l.push(ratatui::text::Line::from(format!("  {}", f.path)));
                    l.push(ratatui::text::Line::from(format!(
                        "    SHA-256: {}",
                        f.sha256
                    )));
                    l.push(ratatui::text::Line::from(format!(
                        "    Size: {} bytes",
                        f.bytes
                    )));
                }
                l
            } else if let Some(tree) = integrity.trees.iter().find(|t| t.tree == tree_name) {
                // Requirement 21.3, 21.4: Display tree files with path, SHA-256, size,
                // and the parent tree's aggregate SHA-256 hash
                let mut l = vec![
                    ratatui::text::Line::from(format!("Tree: {}", tree.tree)),
                    ratatui::text::Line::from(format!(
                        "Aggregate SHA-256: {}",
                        tree.aggregate_sha256
                    )),
                    ratatui::text::Line::from(format!("Files: {}", tree.files.len())),
                    ratatui::text::Line::from(""),
                ];
                for f in &tree.files {
                    l.push(ratatui::text::Line::from(format!("  {}", f.path)));
                    l.push(ratatui::text::Line::from(format!(
                        "    SHA-256: {}",
                        f.sha256
                    )));
                    l.push(ratatui::text::Line::from(format!(
                        "    Size: {} bytes",
                        f.bytes
                    )));
                }
                l
            } else {
                vec![ratatui::text::Line::from("Tree not found")]
            }
        } else {
            vec![ratatui::text::Line::from("No integrity data")]
        };
        let paragraph = ratatui::widgets::Paragraph::new(lines)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Integrity Detail")
                    .border_style(theme.border_style()),
            )
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((self.nav.detail_scroll, 0));
        frame.render_widget(paragraph, area);
    }

    /// Update filtered indices based on current view and search query.
    fn update_filtered(&mut self) {
        if self.nav.current_view == View::AgentList {
            self.filtered_indices = self.search_engine.search_agents(
                &self.search_query,
                &self.catalog.agents,
                self.provider_filter.as_deref(),
                self.harness_filter.as_deref(),
            );
        }
    }

    /// Cycle through provider filter values (None -> first provider -> second -> ... -> None).
    fn cycle_provider_filter(&mut self) {
        let providers = self.catalog.provider_names();
        if providers.is_empty() {
            return;
        }
        self.provider_filter = match &self.provider_filter {
            None => Some(providers[0].clone()),
            Some(current) => {
                let idx = providers.iter().position(|p| p == current);
                match idx {
                    Some(i) if i + 1 < providers.len() => Some(providers[i + 1].clone()),
                    _ => None,
                }
            }
        };
        self.update_filtered();
    }

    /// Cycle through harness filter values (None -> first harness -> second -> ... -> None).
    fn cycle_harness_filter(&mut self) {
        let harnesses = self.catalog.harness_names();
        if harnesses.is_empty() {
            return;
        }
        self.harness_filter = match &self.harness_filter {
            None => Some(harnesses[0].clone()),
            Some(current) => {
                let idx = harnesses.iter().position(|h| h == current);
                match idx {
                    Some(i) if i + 1 < harnesses.len() => Some(harnesses[i + 1].clone()),
                    _ => None,
                }
            }
        };
        self.update_filtered();
    }

    /// Render filter chips showing active filters.
    fn render_filter_chips(&self, area: ratatui::layout::Rect, frame: &mut Frame, theme: &Theme) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let mut spans: Vec<Span> = Vec::new();
        if let Some(ref pf) = self.provider_filter {
            spans.push(Span::styled(
                format!(" [provider:{pf}] "),
                theme.filter_chip(),
            ));
            spans.push(Span::raw(" "));
        }
        if let Some(ref hf) = self.harness_filter {
            spans.push(Span::styled(
                format!(" [harness:{hf}] "),
                theme.filter_chip(),
            ));
            spans.push(Span::raw(" "));
        }
        if !self.search_query.is_empty() {
            spans.push(Span::styled(
                format!(" [query:\"{}\"] ", self.search_query),
                theme.filter_chip(),
            ));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(vec![line]);
        frame.render_widget(paragraph, area);
    }

    /// Render the full-screen help overlay showing all keybindings.
    fn render_help_overlay(&self, frame: &mut Frame, theme: &Theme) {
        use ratatui::layout::Rect;
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

        let area = frame.area();
        // Center the overlay with some padding
        let overlay = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };

        frame.render_widget(Clear, overlay);

        let lines = vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                theme.help_overlay_title(),
            )),
            Line::from(""),
            Line::from(Span::styled("Navigation", theme.help_overlay_section())),
            Line::from("  j / ↓         Move down"),
            Line::from("  k / ↑         Move up"),
            Line::from("  g             Jump to top"),
            Line::from("  G             Jump to bottom"),
            Line::from("  Enter         Select / drill in"),
            Line::from("  Esc           Back / clear filters"),
            Line::from("  Tab           Next section"),
            Line::from("  Shift+Tab     Previous section"),
            Line::from(""),
            Line::from(Span::styled(
                "Search & Filter",
                theme.help_overlay_section(),
            )),
            Line::from("  /             Activate search"),
            Line::from("  p             Cycle provider filter (agent list)"),
            Line::from("  h             Cycle harness filter (agent list)"),
            Line::from(""),
            Line::from(Span::styled("Model Policy", theme.help_overlay_section())),
            Line::from("  m             Assign models/reasoning (scope from current view)"),
            Line::from("  j/k           Move between fields"),
            Line::from("  Space/Enter   Cycle or toggle the focused field"),
            Line::from(""),
            Line::from(Span::styled("General", theme.help_overlay_section())),
            Line::from("  ?             Toggle this help overlay"),
            Line::from("  t             Toggle light / dark theme"),
            Line::from("  q             Quit"),
            Line::from("  Ctrl+C        Quit"),
            Line::from(""),
            Line::from(Span::styled("Export Builder", theme.help_overlay_section())),
            Line::from("  j/k           Move between fields"),
            Line::from("  Enter         Confirm export"),
            Line::from("  Esc           Cancel"),
            Line::from(""),
            Line::from(Span::styled("Press ? or Esc to close", theme.help_bar())),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .border_style(theme.help_overlay_title()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, overlay);
    }

    /// Update tab completion suggestions based on current export builder field.
    pub fn update_completion_suggestions(&mut self) {
        self.completion_suggestions.clear();
        self.completion_index = 0;

        if self.nav.current_view != View::ExportBuilder {
            return;
        }

        match self.export_state.focused_field {
            0 => {
                // Platform field
                let platforms = self.catalog.platform_names();
                self.completion_suggestions = platforms
                    .iter()
                    .filter(|p| p.starts_with(&self.export_state.platform))
                    .map(|p| p.to_string())
                    .collect();
            }
            1 => {
                // Selection field — show roles
                let roles = self.catalog.role_ids();
                self.completion_suggestions = roles;
            }
            _ => {}
        }
    }

    /// Get the length of the current list for navigation.
    fn current_list_len(&self) -> usize {
        match &self.nav.current_view {
            View::AgentList => self.filtered_indices.len(),
            View::SkillList => self.catalog.skills.len(),
            View::RoleList => self.catalog.roles.len(),
            View::ProviderList => self.get_provider_list().len(),
            View::McpList => self.catalog.mcp_refs.len(),
            View::RuleList => self.catalog.rules.len(),
            View::WorkflowList => self.workflows().len(),
            View::ValidationList => self.validation_gates.len() + 1, // +1 for "Run All"
            View::ProviderAgents(p) => self.catalog.agents_by_provider(p).len(),
            View::IntegrityOverview => self
                .catalog
                .integrity
                .as_ref()
                .map(|i| {
                    // 1 summary + trees + optional root_files entry
                    1 + i.trees.len() + if i.root_files.is_empty() { 0 } else { 1 }
                })
                .unwrap_or(0),
            _ => 0,
        }
    }

    fn get_counts(&self) -> (usize, usize) {
        match &self.nav.current_view {
            View::AgentList => (self.filtered_indices.len(), self.catalog.agents.len()),
            View::SkillList => (self.catalog.skills.len(), self.catalog.skills.len()),
            View::McpList => (self.catalog.mcp_refs.len(), self.catalog.mcp_refs.len()),
            View::RuleList => (self.catalog.rules.len(), self.catalog.rules.len()),
            View::WorkflowList => (self.workflows().len(), self.workflows().len()),
            _ => (0, 0),
        }
    }

    /// Get provider list with counts.
    pub fn get_provider_list(&self) -> Vec<(String, usize)> {
        let mut providers: HashSet<String> = HashSet::new();
        for agent in &self.catalog.agents {
            let p = serde_json::to_value(agent.provider)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", agent.provider));
            providers.insert(p);
        }
        let mut list: Vec<(String, usize)> = providers
            .into_iter()
            .map(|p| {
                let count = self.catalog.agents_by_provider(&p).len();
                (p, count)
            })
            .collect();
        list.sort_by_key(|a| a.0.clone());
        list
    }

    /// Get role list with labels and agent counts.
    pub fn get_role_list(&self) -> Vec<(String, String, usize)> {
        let mut list: Vec<(String, String, usize)> = self
            .catalog
            .roles
            .iter()
            .map(|(id, role)| (id.clone(), role.label.clone(), role.agents.len()))
            .collect();
        list.sort_by_key(|a| a.0.clone());
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::path::Path;

    fn workspace_root() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn make_app() -> App {
        let ws = workspace_root();
        let catalog = CatalogStore::load(&ws);
        App::new(catalog, ws, Uuid::new_v4(), true)
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn app_new_starts_at_agent_list() {
        let app = make_app();
        assert_eq!(app.nav.current_view, View::AgentList);
        assert!(!app.should_quit);
        assert!(!app.filtered_indices.is_empty());
    }

    #[test]
    fn app_quit_on_q() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn app_quit_on_ctrl_c() {
        let mut app = make_app();
        let key = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key);
        assert!(app.should_quit);
    }

    #[test]
    fn app_t_toggles_theme_mode() {
        // Req 35.6 / Task 15.3: `t` toggles the session theme mode and marks
        // the UI dirty so it re-renders with the new palette.
        let mut app = make_app();
        assert_eq!(app.theme_mode, ThemeMode::Dark);
        app.dirty = false;

        app.handle_key_event(key_event(KeyCode::Char('t')));
        assert_eq!(app.theme_mode, ThemeMode::Light);
        assert!(app.dirty, "theme toggle must set dirty flag");

        app.handle_key_event(key_event(KeyCode::Char('t')));
        assert_eq!(app.theme_mode, ThemeMode::Dark);
    }

    #[test]
    fn app_t_does_not_toggle_theme_in_search_mode() {
        // In search mode, `t` is literal search input, not a theme toggle.
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('/')));
        assert!(app.search_active);
        app.handle_key_event(key_event(KeyCode::Char('t')));
        assert_eq!(app.theme_mode, ThemeMode::Dark);
        assert_eq!(app.search_query, "t");
    }

    #[test]
    fn app_tab_advances_current_tab() {
        // v2: Tab key cycles the operator-console tab bar, not the sidebar.
        let mut app = make_app();
        let initial_tab = app.nav.current_tab.clone();
        app.handle_key_event(key_event(KeyCode::Tab));
        // current_tab must have advanced by one step.
        assert_ne!(
            app.nav.current_tab, initial_tab,
            "Tab should advance the active operator-console tab"
        );
        assert!(app.dirty, "Tab must set dirty flag");
    }

    #[test]
    fn sync_view_to_tab_aligns_legacy_view() {
        use crate::ui::nav::Tab;
        let mut app = make_app();
        // Arriving at the Gates tab from a non-catalog view must retarget key
        // dispatch to the validation list (not whatever legacy view was active).
        app.nav.current_view = View::AgentList;
        app.nav.current_tab = Tab::ValidationGates;
        app.sync_view_to_tab();
        assert_eq!(app.nav.current_view, View::ValidationList);

        // CatalogBrowser lands on a catalog list view.
        app.nav.current_view = View::ValidationList;
        app.nav.current_tab = Tab::CatalogBrowser;
        app.sync_view_to_tab();
        assert_eq!(app.nav.current_view, View::AgentList);
    }

    #[test]
    fn app_backtab_retreats_current_tab() {
        // v2: BackTab cycles the operator-console tab bar backwards.
        let mut app = make_app();
        // Advance to tab 1 first so we can retreat back.
        app.nav.next_tab();
        let tab_at_1 = app.nav.current_tab.clone();
        let key = KeyEvent {
            code: KeyCode::BackTab,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key);
        assert_ne!(
            app.nav.current_tab, tab_at_1,
            "BackTab should retreat the active operator-console tab"
        );
        assert!(app.dirty, "BackTab must set dirty flag");
    }

    #[test]
    fn app_j_navigates_down() {
        let mut app = make_app();
        assert_eq!(app.nav.selected_index(), 0);
        app.handle_key_event(key_event(KeyCode::Char('j')));
        assert_eq!(app.nav.selected_index(), 1);
    }

    #[test]
    fn app_k_navigates_up() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('j')));
        app.handle_key_event(key_event(KeyCode::Char('k')));
        assert_eq!(app.nav.selected_index(), 0);
    }

    #[test]
    fn app_enter_drills_into_agent_detail() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Enter));
        assert!(matches!(app.nav.current_view, View::AgentDetail(_)));
    }

    #[test]
    fn app_escape_goes_back() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Enter));
        assert!(matches!(app.nav.current_view, View::AgentDetail(_)));
        app.handle_key_event(key_event(KeyCode::Esc));
        assert_eq!(app.nav.current_view, View::AgentList);
    }

    #[test]
    fn app_search_activation() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('/')));
        assert!(app.search_active);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn app_search_typing() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('/')));
        app.handle_key_event(key_event(KeyCode::Char('a')));
        app.handle_key_event(key_event(KeyCode::Char('w')));
        app.handle_key_event(key_event(KeyCode::Char('s')));
        assert_eq!(app.search_query, "aws");
    }

    #[test]
    fn app_search_backspace() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('/')));
        app.handle_key_event(key_event(KeyCode::Char('x')));
        app.handle_key_event(key_event(KeyCode::Backspace));
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn app_search_escape_deactivates() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('/')));
        app.handle_key_event(key_event(KeyCode::Esc));
        assert!(!app.search_active);
    }

    #[test]
    fn app_g_goes_to_top() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('j')));
        app.handle_key_event(key_event(KeyCode::Char('j')));
        app.handle_key_event(key_event(KeyCode::Char('g')));
        assert_eq!(app.nav.selected_index(), 0);
    }

    #[test]
    fn app_big_g_goes_to_bottom() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('G')));
        let max = app.current_list_len();
        assert_eq!(app.nav.selected_index(), max - 1);
    }

    #[test]
    fn app_empty_catalog_does_not_panic() {
        let catalog = CatalogStore {
            agents: Vec::new(),
            skills: Vec::new(),
            roles: std::collections::HashMap::new(),
            role_catalog_version: String::new(),
            role_catalog_description: String::new(),
            mcp_refs: Vec::new(),
            rules: Vec::new(),
            integrity: None,
            model_assignments: None,
            model_registry: None,
            workflows: None,
            load_errors: Vec::new(),
            content_hashes: std::collections::HashMap::new(),
            catalog_root: std::path::PathBuf::from("."),
        };
        let app = App::new(catalog, PathBuf::from("/tmp"), Uuid::new_v4(), true);
        assert!(app.filtered_indices.is_empty());
        assert!(app.provider_filter.is_none());
        assert!(app.harness_filter.is_none());
        assert!(app.running_gate.is_none());
        assert!(app.running_gate_start.is_none());
        assert!(!app.show_help_overlay);
    }

    #[test]
    fn app_tick_clears_old_messages() {
        let mut app = make_app();
        app.status_message = Some((
            "test".to_string(),
            Instant::now() - std::time::Duration::from_secs(20),
        ));
        app.tick();
        assert!(app.status_message.is_none());
    }

    #[test]
    fn app_tick_keeps_recent_messages() {
        let mut app = make_app();
        app.status_message = Some(("test".to_string(), Instant::now()));
        app.tick();
        assert!(app.status_message.is_some());
    }

    #[test]
    fn app_get_provider_list_non_empty() {
        let app = make_app();
        let providers = app.get_provider_list();
        assert!(!providers.is_empty());
    }

    #[test]
    fn app_get_role_list_non_empty() {
        let app = make_app();
        let roles = app.get_role_list();
        assert!(!roles.is_empty());
    }

    #[test]
    fn app_subprocess_output_capped_at_max() {
        let mut app = make_app();
        // Fill beyond the limit
        for i in 0..(MAX_SUBPROCESS_OUTPUT_LINES + 500) {
            app.subprocess_output.push(output::OutputLine {
                content: format!("line {i}"),
                stream: crate::subprocess::OutputStream::Stdout,
            });
        }
        app.tick();
        assert_eq!(app.subprocess_output.len(), MAX_SUBPROCESS_OUTPUT_LINES);
        // Verify oldest lines were removed (first line should be "line 500")
        assert_eq!(app.subprocess_output[0].content, "line 500");
    }

    #[test]
    fn app_search_query_length_capped() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('/')));
        // Type more than MAX_SEARCH_QUERY_LEN characters
        for _ in 0..(MAX_SEARCH_QUERY_LEN + 50) {
            app.handle_key_event(key_event(KeyCode::Char('x')));
        }
        assert_eq!(app.search_query.len(), MAX_SEARCH_QUERY_LEN);
    }

    #[test]
    fn export_builder_defaults_to_dry_run() {
        let app = make_app();
        assert!(app.export_state.dry_run);
    }

    #[test]
    fn export_builder_field_navigation() {
        let mut app = make_app();
        // Navigate to Export section
        app.nav.set_sidebar_index(7); // Export is index 7
        assert_eq!(app.nav.current_view, View::ExportBuilder);
        assert_eq!(app.export_state.focused_field, 0);

        // Navigate down
        app.handle_key_event(key_event(KeyCode::Char('j')));
        assert_eq!(app.export_state.focused_field, 1);

        app.handle_key_event(key_event(KeyCode::Char('j')));
        assert_eq!(app.export_state.focused_field, 2);

        // Navigate up
        app.handle_key_event(key_event(KeyCode::Char('k')));
        assert_eq!(app.export_state.focused_field, 1);
    }

    #[test]
    fn export_builder_toggle_dry_run() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        // Navigate to dry_run field (index 3)
        app.export_state.focused_field = 3;
        assert!(app.export_state.dry_run);

        // Toggle with space
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert!(!app.export_state.dry_run);

        // Toggle back
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert!(app.export_state.dry_run);
    }

    #[test]
    fn export_builder_toggle_force() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 4;
        assert!(!app.export_state.force);

        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert!(app.export_state.force);
    }

    #[test]
    fn export_builder_toggle_no_skills() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 5;
        assert!(!app.export_state.no_skills);

        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert!(app.export_state.no_skills);
    }

    #[test]
    fn export_builder_type_platform() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 0;
        app.export_state.platform.clear();

        app.handle_key_event(key_event(KeyCode::Char('c')));
        app.handle_key_event(key_event(KeyCode::Char('u')));
        app.handle_key_event(key_event(KeyCode::Char('r')));
        assert_eq!(app.export_state.platform, "cur");

        app.handle_key_event(key_event(KeyCode::Backspace));
        assert_eq!(app.export_state.platform, "cu");
    }

    #[test]
    fn export_builder_type_target_repo() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 2;

        app.handle_key_event(key_event(KeyCode::Char('/')));
        app.handle_key_event(key_event(KeyCode::Char('t')));
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.handle_key_event(key_event(KeyCode::Char('p')));
        assert_eq!(app.export_state.target_repo, "/tmp");
    }

    #[test]
    fn export_builder_cycle_selection() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 1;

        // Start at All
        assert_eq!(app.export_state.selection, ExportSelection::All);

        // Enter cycles to Role
        app.handle_key_event(key_event(KeyCode::Enter));
        assert!(matches!(
            app.export_state.selection,
            ExportSelection::Role(_)
        ));
    }

    #[test]
    fn export_builder_enter_validates_empty_platform() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 0;
        app.export_state.platform.clear();
        app.export_state.target_repo = "/tmp".to_string();

        app.handle_key_event(key_event(KeyCode::Enter));
        // Should show error about platform being required
        assert!(app.status_message.is_some());
        assert!(app.status_message.as_ref().unwrap().0.contains("Platform"));
    }

    #[test]
    fn export_builder_enter_validates_empty_target() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 0;
        app.export_state.platform = "kiro".to_string();
        app.export_state.target_repo.clear();

        app.handle_key_event(key_event(KeyCode::Enter));
        // Should show error about target repo being required
        assert!(app.status_message.is_some());
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .0
            .contains("Target repo"));
    }

    #[test]
    fn export_builder_enter_validates_nonexistent_path() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 0;
        app.export_state.platform = "kiro".to_string();
        app.export_state.target_repo = "/nonexistent/path/xyz123".to_string();

        app.handle_key_event(key_event(KeyCode::Enter));
        // Should show error about path not existing
        assert!(app.status_message.is_some());
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .0
            .contains("does not exist"));
    }

    #[test]
    fn export_builder_enter_navigates_to_confirm_with_valid_path() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 0;
        app.export_state.platform = "kiro".to_string();
        app.export_state.target_repo = "/tmp".to_string();

        app.handle_key_event(key_event(KeyCode::Enter));
        // Should navigate to ExportConfirm
        assert_eq!(app.nav.current_view, View::ExportConfirm);
    }

    #[test]
    fn export_confirm_esc_goes_back_to_builder() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.platform = "kiro".to_string();
        app.export_state.target_repo = "/tmp".to_string();
        app.export_state.focused_field = 0;
        app.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(app.nav.current_view, View::ExportConfirm);

        // Esc should go back to builder, preserving selections
        app.handle_key_event(key_event(KeyCode::Esc));
        assert_eq!(app.nav.current_view, View::ExportBuilder);
        assert_eq!(app.export_state.platform, "kiro");
        assert_eq!(app.export_state.target_repo, "/tmp");
    }

    #[test]
    fn export_builder_esc_goes_back() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        assert_eq!(app.nav.current_view, View::ExportBuilder);

        app.handle_key_event(key_event(KeyCode::Esc));
        // Should quit since there's no history
        assert!(app.should_quit);
    }

    #[test]
    fn export_builder_field_stops_at_boundary() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7);
        app.export_state.focused_field = 5; // last field

        app.handle_key_event(key_event(KeyCode::Char('j')));
        assert_eq!(app.export_state.focused_field, 5); // stays at boundary

        app.export_state.focused_field = 0;
        app.handle_key_event(key_event(KeyCode::Char('k')));
        assert_eq!(app.export_state.focused_field, 0); // stays at 0
    }

    #[test]
    fn m_opens_model_policy_builder_with_agent_scope_from_list() {
        let mut app = make_app();
        assert_eq!(app.nav.current_view, View::AgentList);
        app.handle_key_event(key_event(KeyCode::Char('m')));
        assert_eq!(app.nav.current_view, View::ModelPolicyBuilder);
        // The highlighted agent (first in the filtered list) becomes the scope.
        match &app.model_policy_state.scope {
            crate::models::model_policy::ModelScope::Agent(id) => {
                assert!(!id.is_empty());
            }
            other => panic!("expected agent scope, got {other:?}"),
        }
        // Safe defaults: dry-run on, integrity refresh on.
        assert!(app.model_policy_state.dry_run);
        assert!(app.model_policy_state.refresh_integrity);
    }

    #[test]
    fn m_refused_while_gate_running() {
        let mut app = make_app();
        assert_eq!(app.nav.current_view, View::AgentList);
        // Simulate a validation gate in flight.
        app.running_gate = Some("validate".to_string());
        app.handle_key_event(key_event(KeyCode::Char('m')));
        // The builder must not open over a running gate.
        assert_eq!(app.nav.current_view, View::AgentList);
        assert!(app.status_message.is_some());
    }

    #[test]
    fn m_refused_while_subprocess_pending() {
        let mut app = make_app();
        let (_tx, rx) = tokio::sync::oneshot::channel();
        app.pending_subprocess = Some(rx);
        app.handle_key_event(key_event(KeyCode::Char('m')));
        assert_eq!(app.nav.current_view, View::AgentList);
        assert!(app.subprocess_busy());
    }

    #[test]
    fn execute_model_policy_refused_while_busy() {
        let mut app = make_app();
        // Open the builder cleanly, then a gate starts before the operator
        // confirms — execution must refuse rather than clobber the gate handle.
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.model = "auto".to_string();
        app.model_policy_state.focused_field = App::MODEL_POLICY_FIELD_COUNT - 1;
        app.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(app.nav.current_view, View::ModelPolicyConfirm);
        app.running_gate = Some("validate".to_string());
        app.handle_key_event(key_event(KeyCode::Enter));
        // Still on confirm; no model-policy stage was started.
        assert_eq!(app.nav.current_view, View::ModelPolicyConfirm);
        assert!(app.model_policy_stage.is_none());
    }

    // -----------------------------------------------------------------------
    // SEC-6 regression tests: subprocess_busy() must gate every launch path
    // (not just execute_model_policy) and every cancel/quit branch in the
    // model-policy output view (not just subprocess_handle.is_some()).
    // -----------------------------------------------------------------------

    #[test]
    fn execute_export_refused_while_model_policy_stage_pending() {
        let mut app = make_app();
        app.nav.set_sidebar_index(7); // Export
        app.export_state.platform = "kiro".to_string();
        app.export_state.target_repo = "/tmp".to_string();
        app.export_state.focused_field = 0;
        app.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(app.nav.current_view, View::ExportConfirm);

        // A model-policy operation is mid-flight (the pending_subprocess
        // handoff window, or the subprocess itself) when the operator tries
        // to fire off an export from a different tab.
        app.model_policy_stage = Some(ModelPolicyStage::Apply);
        app.handle_key_event(key_event(KeyCode::Enter));

        // Still on confirm; no export subprocess was spawned to clobber the
        // in-flight model-policy operation's pending_subprocess/subprocess_handle.
        assert_eq!(app.nav.current_view, View::ExportConfirm);
        assert!(app.pending_subprocess.is_none());
        assert!(app.status_message.is_some());
    }

    #[test]
    fn validation_gate_refused_while_pending_subprocess_set() {
        let mut app = make_app();
        let (_tx, rx) = tokio::sync::oneshot::channel();
        app.pending_subprocess = Some(rx);
        app.nav.set_sidebar_index(6); // Validation
        assert_eq!(app.nav.current_view, View::ValidationList);

        // "Run All Validations" — running_gate is still None at this point
        // (only pending_subprocess/model_policy_stage signal busy), so the
        // pre-existing `running_gate.is_some()` check alone would have let
        // this through and clobbered the pending handle.
        app.handle_validation_enter(0);

        assert!(app.running_gate.is_none());
        assert!(app.status_message.is_some());
        // The original pending_subprocess must not have been overwritten by
        // a second spawn.
        assert!(app.pending_subprocess.is_some());
    }

    #[test]
    fn model_policy_output_esc_and_q_refuse_while_busy() {
        let mut app = make_app();
        app.nav.push_view(View::ModelPolicyOutput);
        let (_tx, rx) = tokio::sync::oneshot::channel();
        app.pending_subprocess = Some(rx);
        app.model_policy_stage = Some(ModelPolicyStage::Apply);

        app.handle_key_event(key_event(KeyCode::Esc));
        // Esc must not silently pop away from a pending policy/integrity
        // write — popping would reach a view (e.g. ModelPolicyConfirm)
        // whose own 'q' handler quits unconditionally, dropping the
        // eventual SubprocessHandle (kill_on_drop) mid-write.
        assert_eq!(app.nav.current_view, View::ModelPolicyOutput);
        assert!(app.pending_subprocess.is_some());
        assert!(app.model_policy_stage.is_some());

        // 'q' must likewise refuse to quit while busy, instead of relying on
        // the stale `subprocess_handle.is_none()` check that was true during
        // this exact pending window.
        app.handle_key_event(key_event(KeyCode::Char('q')));
        assert!(!app.should_quit);
        assert!(app.pending_subprocess.is_some());
        assert!(app.model_policy_stage.is_some());
    }

    #[test]
    fn m_prefills_provider_scope_from_provider_agents_view() {
        let mut app = make_app();
        app.nav.push_view(View::ProviderAgents("gcp".to_string()));
        app.handle_key_event(key_event(KeyCode::Char('m')));
        assert_eq!(app.nav.current_view, View::ModelPolicyBuilder);
        assert_eq!(
            app.model_policy_state.scope,
            crate::models::model_policy::ModelScope::Provider("gcp".to_string())
        );
    }

    #[test]
    fn m_prefills_role_scope_from_role_detail_view() {
        let mut app = make_app();
        app.nav
            .push_view(View::RoleDetail("cloud-security-engineer".to_string()));
        app.handle_key_event(key_event(KeyCode::Char('m')));
        assert_eq!(
            app.model_policy_state.scope,
            crate::models::model_policy::ModelScope::Role("cloud-security-engineer".to_string())
        );
    }

    #[test]
    fn model_policy_harness_cycle_renarrows_reasoning_per_registry() {
        // Supersedes an earlier test that asserted reasoning was codex-only.
        // That gate was wrong: HARNESS_CAPABILITIES in scripts/model-policy.mjs
        // gives claude-code reasoning_effort: true (projected as the subagent
        // `effort:` key), so the builder must offer it there too. Availability
        // now comes from catalog/model-registry.json rather than a hardcoded
        // harness name.
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        assert_eq!(app.model_policy_state.harness(), "codex");
        assert!(app.model_policy_state.reasoning_supported());

        app.model_policy_state.focused_field = 3;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_ne!(app.model_policy_state.reasoning_index, 0);

        // claude-code projects `effort:` — reasoning stays available.
        app.model_policy_state.focused_field = 1;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.harness(), "claude-code");
        assert!(
            app.model_policy_state.reasoning_supported(),
            "claude-code supports the effort field and must offer it"
        );

        // cursor has no reasoning field at all — the cycle collapses and the
        // selection falls back to "(unchanged)".
        app.model_policy_state.focused_field = 1;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.harness(), "cursor");
        assert!(!app.model_policy_state.reasoning_supported());
        assert_eq!(app.model_policy_state.reasoning_index, 0);

        // Pressing Space on the collapsed field explains itself rather than
        // silently doing nothing.
        app.model_policy_state.focused_field = 3;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.reasoning_index, 0);
        assert!(app.status_message.is_some());
    }

    #[test]
    fn model_policy_space_cycles_registry_verified_models() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        assert!(
            !app.model_policy_state.model_choices.is_empty(),
            "the committed registry should populate the codex picker"
        );
        assert!(app.model_policy_state.model.is_empty());

        app.model_policy_state.focused_field = 2;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        let first = app.model_policy_state.model.clone();
        assert!(!first.is_empty());
        assert_eq!(
            app.model_policy_state
                .selected_choice()
                .map(|c| c.model.clone()),
            Some(first.clone())
        );

        // Every picked value must be one the policy engine would accept.
        app.model_policy_state.focused_field = App::MODEL_POLICY_FIELD_COUNT - 1;
        app.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(app.nav.current_view, View::ModelPolicyConfirm);
    }

    #[test]
    fn model_policy_typing_a_model_clears_the_picked_entry() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.focused_field = 2;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert!(app.model_policy_state.selected_choice().is_some());

        // A free-typed tag (open Ollama namespace) is still allowed, but it is
        // no longer a picked registry entry.
        app.handle_key_event(key_event(KeyCode::Char('x')));
        assert!(app.model_policy_state.selected_choice().is_none());
        assert!(app.model_policy_state.model.ends_with('x'));
    }

    #[test]
    fn model_policy_reasoning_narrows_to_the_selected_model() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        // claude-code: an Opus model takes an effort, Haiku 4.5 does not
        // (the models overview lists effort as unsupported for it).
        app.model_policy_state.focused_field = 1;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.harness(), "claude-code");

        app.model_policy_state.model = "claude-opus-5".to_string();
        app.refresh_model_policy_choices();
        assert!(app.model_policy_state.reasoning_supported());

        app.model_policy_state.model = "claude-haiku-4-5".to_string();
        app.refresh_model_policy_choices();
        assert!(
            app.model_policy_state.reasoning_clear_only(),
            "claude-haiku-4-5 declares an empty reasoning_efforts list, so only auto remains"
        );
        assert_eq!(
            app.model_policy_state.reasoning_cycle,
            vec!["(unchanged)".to_string(), "auto".to_string()]
        );
    }

    #[test]
    fn model_policy_keeps_auto_for_models_without_effort_support() {
        // scripts/model-policy.mjs rejects an inherited effort on such a model
        // with "set reasoning to auto" — so auto has to stay reachable, or the
        // builder cannot perform the engine's own remedy. The floating `haiku`
        // alias is gated like the pinned ids and must behave the same way.
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.focused_field = 1;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.harness(), "claude-code");

        for model in ["claude-haiku-4-5", "haiku"] {
            app.model_policy_state.model = model.to_string();
            app.refresh_model_policy_choices();
            assert!(
                app.model_policy_state
                    .reasoning_cycle
                    .iter()
                    .any(|e| e == "auto"),
                "{model} must still offer auto to clear an inherited effort"
            );
            assert!(
                app.model_policy_state.reasoning_supported(),
                "{model} must remain cyclable so auto can be selected"
            );
        }

        // Selecting auto emits --reasoning auto, which is what clears the field.
        app.model_policy_state.reasoning_index = 1;
        let cmd = app.model_policy_state.command();
        assert_eq!(cmd.reasoning.as_deref(), Some("auto"));
    }

    #[test]
    fn model_policy_harness_change_drops_a_model_the_new_harness_lacks() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.model = "gpt-6-astra".to_string();
        app.refresh_model_policy_choices();
        assert!(app.model_policy_state.selected_choice().is_some());

        // codex -> claude-code: gpt-6-astra is not offered there, so carrying
        // the text over would hand the script a value it must reject.
        app.model_policy_state.focused_field = 1;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.harness(), "claude-code");
        assert!(
            app.model_policy_state.model.is_empty(),
            "a picked model absent from the new harness must be cleared, got {:?}",
            app.model_policy_state.model
        );
    }

    #[test]
    fn model_policy_harness_change_keeps_free_typed_text() {
        // Only *picked* entries are dropped; a value the operator typed is
        // theirs to keep (open namespaces cannot be enumerated).
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.focused_field = 2;
        for c in ["q", "w", "e"] {
            app.handle_key_event(key_event(KeyCode::Char(c.chars().next().unwrap())));
        }
        assert_eq!(app.model_policy_state.model, "qwe");
        app.model_policy_state.focused_field = 1;
        app.handle_key_event(key_event(KeyCode::Char(' ')));
        assert_eq!(app.model_policy_state.model, "qwe");
    }

    #[test]
    fn model_policy_pickers_refresh_after_a_catalog_reload() {
        // The builder caches its choices; a reload must rebuild them or the
        // view keeps offering values the new registry no longer verifies.
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        let before = app.model_policy_state.model_choices.len();
        assert!(before > 0);

        app.model_policy_state.model_choices.clear();
        app.model_policy_state.reasoning_cycle = vec!["(unchanged)".to_string()];
        app.reload_catalog();
        assert_eq!(app.model_policy_state.model_choices.len(), before);
        assert!(app.model_policy_state.reasoning_supported());
    }

    #[test]
    fn model_policy_continue_requires_model_or_reasoning() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.focused_field = App::MODEL_POLICY_FIELD_COUNT - 1;
        app.handle_key_event(key_event(KeyCode::Enter));
        // Nothing set — stays in the builder with a validation message.
        assert_eq!(app.nav.current_view, View::ModelPolicyBuilder);
        assert!(app.status_message.is_some());
    }

    #[test]
    fn model_policy_continue_rejects_unknown_scope_id() {
        let mut app = make_app();
        app.nav.push_view(View::ProviderAgents("gcp".to_string()));
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.scope =
            crate::models::model_policy::ModelScope::Provider("no-such-provider".to_string());
        app.model_policy_state.model = "gpt-5.4".to_string();
        app.model_policy_state.focused_field = App::MODEL_POLICY_FIELD_COUNT - 1;
        app.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(app.nav.current_view, View::ModelPolicyBuilder);
        let (msg, _) = app.status_message.clone().expect("status message");
        assert!(msg.contains("Unknown provider"));
    }

    #[test]
    fn model_policy_continue_navigates_to_confirm() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.model = "auto".to_string();
        app.model_policy_state.focused_field = App::MODEL_POLICY_FIELD_COUNT - 1;
        app.handle_key_event(key_event(KeyCode::Enter));
        assert_eq!(app.nav.current_view, View::ModelPolicyConfirm);
        // Esc returns to the builder preserving state.
        app.handle_key_event(key_event(KeyCode::Esc));
        assert_eq!(app.nav.current_view, View::ModelPolicyBuilder);
        assert_eq!(app.model_policy_state.model, "auto");
    }

    #[test]
    fn model_policy_typing_edits_model_field() {
        let mut app = make_app();
        app.handle_key_event(key_event(KeyCode::Char('m')));
        app.model_policy_state.focused_field = 2;
        for c in "haiku".chars() {
            app.handle_key_event(key_event(KeyCode::Char(c)));
        }
        assert_eq!(app.model_policy_state.model, "haiku");
        app.handle_key_event(key_event(KeyCode::Backspace));
        assert_eq!(app.model_policy_state.model, "haik");
        // j types into the text field instead of navigating.
        app.handle_key_event(key_event(KeyCode::Char('j')));
        assert_eq!(app.model_policy_state.model, "haikj");
        assert_eq!(app.model_policy_state.focused_field, 2);
    }

    #[test]
    fn model_policy_command_builds_expected_args() {
        let mut state = ModelPolicyBuilderState::new();
        state.scope = crate::models::model_policy::ModelScope::Role("cloud-dba".to_string());
        state.model = "gpt-5.5".to_string();
        // No registry available here, so the builder falls back to the
        // pre-registry union; select "high" by name rather than by index.
        state.refresh_choices(None);
        state.reasoning_index = state
            .reasoning_cycle
            .iter()
            .position(|e| e == "high")
            .expect("fallback cycle offers \"high\" for codex");
        state.dry_run = false;
        let cmd = state.command();
        assert!(cmd.validate().is_ok());
        assert_eq!(
            cmd.to_args(),
            vec![
                "set",
                "--scope",
                "role=cloud-dba",
                "--harness",
                "codex",
                "--model",
                "gpt-5.5",
                "--reasoning",
                "high",
            ]
        );
    }

    #[test]
    fn model_assignments_loaded_and_queryable() {
        let app = make_app();
        let assignments = app.catalog.model_assignments.as_ref();
        assert!(
            assignments.is_some(),
            "catalog/model-assignments.json should load"
        );
        // Every codex assignment carries a concrete model (seeded policy).
        let firebase = app
            .catalog
            .model_assignments_for_agent("gcp-firebase-developer-agent");
        assert!(firebase.iter().any(|a| a.harness == "codex"
            && a.model.as_deref() == Some("gpt-5.4")
            && a.reasoning_effort.as_deref() == Some("high")));
    }

    fn assignment_fixture(model_warning: Option<&str>) -> crate::models::ModelAssignment {
        crate::models::ModelAssignment {
            agent_id: "aws-solution-architect-agent".to_string(),
            harness: "codex".to_string(),
            model: Some("gpt-5.5".to_string()),
            model_provider: None,
            model_fallback_from: model_warning.map(|_| "gpt-5-2025-08-07".to_string()),
            model_warning: model_warning.map(|s| s.to_string()),
            reasoning_effort: Some("high".to_string()),
            model_source: "agent:aws-solution-architect-agent".to_string(),
            reasoning_source: "agent:aws-solution-architect-agent".to_string(),
        }
    }

    #[test]
    fn build_model_lines_omits_warning_row_when_absent() {
        let assignment = assignment_fixture(None);
        let lines = build_model_lines(&[&assignment]);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "codex");
        assert!(!lines.iter().any(|(label, _)| label == "warning"));
    }

    #[test]
    fn build_model_lines_emits_warning_row_when_present() {
        let warning = "model \"gpt-5-2025-08-07\" was retired by the provider — projecting documented successor \"gpt-5.5\"; migrate the policy rule";
        let assignment = assignment_fixture(Some(warning));
        let lines = build_model_lines(&[&assignment]);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "codex");
        assert_eq!(lines[1].0, "warning");
        assert_eq!(lines[1].1, format!("warning: {warning}"));
    }
}
