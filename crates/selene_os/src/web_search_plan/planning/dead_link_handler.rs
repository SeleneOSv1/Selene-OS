#![forbid(unsafe_code)]

pub fn should_select_replacement(reason_code: &str) -> bool {
    !matches!(reason_code, "budget_exhausted")
}
