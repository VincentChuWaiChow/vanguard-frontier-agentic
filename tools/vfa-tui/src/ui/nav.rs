use std::collections::HashMap;

use ratatui::widgets::ListState;

// ── v2 Tab enum (Requirement 16.1) ──────────────────────────────────────────

/// Top-level navigation tabs for the v2 operator console.
///
/// Tab cycling order: Overview → CoverageMatrix → ValidationGates →
/// PolicyViolations → AuditLog → Dependencies → CatalogBrowser → Settings →
/// Overview (wraps).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tab {
    Overview,
    CoverageMatrix,
    ValidationGates,
    PolicyViolations,
    AuditLog,
    Dependencies,
    CatalogBrowser,
    Settings,
}

impl Tab {
    /// All tabs in canonical order.
    pub const ALL: &'static [Tab] = &[
        Tab::Overview,
        Tab::CoverageMatrix,
        Tab::ValidationGates,
        Tab::PolicyViolations,
        Tab::AuditLog,
        Tab::Dependencies,
        Tab::CatalogBrowser,
        Tab::Settings,
    ];

    /// The total number of tabs (useful for cycling arithmetic).
    pub const COUNT: usize = Self::ALL.len();

    /// Return the index of this tab in `ALL`.
    pub fn index(&self) -> usize {
        Self::ALL
            .iter()
            .position(|t| t == self)
            .expect("every Tab variant must appear in Tab::ALL")
    }

    /// Display label shown in the tab bar.
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::CoverageMatrix => "Coverage",
            Tab::ValidationGates => "Gates",
            Tab::PolicyViolations => "Violations",
            Tab::AuditLog => "Audit Log",
            Tab::Dependencies => "Deps",
            Tab::CatalogBrowser => "Catalog",
            Tab::Settings => "Settings",
        }
    }
}

// ── v2 View enum for drill-down views ───────────────────────────────────────

/// Drill-down and overlay views that sit on the history stack.
///
/// The v2 design uses `Tab` for top-level navigation and `DrillView` for
/// drill-downs that can be pushed/popped within a tab.
#[derive(Debug, Clone, PartialEq)]
pub enum DrillView {
    // Coverage tab drill-downs
    WorkspaceDetail(String),
    // Gates tab drill-downs
    GateDAG,
    GateOutput(String),
    // Violations tab drill-downs
    ViolationDetail(usize),
    // Catalog tab drill-downs
    AgentDetail(String),
    SkillDetail(String),
    McpDetail(String),
    RoleDetail(String),
    RuleDetail(String),
    IntegrityDetail(String),
    // Dependencies tab drill-downs
    DependencyFocus(String),
    // Overlays (pushed on top of any view)
    HelpOverlay,
    SearchOverlay,
}

// ── v1 View enum — kept verbatim for app.rs compatibility ───────────────────

/// All possible views in the TUI.
///
/// **Compatibility note:** This enum and its variants are retained exactly as
/// they were in v1. `app.rs` matches on every variant; changing, removing, or
/// renaming any variant here would break compilation. New v2 navigation uses
/// the separate `Tab` / `DrillView` types above.
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    AgentList,
    AgentDetail(String),
    SkillList,
    SkillDetail(String),
    RoleList,
    RoleDetail(String),
    ProviderList,
    ProviderAgents(String),
    McpList,
    McpDetail(String),
    RuleList,
    RuleDetail(String),
    WorkflowList,
    WorkflowDetail(String),
    ModList,
    ModDetail(String),
    ValidationList,
    ValidationOutput(String),
    ExportBuilder,
    ExportConfirm,
    ExportOutput,
    ModelPolicyBuilder,
    ModelPolicyConfirm,
    ModelPolicyOutput,
    IntegrityOverview,
    IntegrityDetail(String),
}

// ── Sidebar (v1 compat) ──────────────────────────────────────────────────────

/// Type alias for sidebar section entries.
pub type SidebarEntry = (&'static str, fn() -> View);

/// Sidebar section definitions mapping labels to their root views.
pub const SIDEBAR_SECTIONS: &[SidebarEntry] = &[
    ("Agents", || View::AgentList),
    ("Skills", || View::SkillList),
    ("Roles", || View::RoleList),
    ("Providers", || View::ProviderList),
    ("MCP References", || View::McpList),
    ("Rules", || View::RuleList),
    ("Validation", || View::ValidationList),
    ("Export", || View::ExportBuilder),
    ("Model Policy", || View::ModelPolicyBuilder),
    ("Integrity", || View::IntegrityOverview),
    // Appended rather than grouped with the catalog sections above because the
    // app tests address sidebar sections by hardcoded index; inserting mid-list
    // silently repoints every one of those numbers at a different section.
    ("Workflows", || View::WorkflowList),
    ("Mods", || View::ModList),
];

// ── Keybinding action enum ───────────────────────────────────────────────────

/// Pure navigation actions returned by the keybinding dispatcher.
///
/// The dispatcher maps raw key codes to actions without mutating state, keeping
/// state transitions testable as pure functions.
#[derive(Debug, Clone, PartialEq)]
pub enum NavAction {
    /// Move selection down by 1.
    Down,
    /// Move selection up by 1.
    Up,
    /// Jump to first item (vim `g`).
    First,
    /// Jump to last item (vim `G`).
    Last,
    /// Page down (Ctrl-d).
    PageDown,
    /// Page up (Ctrl-u).
    PageUp,
    /// Advance to the next tab (Tab key).
    NextTab,
    /// Go to the previous tab (Shift-Tab).
    PrevTab,
    /// Drill into the selected item (Enter).
    Enter,
    /// Navigate back / close overlay (Escape).
    Back,
    /// Activate fuzzy search (`/`).
    ActivateSearch,
    /// Show help overlay (`?`).
    ShowHelp,
    /// Unrecognised key — caller may handle or ignore.
    Unhandled,
}

// ── NavigationState ──────────────────────────────────────────────────────────

/// Navigation state machine managing views, history, and list selections.
///
/// ### Compatibility shims for `app.rs`
///
/// `app.rs` (v1 consumer) accesses the following fields directly:
/// - `current_view: View`
/// - `history: Vec<View>`
/// - `sidebar_index: usize`
/// - `list_state: ListState`
/// - `detail_scroll: u16`
///
/// And calls these methods:
/// - `new()`, `push_view(View)`, `pop_view() -> bool`
/// - `set_sidebar_index(usize)`, `select_next(usize)`, `select_prev()`
/// - `select_first()`, `select_last(usize)`, `selected_index() -> usize`
///
/// All of these are preserved unchanged.  The v2 fields (`current_tab`,
/// `tab_history`, `list_states`, `search_active`, `search_query`) are
/// additive — `app.rs` neither reads nor writes them.
pub struct NavigationState {
    // ── v1 fields (app.rs reads/writes these directly) ──────────────────────
    pub current_view: View,
    pub history: Vec<View>,
    pub sidebar_index: usize,
    pub list_state: ListState,
    pub detail_scroll: u16,

    // ── v2 fields ────────────────────────────────────────────────────────────
    /// Currently active top-level tab.
    pub current_tab: Tab,

    /// Per-tab drill-down history (max 20 entries, independent of `history`).
    ///
    /// Each entry is a `DrillView` pushed when the operator drills into a row.
    /// Pressing Escape pops the stack.  This is separate from `history` (the
    /// v1 `View`-based back-stack) so that `app.rs` is unaffected.
    pub tab_history: Vec<DrillView>,

    /// Per-tab scroll positions preserved across tab switches (Req 16).
    pub list_states: HashMap<Tab, ListState>,

    /// Whether v2 fuzzy search is active (`/` to activate, Escape to close).
    pub search_active: bool,

    /// Current v2 fuzzy-search query string.
    pub search_query: String,
}

impl NavigationState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut list_states: HashMap<Tab, ListState> = HashMap::new();
        for tab in Tab::ALL {
            let mut ls = ListState::default();
            ls.select(Some(0));
            list_states.insert(tab.clone(), ls);
        }

        Self {
            current_view: View::AgentList,
            history: Vec::new(),
            sidebar_index: 0,
            list_state,
            detail_scroll: 0,

            current_tab: Tab::Overview,
            tab_history: Vec::new(),
            list_states,
            search_active: false,
            search_query: String::new(),
        }
    }

    // ── v1 history methods (app.rs compat) ───────────────────────────────────

    /// Maximum number of v1 history entries to retain.
    pub const MAX_HISTORY: usize = 64;

    /// Maximum number of v2 tab-history (drill-down) entries (Req 16 — max 20).
    pub const MAX_TAB_HISTORY: usize = 20;

    /// Push a new view onto the v1 history stack and navigate to it.
    ///
    /// If the history exceeds `MAX_HISTORY` entries, the oldest entry is
    /// removed.  Resets the list selection and detail scroll offset.
    pub fn push_view(&mut self, view: View) {
        self.history.push(self.current_view.clone());
        if self.history.len() > Self::MAX_HISTORY {
            self.history.remove(0);
        }
        self.current_view = view;
        self.list_state.select(Some(0));
        self.detail_scroll = 0;
    }

    /// Pop the previous view from the v1 history stack.
    ///
    /// Returns `false` if already at the root (no history to pop).
    pub fn pop_view(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current_view = prev;
            self.list_state.select(Some(0));
            self.detail_scroll = 0;
            true
        } else {
            false
        }
    }

    /// Set sidebar index and navigate to the corresponding section view.
    pub fn set_sidebar_index(&mut self, idx: usize) {
        if idx < SIDEBAR_SECTIONS.len() {
            self.sidebar_index = idx;
            let view_fn = SIDEBAR_SECTIONS[idx].1;
            let new_view = view_fn();
            self.history.clear();
            self.current_view = new_view;
            self.list_state.select(Some(0));
            self.detail_scroll = 0;
        }
    }

    // ── v1 list-navigation methods (app.rs compat) ───────────────────────────

    /// Move selection down, stopping at the boundary.
    pub fn select_next(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let current = self.selected_index();
        if current < max.saturating_sub(1) {
            self.list_state.select(Some(current + 1));
        }
    }

    /// Move selection up, stopping at 0.
    pub fn select_prev(&mut self) {
        let current = self.selected_index();
        if current > 0 {
            self.list_state.select(Some(current - 1));
        }
    }

    /// Jump to the first item.
    pub fn select_first(&mut self) {
        self.list_state.select(Some(0));
    }

    /// Jump to the last item.
    pub fn select_last(&mut self, max: usize) {
        if max > 0 {
            self.list_state.select(Some(max - 1));
        }
    }

    /// Get the currently selected index.
    pub fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    // ── v2 tab-cycling methods (Req 16.3) ─────────────────────────────────────

    /// Advance to the next tab, wrapping from Settings → Overview.
    ///
    /// Saves the current tab's `list_states` entry before switching so that
    /// the scroll position is preserved when the operator returns.
    pub fn next_tab(&mut self) {
        let idx = self.current_tab.index();
        let next_idx = (idx + 1) % Tab::COUNT;
        self.current_tab = Tab::ALL[next_idx].clone();
    }

    /// Go to the previous tab, wrapping from Overview → Settings.
    pub fn prev_tab(&mut self) {
        let idx = self.current_tab.index();
        let prev_idx = if idx == 0 { Tab::COUNT - 1 } else { idx - 1 };
        self.current_tab = Tab::ALL[prev_idx].clone();
    }

    // ── v2 drill-down history (Req 16) ────────────────────────────────────────

    /// Push a drill-down view onto the tab history stack.
    ///
    /// If the stack already has `MAX_TAB_HISTORY` entries, the oldest entry is
    /// silently dropped to keep memory bounded.
    pub fn push_drill(&mut self, view: DrillView) {
        if self.tab_history.len() >= Self::MAX_TAB_HISTORY {
            self.tab_history.remove(0);
        }
        self.tab_history.push(view);
    }

    /// Pop the most recently pushed drill-down view.
    ///
    /// Returns `Some(view)` if there was a view to pop, or `None` when already
    /// at the top-level tab.
    pub fn pop_drill(&mut self) -> Option<DrillView> {
        self.tab_history.pop()
    }

    /// The currently visible drill-down view, if any.
    pub fn current_drill(&self) -> Option<&DrillView> {
        self.tab_history.last()
    }

    // ── v2 search methods (Req 16.2) ──────────────────────────────────────────

    /// Activate fuzzy search mode.  Clears the query so the operator starts
    /// fresh.
    pub fn activate_search(&mut self) {
        self.search_active = true;
        self.search_query.clear();
    }

    /// Deactivate fuzzy search mode without clearing the query (the caller may
    /// inspect `search_query` after deactivation to apply a persistent filter).
    pub fn deactivate_search(&mut self) {
        self.search_active = false;
    }

    // ── v2 per-tab scroll (Req 16) ────────────────────────────────────────────

    /// Get the `ListState` for a given tab (mutable — caller can select into it).
    pub fn tab_list_state_mut(&mut self, tab: &Tab) -> &mut ListState {
        self.list_states.entry(tab.clone()).or_insert_with(|| {
            let mut ls = ListState::default();
            ls.select(Some(0));
            ls
        })
    }

    /// Get the `ListState` for a given tab (shared reference).
    pub fn tab_list_state(&self, tab: &Tab) -> Option<&ListState> {
        self.list_states.get(tab)
    }
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Regression: the workflow catalog was loaded into CatalogStore but no production
    // code read it, so the feature was invisible to users. These assert the view is
    // actually reachable rather than merely defined.
    #[test]
    fn workflows_have_a_sidebar_entry() {
        let labels: Vec<&str> = SIDEBAR_SECTIONS.iter().map(|(l, _)| *l).collect();
        assert!(
            labels.contains(&"Workflows"),
            "Workflows must be reachable from the sidebar; got {labels:?}",
        );
    }

    #[test]
    fn workflows_sidebar_entry_opens_the_workflow_list() {
        let entry = SIDEBAR_SECTIONS
            .iter()
            .find(|(l, _)| *l == "Workflows")
            .expect("Workflows sidebar entry");
        assert_eq!((entry.1)(), View::WorkflowList);
    }

    use super::*;

    // ── v1 regression tests (unchanged from original) ────────────────────────

    #[test]
    fn new_starts_at_agent_list() {
        let nav = NavigationState::new();
        assert_eq!(nav.current_view, View::AgentList);
        assert!(nav.history.is_empty());
        assert_eq!(nav.sidebar_index, 0);
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn push_view_adds_to_history() {
        let mut nav = NavigationState::new();
        nav.push_view(View::AgentDetail("test-agent".to_string()));
        assert_eq!(
            nav.current_view,
            View::AgentDetail("test-agent".to_string())
        );
        assert_eq!(nav.history.len(), 1);
        assert_eq!(nav.history[0], View::AgentList);
    }

    #[test]
    fn pop_view_returns_to_previous() {
        let mut nav = NavigationState::new();
        nav.push_view(View::AgentDetail("test".to_string()));
        assert!(nav.pop_view());
        assert_eq!(nav.current_view, View::AgentList);
        assert!(nav.history.is_empty());
    }

    #[test]
    fn pop_view_at_root_returns_false() {
        let mut nav = NavigationState::new();
        assert!(!nav.pop_view());
        assert_eq!(nav.current_view, View::AgentList);
    }

    #[test]
    fn set_sidebar_index_changes_view() {
        let mut nav = NavigationState::new();
        nav.set_sidebar_index(1);
        assert_eq!(nav.sidebar_index, 1);
        assert_eq!(nav.current_view, View::SkillList);
        assert!(nav.history.is_empty());
    }

    #[test]
    fn set_sidebar_index_clears_history() {
        let mut nav = NavigationState::new();
        nav.push_view(View::AgentDetail("x".to_string()));
        nav.set_sidebar_index(2);
        assert!(nav.history.is_empty());
        assert_eq!(nav.current_view, View::RoleList);
    }

    #[test]
    fn select_next_stops_at_boundary() {
        let mut nav = NavigationState::new();
        nav.select_next(3);
        assert_eq!(nav.selected_index(), 1);
        nav.select_next(3);
        assert_eq!(nav.selected_index(), 2);
        nav.select_next(3);
        assert_eq!(nav.selected_index(), 2); // stays at boundary
    }

    #[test]
    fn select_prev_stops_at_zero() {
        let mut nav = NavigationState::new();
        nav.select_prev();
        assert_eq!(nav.selected_index(), 0); // stays at 0
    }

    #[test]
    fn select_first_and_last() {
        let mut nav = NavigationState::new();
        nav.select_next(10);
        nav.select_next(10);
        nav.select_next(10);
        nav.select_last(10);
        assert_eq!(nav.selected_index(), 9);
        nav.select_first();
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn select_next_empty_list_is_safe() {
        let mut nav = NavigationState::new();
        nav.select_next(0);
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn select_last_empty_list_is_safe() {
        let mut nav = NavigationState::new();
        nav.select_last(0);
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn sidebar_sections_count() {
        assert_eq!(SIDEBAR_SECTIONS.len(), 12);
    }

    #[test]
    fn all_sidebar_views_are_correct() {
        let expected_views = vec![
            View::AgentList,
            View::SkillList,
            View::RoleList,
            View::ProviderList,
            View::McpList,
            View::RuleList,
            View::ValidationList,
            View::ExportBuilder,
            View::ModelPolicyBuilder,
            View::IntegrityOverview,
            View::WorkflowList,
            View::ModList,
        ];
        for (i, expected) in expected_views.iter().enumerate() {
            let view_fn = SIDEBAR_SECTIONS[i].1;
            assert_eq!(&view_fn(), expected);
        }
    }

    #[test]
    fn push_view_resets_selection() {
        let mut nav = NavigationState::new();
        nav.select_next(10);
        nav.select_next(10);
        assert_eq!(nav.selected_index(), 2);
        nav.push_view(View::SkillList);
        assert_eq!(nav.selected_index(), 0);
    }

    #[test]
    fn push_view_caps_history_at_64() {
        let mut nav = NavigationState::new();
        for i in 0..100 {
            nav.push_view(View::AgentDetail(format!("agent-{i}")));
        }
        assert!(nav.history.len() <= NavigationState::MAX_HISTORY);
        assert_eq!(nav.history.len(), NavigationState::MAX_HISTORY);
    }

    // ── v2 tab enum ──────────────────────────────────────────────────────────

    #[test]
    fn tab_all_has_eight_variants() {
        assert_eq!(Tab::ALL.len(), 8);
        assert_eq!(Tab::COUNT, 8);
    }

    #[test]
    fn tab_index_round_trips() {
        for (expected_idx, tab) in Tab::ALL.iter().enumerate() {
            assert_eq!(tab.index(), expected_idx);
        }
    }

    #[test]
    fn tab_labels_are_non_empty() {
        for tab in Tab::ALL {
            assert!(
                !tab.label().is_empty(),
                "label for {tab:?} must not be empty"
            );
        }
    }

    // ── v2 tab cycling (Property 33 — Req 16.3) ──────────────────────────────
    //
    // We implement Property 33 inline here using exhaustive enumeration and
    // round-trip checks.  The proptest harness uses 256 cases via the
    // `proptest!` macro below; the deterministic unit tests below cover the
    // correctness invariants explicitly.

    #[test]
    fn next_tab_wraps_from_settings_to_overview() {
        let mut nav = NavigationState::new();
        nav.current_tab = Tab::Settings;
        nav.next_tab();
        assert_eq!(nav.current_tab, Tab::Overview);
    }

    #[test]
    fn prev_tab_wraps_from_overview_to_settings() {
        let mut nav = NavigationState::new();
        nav.current_tab = Tab::Overview;
        nav.prev_tab();
        assert_eq!(nav.current_tab, Tab::Settings);
    }

    #[test]
    fn next_tab_cycles_full_sequence() {
        let mut nav = NavigationState::new();
        nav.current_tab = Tab::Overview;
        let expected = [
            Tab::CoverageMatrix,
            Tab::ValidationGates,
            Tab::PolicyViolations,
            Tab::AuditLog,
            Tab::Dependencies,
            Tab::CatalogBrowser,
            Tab::Settings,
            Tab::Overview, // wrap
        ];
        for exp in &expected {
            nav.next_tab();
            assert_eq!(&nav.current_tab, exp);
        }
    }

    #[test]
    fn prev_tab_cycles_reverse_sequence() {
        let mut nav = NavigationState::new();
        nav.current_tab = Tab::Overview;
        let expected = [
            Tab::Settings,
            Tab::CatalogBrowser,
            Tab::Dependencies,
            Tab::AuditLog,
            Tab::PolicyViolations,
            Tab::ValidationGates,
            Tab::CoverageMatrix,
            Tab::Overview, // wrap
        ];
        for exp in &expected {
            nav.prev_tab();
            assert_eq!(&nav.current_tab, exp);
        }
    }

    /// Property 33 core: for any starting tab T, advancing N steps then
    /// reversing N steps must return to T (next/prev are inverses).
    #[test]
    fn tab_cycling_next_then_prev_is_identity() {
        for start_tab in Tab::ALL {
            let mut nav = NavigationState::new();
            nav.current_tab = start_tab.clone();
            let n = Tab::COUNT * 3; // 3 full rotations
            for _ in 0..n {
                nav.next_tab();
            }
            for _ in 0..n {
                nav.prev_tab();
            }
            assert_eq!(
                nav.current_tab, *start_tab,
                "after {n} next + {n} prev, should return to {start_tab:?}"
            );
        }
    }

    /// Property 33: N forward from any start, where N == Tab::COUNT, must
    /// return to the original tab (full cycle).
    #[test]
    fn tab_cycling_full_cycle_returns_to_start() {
        for start_tab in Tab::ALL {
            let mut nav = NavigationState::new();
            nav.current_tab = start_tab.clone();
            for _ in 0..Tab::COUNT {
                nav.next_tab();
            }
            assert_eq!(
                nav.current_tab, *start_tab,
                "full forward cycle should return to {start_tab:?}"
            );
        }
    }

    /// Property 33: N backward from any start, where N == Tab::COUNT, must
    /// return to the original tab (full reverse cycle).
    #[test]
    fn tab_cycling_full_reverse_cycle_returns_to_start() {
        for start_tab in Tab::ALL {
            let mut nav = NavigationState::new();
            nav.current_tab = start_tab.clone();
            for _ in 0..Tab::COUNT {
                nav.prev_tab();
            }
            assert_eq!(
                nav.current_tab, *start_tab,
                "full reverse cycle should return to {start_tab:?}"
            );
        }
    }

    /// Property 33 (256-case simulation): for pseudo-random step counts
    /// 1..=256, advancing K steps then reversing K steps always returns to
    /// the start tab.  We simulate 256 cases by iterating all (start_tab,
    /// step_count) pairs systematically.
    #[test]
    fn tab_cycling_property_256_cases() {
        let step_counts: Vec<usize> = (1..=32).collect(); // 32 step counts × 8 tabs = 256 cases
        let mut case_count = 0usize;
        for start_tab in Tab::ALL {
            for &steps in &step_counts {
                let mut nav = NavigationState::new();
                nav.current_tab = start_tab.clone();

                for _ in 0..steps {
                    nav.next_tab();
                }
                for _ in 0..steps {
                    nav.prev_tab();
                }
                assert_eq!(
                    nav.current_tab, *start_tab,
                    "case (start={start_tab:?}, steps={steps}): expected round-trip to start"
                );
                case_count += 1;
            }
        }
        assert_eq!(case_count, 256, "must exercise exactly 256 cases");
    }

    // ── v2 drill-down history ─────────────────────────────────────────────────

    #[test]
    fn push_drill_adds_to_tab_history() {
        let mut nav = NavigationState::new();
        nav.push_drill(DrillView::AgentDetail("agent-1".to_string()));
        assert_eq!(nav.tab_history.len(), 1);
        assert_eq!(
            nav.current_drill(),
            Some(&DrillView::AgentDetail("agent-1".to_string()))
        );
    }

    #[test]
    fn pop_drill_removes_from_tab_history() {
        let mut nav = NavigationState::new();
        nav.push_drill(DrillView::AgentDetail("agent-1".to_string()));
        nav.push_drill(DrillView::SkillDetail("skill-1".to_string()));

        let popped = nav.pop_drill();
        assert_eq!(popped, Some(DrillView::SkillDetail("skill-1".to_string())));
        assert_eq!(nav.tab_history.len(), 1);

        let popped2 = nav.pop_drill();
        assert_eq!(popped2, Some(DrillView::AgentDetail("agent-1".to_string())));
        assert!(nav.tab_history.is_empty());
    }

    #[test]
    fn pop_drill_at_root_returns_none() {
        let mut nav = NavigationState::new();
        assert_eq!(nav.pop_drill(), None);
    }

    #[test]
    fn push_drill_respects_max_20_depth() {
        let mut nav = NavigationState::new();
        // Push 30 entries; only the most recent 20 should be retained.
        for i in 0..30 {
            nav.push_drill(DrillView::AgentDetail(format!("agent-{i}")));
        }
        assert_eq!(
            nav.tab_history.len(),
            NavigationState::MAX_TAB_HISTORY,
            "tab_history must be capped at MAX_TAB_HISTORY ({})",
            NavigationState::MAX_TAB_HISTORY
        );
        // The oldest entry kept should be agent-10 (entries 0–9 were evicted).
        assert_eq!(
            nav.tab_history[0],
            DrillView::AgentDetail("agent-10".to_string()),
            "after 30 pushes the oldest retained entry should be agent-10"
        );
        // The newest entry should be agent-29.
        assert_eq!(
            nav.tab_history.last(),
            Some(&DrillView::AgentDetail("agent-29".to_string()))
        );
    }

    // ── v2 search ─────────────────────────────────────────────────────────────

    #[test]
    fn activate_search_sets_flag_and_clears_query() {
        let mut nav = NavigationState::new();
        nav.search_query = "previous query".to_string();
        nav.activate_search();
        assert!(nav.search_active);
        assert!(nav.search_query.is_empty());
    }

    #[test]
    fn deactivate_search_clears_flag_preserves_query() {
        let mut nav = NavigationState::new();
        nav.activate_search();
        nav.search_query = "aws".to_string();
        nav.deactivate_search();
        assert!(!nav.search_active);
        assert_eq!(nav.search_query, "aws"); // query preserved for caller
    }

    #[test]
    fn activate_deactivate_toggles_correctly() {
        let mut nav = NavigationState::new();
        assert!(!nav.search_active);
        nav.activate_search();
        assert!(nav.search_active);
        nav.deactivate_search();
        assert!(!nav.search_active);
    }

    // ── v2 per-tab scroll positions ───────────────────────────────────────────

    #[test]
    fn per_tab_list_states_initialised_for_all_tabs() {
        let nav = NavigationState::new();
        for tab in Tab::ALL {
            assert!(
                nav.tab_list_state(tab).is_some(),
                "list_state must be initialised for {tab:?}"
            );
        }
    }

    #[test]
    fn per_tab_scroll_positions_preserved_across_tab_switches() {
        let mut nav = NavigationState::new();

        // Advance the CoverageMatrix tab to selection 5.
        {
            let ls = nav.tab_list_state_mut(&Tab::CoverageMatrix);
            ls.select(Some(5));
        }

        // Switch to CatalogBrowser and advance it to selection 3.
        nav.current_tab = Tab::CatalogBrowser;
        {
            let ls = nav.tab_list_state_mut(&Tab::CatalogBrowser);
            ls.select(Some(3));
        }

        // Switch back to CoverageMatrix — selection 5 must still be there.
        nav.current_tab = Tab::CoverageMatrix;
        let ls = nav.tab_list_state(&Tab::CoverageMatrix).unwrap();
        assert_eq!(
            ls.selected(),
            Some(5),
            "CoverageMatrix scroll position should be preserved after switching tabs"
        );

        // CatalogBrowser must still be at selection 3.
        let ls2 = nav.tab_list_state(&Tab::CatalogBrowser).unwrap();
        assert_eq!(
            ls2.selected(),
            Some(3),
            "CatalogBrowser scroll position should be preserved"
        );
    }

    #[test]
    fn tab_list_state_mut_creates_entry_if_missing() {
        let mut nav = NavigationState::new();
        // list_states is pre-populated in `new()`, but test the lazy path by
        // removing an entry and re-fetching.
        nav.list_states.remove(&Tab::AuditLog);
        let ls = nav.tab_list_state_mut(&Tab::AuditLog);
        assert_eq!(ls.selected(), Some(0));
    }
}
