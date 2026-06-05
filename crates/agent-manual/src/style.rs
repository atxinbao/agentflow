use crate::model::{AgentStyleState, MANUAL_LANGUAGE, PLAIN_WORK_STYLE_ID, STYLE_VERSION};

pub(crate) fn expected_style_state(checked_at: u64) -> AgentStyleState {
    AgentStyleState {
        version: STYLE_VERSION.to_string(),
        style_id: PLAIN_WORK_STYLE_ID.to_string(),
        manual_language: MANUAL_LANGUAGE.to_string(),
        applies_to_agent_locale: true,
        applies_to_code_comments: true,
        checked_at,
        warnings: Vec::new(),
    }
}
