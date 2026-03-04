#![forbid(unsafe_code)]

use crate::web_search_plan::write::WriteFormatMode;

pub const PRESENTATION_POLICY_VERSION: &str = "run33-presentation-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationPlan {
    pub inline_citations_per_bullet: usize,
    pub include_conflict_detail: bool,
}

pub fn plan_for_mode(mode: WriteFormatMode) -> PresentationPlan {
    match mode {
        WriteFormatMode::Brief => PresentationPlan {
            inline_citations_per_bullet: 1,
            include_conflict_detail: false,
        },
        WriteFormatMode::Standard => PresentationPlan {
            inline_citations_per_bullet: usize::MAX,
            include_conflict_detail: false,
        },
        WriteFormatMode::Deep => PresentationPlan {
            inline_citations_per_bullet: usize::MAX,
            include_conflict_detail: true,
        },
    }
}
