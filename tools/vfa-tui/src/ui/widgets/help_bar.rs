use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::nav::View;

use super::super::theme::Theme;

/// Render context-sensitive help bar with keybindings for the current view.
pub fn render_help_bar(view: &View, area: Rect, frame: &mut Frame, theme: &Theme) {
    let help_text = match view {
        View::ModList => " j/k:Navigate  Enter:Details  Left/Right:Section  Tab:Tabs  q:Quit",
        View::ModDetail(_) => " j/k:Scroll  Esc:Back  Left/Right:Section  Tab:Tabs  q:Quit",
        View::AgentList
        | View::SkillList
        | View::RoleList
        | View::ProviderList
        | View::McpList
        | View::RuleList
        | View::WorkflowList
        | View::ValidationList => {
            " j/k:Navigate  Enter:Select  /:Search  Tab:Section  g/G:Top/Bot  q:Quit"
        }
        View::AgentDetail(_)
        | View::SkillDetail(_)
        | View::RoleDetail(_)
        | View::McpDetail(_)
        | View::RuleDetail(_)
        | View::WorkflowDetail(_)
        | View::IntegrityDetail(_) => " j/k:Scroll  Esc:Back  Tab:Section  q:Quit",
        View::ProviderAgents(_) => {
            " j/k:Navigate  Enter:Select  Esc:Back  Tab:Section  g/G:Top/Bot  q:Quit"
        }
        View::ValidationOutput(_) | View::ExportOutput | View::ModelPolicyOutput => {
            " j/k:Scroll  Esc:Back  q:Quit"
        }
        View::ExportBuilder | View::ModelPolicyBuilder => {
            " j/k:Fields  Enter:Edit/Confirm  Esc:Back  Tab:Section  q:Quit"
        }
        View::ExportConfirm | View::ModelPolicyConfirm => " Enter:Execute  Esc:Cancel  q:Quit",
        View::IntegrityOverview => " j/k:Navigate  Enter:Details  Tab:Section  g/G:Top/Bot  q:Quit",
    };

    let line = Line::from(vec![Span::styled(help_text, theme.help_bar())]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
