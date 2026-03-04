#![forbid(unsafe_code)]

pub mod claim_confidence;
pub mod freshness_watchdog;
pub mod injection_defense;
pub mod table_render;
pub mod transparency;
pub mod unknown_first;

pub const GAP_CLOSERS_VERSION: &str = "1.0.0";

#[cfg(test)]
pub mod gap_closers_tests;
